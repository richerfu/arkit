//! Property composition modes shared by all target adapters.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Composition {
    #[default]
    Replace,
    Add,
    Accumulate,
}
