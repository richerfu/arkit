use arkit_prelude::*;

pub(crate) fn forward<P, M: 'static>(
    render: impl dioxus_core::ComponentFunction<P, M>,
    props: P,
    name: &'static str,
) -> Element
where
    P: dioxus_core::Properties + 'static,
{
    let node =
        dioxus_core::DynamicNode::Component(dioxus_core::VComponent::new(render, props, name));
    rsx! { {node} }
}
