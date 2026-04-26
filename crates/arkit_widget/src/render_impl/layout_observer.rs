use super::*;

pub(crate) fn read_layout_frame(node: &ArkUINode) -> Option<LayoutFrame> {
    let size = read_layout_size(node)?;
    let position = node
        .position_with_translate_in_window()
        .or_else(|_| node.layout_position_in_window())
        .ok()?;
    Some(LayoutFrame {
        x: position.x as f32,
        y: position.y as f32,
        width: size.width,
        height: size.height,
    })
}

pub fn observe_layout_size<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    on_change: impl Fn(LayoutSize) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let on_change = Rc::new(on_change) as Rc<dyn Fn(LayoutSize)>;

    into_node(element)
        .with_patch({
            let node_ref = node_ref.clone();
            let on_change = on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if let Some(size) = read_layout_size(&runtime) {
                    on_change(size);
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            if let Some(node) = node_ref.borrow().as_ref() {
                if let Some(size) = read_layout_size(node) {
                    on_change(size);
                }
            }
        })
        .into()
}

pub fn observe_text_layout<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    text: impl Into<String>,
    on_change: impl Fn(TextLayoutSnapshot) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let text = Rc::new(text.into());
    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let on_change = Rc::new(on_change) as Rc<dyn Fn(TextLayoutSnapshot)>;
    const RETRY_FRAMES: u8 = 12;

    into_node(element)
        .with_patch({
            let text = text.clone();
            let node_ref = node_ref.clone();
            let on_change = on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if !emit_text_layout_snapshot(&runtime, text.as_str(), &on_change) {
                    schedule_text_layout_snapshot(
                        runtime.clone(),
                        text.clone(),
                        on_change.clone(),
                        RETRY_FRAMES,
                    );
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            if let Some(node) = node_ref.borrow().as_ref() {
                if !emit_text_layout_snapshot(node, text.as_str(), &on_change) {
                    schedule_text_layout_snapshot(
                        node.clone(),
                        text.clone(),
                        on_change.clone(),
                        RETRY_FRAMES,
                    );
                }
            }
        })
        .into()
}

fn emit_text_layout_snapshot(
    node: &ArkUINode,
    text: &str,
    on_change: &Rc<dyn Fn(TextLayoutSnapshot)>,
) -> bool {
    if let Some(snapshot) = read_text_layout_snapshot(node, text) {
        on_change(snapshot);
        true
    } else {
        false
    }
}

fn schedule_text_layout_snapshot(
    node: ArkUINode,
    text: Rc<String>,
    on_change: Rc<dyn Fn(TextLayoutSnapshot)>,
    retry_frames: u8,
) {
    let _ = node.post_frame_callback({
        let node = node.clone();
        move |_, _| {
            if emit_text_layout_snapshot(&node, text.as_str(), &on_change) || retry_frames == 0 {
                return;
            }
            schedule_text_layout_snapshot(
                node.clone(),
                text.clone(),
                on_change.clone(),
                retry_frames - 1,
            );
        }
    });
}

fn read_text_layout_snapshot(node: &ArkUINode, text: &str) -> Option<TextLayoutSnapshot> {
    let size = read_layout_size(node)?;
    let manager = node.text_layout_manager().ok()??;

    let line_count = manager.get_line_count().ok()?;
    if line_count <= 0 {
        manager.dispose();
        return None;
    }

    let mut native_lines = Vec::with_capacity(line_count as usize);
    for index in 0..line_count {
        let Ok(metrics) = manager.get_line_metrics(index) else {
            continue;
        };
        native_lines.push((
            index as usize,
            metrics.y as f32,
            (metrics.y + metrics.height) as f32,
            metrics.start_index,
            metrics.end_index,
        ));
    }

    if native_lines.is_empty() {
        manager.dispose();
        return None;
    }

    let native_lines = fill_text_layout_offsets(text, &manager, size.width, native_lines);
    manager.dispose();

    if native_lines.is_empty() {
        return None;
    }

    let offset_unit = TextOffsetUnit::infer(text, native_lines.iter().map(|line| line.4).max());
    let lines = native_lines
        .into_iter()
        .filter_map(|(index, top, bottom, start, end)| {
            let start = offset_unit.to_byte_offset(text, start)?;
            let end = offset_unit.to_byte_offset(text, end)?;
            Some(TextLayoutLine {
                index,
                top,
                bottom,
                start: start.min(end),
                end: end.max(start),
            })
        })
        .collect::<Vec<_>>();

    if lines.is_empty() {
        return None;
    }

    Some(TextLayoutSnapshot {
        width: size.width,
        height: size.height,
        line_count: lines.len(),
        lines,
    })
}

fn fill_text_layout_offsets(
    text: &str,
    manager: &TextLayoutManager,
    width: f32,
    native_lines: Vec<(usize, f32, f32, usize, usize)>,
) -> Vec<(usize, f32, f32, usize, usize)> {
    if native_lines
        .iter()
        .any(|(_, _, _, start, end)| end > start && *end <= text.len())
    {
        return native_lines;
    }

    let mut previous_end = 0usize;
    native_lines
        .into_iter()
        .map(|(index, top, bottom, start, end)| {
            let y = ((top + bottom) / 2.0) as f64;
            let glyph_end = manager
                .get_glyph_position_at_coordinate(width.max(1.0) as f64, y)
                .ok()
                .map(|position| position.position())
                .filter(|offset| *offset > previous_end)
                .unwrap_or(end);
            let line_start = start.max(previous_end);
            previous_end = glyph_end.max(line_start);
            (index, top, bottom, line_start, previous_end)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextOffsetUnit {
    Bytes,
    Utf16,
    BoundaryChecked,
}

impl TextOffsetUnit {
    fn infer(text: &str, max_offset: Option<usize>) -> Self {
        let Some(max_offset) = max_offset else {
            return Self::BoundaryChecked;
        };
        if max_offset == text.len() {
            Self::Bytes
        } else if max_offset == text.encode_utf16().count() {
            Self::Utf16
        } else {
            Self::BoundaryChecked
        }
    }

    fn to_byte_offset(self, text: &str, offset: usize) -> Option<usize> {
        match self {
            Self::Bytes => text.is_char_boundary(offset).then_some(offset),
            Self::Utf16 => utf16_offset_to_byte_offset(text, offset),
            Self::BoundaryChecked => {
                if offset <= text.len() && text.is_char_boundary(offset) {
                    Some(offset)
                } else {
                    utf16_offset_to_byte_offset(text, offset)
                }
            }
        }
    }
}

fn utf16_offset_to_byte_offset(text: &str, target: usize) -> Option<usize> {
    let mut units = 0usize;
    for (byte_index, ch) in text.char_indices() {
        if units == target {
            return Some(byte_index);
        }
        units += ch.len_utf16();
        if units > target {
            return None;
        }
    }
    (units == target).then_some(text.len())
}

pub fn observe_layout_frame<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    enabled: bool,
    on_change: impl Fn(LayoutFrame) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    if !enabled {
        return element;
    }

    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let on_change = Rc::new(on_change) as Rc<dyn Fn(LayoutFrame)>;

    into_node(element)
        .with_patch({
            let node_ref = node_ref.clone();
            let on_change = on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if let Some(frame) = read_layout_frame(&runtime) {
                    on_change(frame);
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            if let Some(node) = node_ref.borrow().as_ref() {
                if let Some(frame) = read_layout_frame(node) {
                    on_change(frame);
                }
            }
        })
        .into()
}
