use crate::ohos_arkui_binding::component::built_in_component::Text;
use crate::prelude::ArkUINodeAttributeType;

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
        self.style(ArkUINodeAttributeType::TextContent, content.clone())
            .patch_attr(ArkUINodeAttributeType::TextContent, content)
    }
}
