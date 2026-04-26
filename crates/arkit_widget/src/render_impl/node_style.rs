use super::*;

impl<Message, AppTheme> Node<Message, AppTheme> {
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self = self.builder_attr(ArkUINodeAttributeType::WidthPercent, 1.0_f32);
            }
            Length::FillPortion(portion) => {
                self = self.builder_attr(ArkUINodeAttributeType::LayoutWeight, f32::from(portion));
            }
            Length::Fixed(value) => {
                self = self.builder_attr(ArkUINodeAttributeType::Width, value);
            }
        }
        self
    }

    pub fn height(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self = self.builder_attr(ArkUINodeAttributeType::HeightPercent, 1.0_f32);
            }
            Length::FillPortion(portion) => {
                self = self.builder_attr(ArkUINodeAttributeType::LayoutWeight, f32::from(portion));
            }
            Length::Fixed(value) => {
                self = self.builder_attr(ArkUINodeAttributeType::Height, value);
            }
        }
        self
    }

    pub fn percent_width(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::WidthPercent, value)
    }

    pub fn percent_height(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::HeightPercent, value)
    }

    pub fn max_width_constraint(self, value: f32) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::ConstraintSize,
            vec![0.0_f32, value, 0.0_f32, 100_000.0_f32],
        )
    }

    pub fn constraint_size(
        self,
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        let value = vec![min_width, max_width, min_height, max_height];
        self.builder_attr(ArkUINodeAttributeType::ConstraintSize, value)
    }

    pub fn background_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BackgroundColor, value)
    }

    pub fn padding(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Padding, padding_edges(value.into()))
    }

    pub fn padding_x(self, value: f32) -> Self {
        self.padding(Padding::symmetric(value, 0.0))
    }

    pub fn padding_y(self, value: f32) -> Self {
        self.padding(Padding::symmetric(0.0, value))
    }

    pub fn margin(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Margin, padding_edges(value.into()))
    }

    pub fn margin_x(self, value: f32) -> Self {
        self.margin(Padding::symmetric(value, 0.0))
    }

    pub fn margin_y(self, value: f32) -> Self {
        self.margin(Padding::symmetric(0.0, value))
    }

    pub fn margin_top(self, value: f32) -> Self {
        self.margin(Padding {
            top: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_right(self, value: f32) -> Self {
        self.margin(Padding {
            right: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_bottom(self, value: f32) -> Self {
        self.margin(Padding {
            bottom: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_left(self, value: f32) -> Self {
        self.margin(Padding {
            left: value,
            ..Padding::ZERO
        })
    }

    pub fn foreground_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ForegroundColor, value)
    }

    pub fn font_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontColor, value)
    }

    pub fn font_weight(self, value: FontWeight) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontWeight, font_weight_value(value))
    }

    pub fn font_family(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.builder_attr(ArkUINodeAttributeType::FontFamily, value)
    }

    pub fn font_style(self, value: FontStyle) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontStyle, i32::from(value))
    }

    pub fn font_size(self, value: f32) -> Self {
        let node = self.builder_attr(ArkUINodeAttributeType::FontSize, value);
        let placeholder_attr = match node.kind {
            NodeKind::TextInput => Some(ArkUINodeAttributeType::TextInputPlaceholderFont),
            NodeKind::TextArea => Some(ArkUINodeAttributeType::TextAreaPlaceholderFont),
            _ => None,
        };

        if let Some(attr) = placeholder_attr {
            node.builder_attr(
                attr,
                ArkUINodeAttributeItem::NumberValue(vec![ArkUINodeAttributeNumber::Float(value)]),
            )
        } else {
            node
        }
    }

    pub fn line_height(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextLineHeight, value)
    }

    pub fn text_align(self, value: TextAlignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextAlign, i32::from(value))
    }

    pub fn text_letter_spacing(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextLetterSpacing, value)
    }

    pub fn text_decoration(self, value: impl Into<ArkUINodeAttributeItem>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextDecoration, value)
    }

    pub fn enabled(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Enabled, value)
    }

    pub fn opacity(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Opacity, value)
    }

    pub fn clip(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Clip, value)
    }

    pub fn focusable(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Focusable, value)
    }

    pub fn focus_on_touch(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FocusOnTouch, value)
    }

    pub fn border_radius(self, value: impl EdgeAttributeValue) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderRadius, value.edge_values())
    }

    pub fn border_width(self, value: impl EdgeAttributeValue) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderWidth, value.edge_values())
    }

    pub fn border_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderColor, vec![value])
    }

    pub fn border_color_all(self, value: u32) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::BorderColor,
            vec![value, value, value, value],
        )
    }

    pub fn border_style(self, value: BorderStyle) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderStyle, i32::from(value))
    }

    pub fn shadow(self, value: ShadowStyle) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::Shadow,
            vec![shadow_style_value(value)],
        )
    }

    pub fn custom_shadow(
        self,
        blur_radius: f32,
        offset_x: f32,
        offset_y: f32,
        color: u32,
        fill: bool,
    ) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::CustomShadow,
            vec![
                ArkUINodeAttributeNumber::Float(blur_radius),
                ArkUINodeAttributeNumber::Int(0),
                ArkUINodeAttributeNumber::Float(offset_x),
                ArkUINodeAttributeNumber::Float(offset_y),
                ArkUINodeAttributeNumber::Int(0),
                ArkUINodeAttributeNumber::Uint(color),
                ArkUINodeAttributeNumber::Uint(u32::from(fill)),
            ],
        )
    }

    pub fn clear_shadow(self) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Shadow, vec![0_i32])
    }

    pub fn alignment(self, value: Alignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Alignment, i32::from(value))
    }

    pub fn align_self(self, value: ItemAlignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::AlignSelf, i32::from(value))
    }

    pub fn layout_weight(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::LayoutWeight, value)
    }

    pub fn visibility(self, value: Visibility) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Visibility, i32::from(value))
    }

    pub fn hit_test_behavior(self, value: HitTestBehavior) -> Self {
        self.builder_attr(ArkUINodeAttributeType::HitTestBehavior, i32::from(value))
    }

    pub fn button_type(self, value: ButtonType) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ButtonType, i32::from(value))
    }

    pub fn color_blend(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ColorBlend, value)
    }

    pub fn position(self, x: f32, y: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Position, vec![x, y])
    }

    pub fn z_index(self, value: i32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ZIndex, value)
    }

    pub fn aspect_ratio(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::AspectRatio, value)
    }

    pub fn image_object_fit(self, value: ObjectFit) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ImageObjectFit, i32::from(value))
    }

    pub fn progress_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ProgressColor, value)
    }

    pub fn progress_type(self, value: ProgressType) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ProgressType, i32::from(value))
    }

    pub fn progress_linear_style(mut self, value: ProgressLinearStyle) -> Self {
        self.patch_effects.push(Box::new(move |node| {
            apply_progress_linear_style(node, value)
        }));
        self.mount_effects.push(Box::new(move |node| {
            apply_progress_linear_style(node, value)?;
            Ok(None)
        }));
        self
    }
}
