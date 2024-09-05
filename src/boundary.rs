#[allow(unused)]
mod private {
    use core::{fmt::Debug, hash::Hash};

    pub trait BoundaryAssumption:
        Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash + Debug
    {
    }
}

/// Makes no assumption about the offset of the parse helper
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Byte;
impl private::BoundaryAssumption for Byte {}

/// Assumes the offset of the parse helper is always at utf8 boundaries
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Char;
impl private::BoundaryAssumption for Char {}
