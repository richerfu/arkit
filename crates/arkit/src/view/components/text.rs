use crate::ohos_arkui_binding::component::built_in_component::Text;

use super::super::core::ComponentElement;

pub type TextElement = ComponentElement<Text>;

pub fn text_component() -> TextElement {
    ComponentElement::new(Text::new)
}

pub fn text(content: impl Into<String>) -> TextElement {
    text_component().content(content)
}

impl ComponentElement<Text> {
    pub fn content(self, content: impl Into<String>) -> Self {
        let content = content.into();
        self.with(move |node| node.content(content))
    }
}
