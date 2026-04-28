use super::*;

impl<Message, AppTheme, Kind> Component<Message, AppTheme, Kind> {
    pub fn with(self, effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static) -> Self {
        self.map_node(|node| node.with(effect))
    }

    pub fn with_cleanup<C>(
        self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.map_node(|node| node.with_cleanup(effect))
    }

    pub fn with_exit(
        self,
        effect: impl FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.map_node(|node| node.with_exit(effect))
    }

    pub fn with_exit_cleanup<C>(
        self,
        effect: impl FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.map_node(|node| node.with_exit_cleanup(effect))
    }

    pub fn native(self, effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static) -> Self {
        self.map_node(|node| node.native(effect))
    }

    pub fn with_patch(self, effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static) -> Self {
        self.map_node(|node| node.with_patch(effect))
    }

    pub fn with_next_frame(
        self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.map_node(|node| node.with_next_frame(effect))
    }

    pub fn with_next_idle(
        self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.map_node(|node| node.with_next_idle(effect))
    }

    pub fn native_with_cleanup<C>(
        self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.map_node(|node| node.native_with_cleanup(effect))
    }
}
