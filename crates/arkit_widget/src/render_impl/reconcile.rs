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
    let old_len = mounted_children.len();
    let mut prefix = 0;

    while prefix < next_len && prefix < old_len {
        let next_key = match next_nodes[prefix].key.clone() {
            Some(key) => Some((node_type_id(next_nodes[prefix].kind), key)),
            None => None,
        };
        let current_key = child_key(&mounted_children[prefix]);
        let matches = if next_key.is_none() && current_key.is_none() {
            can_reuse(&next_nodes[prefix], &mounted_children[prefix])
        } else {
            next_key == current_key && can_reuse(&next_nodes[prefix], &mounted_children[prefix])
        };
        if !matches {
            break;
        }
        prefix += 1;
    }

    for i in 0..prefix {
        let child_handle = parent.children()[i].clone();
        let mut child_node = child_handle.borrow_mut();
        patch_node(
            next_nodes.remove(0).into(),
            &mut child_node,
            &mut mounted_children[i],
        )?;
    }

    if prefix == old_len && prefix == next_len {
        return Ok(());
    }

    for (offset, child) in next_nodes.into_iter().enumerate() {
        let index = prefix + offset;
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
