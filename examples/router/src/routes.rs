#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserNavigationState {
    pub(crate) source: String,
    pub(crate) scroll_offset: u32,
}
