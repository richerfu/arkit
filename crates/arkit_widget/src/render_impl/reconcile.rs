use super::*;

pub(super) fn reconcile_children<Message, AppTheme>(
    parent: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
    next_children: Vec<Element<Message, AppTheme>>,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    type ChildKey = (TypeId, String);

    fn child_key(mounted: &MountedRenderNode) -> Option<ChildKey> {
        mounted.key.clone().map(|key| (mounted.tag, key))
    }

    fn can_reuse<Message, AppTheme>(
        next: &Node<Message, AppTheme>,
        mounted: &MountedRenderNode,
    ) -> bool {
        node_type_id(next.kind) == mounted.tag && node_signature(next) == mounted_signature(mounted)
    }

    let mut next_nodes = Vec::with_capacity(next_children.len());
    for child in next_children {
        let node = into_node(child);
        next_nodes.push(node);
    }

    let next_len = next_nodes.len();
    let pending_exits = mounted.exiting_children.clone();
    let mounted_children = &mut mounted.children;

    for (index, child) in next_nodes.into_iter().enumerate() {
        if child.kind == NodeKind::Retained {
            if index >= mounted_children.len() {
                panic!("retained renderer child has no mounted subtree at index {index}");
            }
            continue;
        }

        let next_key = child.key.clone().map(|key| (node_type_id(child.kind), key));
        let current_key = mounted_children.get(index).and_then(child_key);
        let matches = mounted_children.get(index).is_some_and(|mounted_child| {
            if next_key.is_none() && current_key.is_none() {
                can_reuse(&child, mounted_child)
            } else {
                next_key == current_key && can_reuse(&child, mounted_child)
            }
        });

        if matches {
            let child_handle = parent.children()[index].clone();
            let mut child_node = child_handle.borrow_mut();
            patch_node(child.into(), &mut child_node, &mut mounted_children[index])?;
            continue;
        }

        if index < mounted_children.len() {
            let mounted = mounted_children.remove(index);
            remove_or_exit_child(parent, index, mounted, pending_exits.clone())?;
        }

        let (child_node, mut child_meta) = mount_node(child.into())?;
        attach_child_at(parent, child_node, index)?;
        if let Some(child_handle) = parent.children().get(index) {
            let mut child_node = child_handle.borrow_mut();
            realize_attached_node(&mut child_node, &mut child_meta)?;
        }
        mounted_children.insert(index, child_meta);
    }

    while mounted_children.len() > next_len {
        let index = mounted_children.len() - 1;
        let mounted = mounted_children.remove(index);
        remove_or_exit_child(parent, index, mounted, pending_exits.clone())?;
    }

    Ok(())
}
