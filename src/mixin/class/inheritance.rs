/// The [`inheritance`](struct.Class.html#method.inheritance) relationship
/// between two classes.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Inheritance {
    /// There is no relationship between the two classes.
    None,
    /// The first class inherits or is the same as the second; `A < B`.
    SubEq,
    /// The second class inherits the first; `B < A`.
    Super,
}

impl Inheritance {
    /// Returns whether there's no relationship between the classes.
    #[inline]
    pub fn is_none(self) -> bool {
        self == Inheritance::None
    }

    /// Returns whether the first class inherits or is the same as the second.
    #[inline]
    pub fn is_sub_eq(self) -> bool {
        self == Inheritance::SubEq
    }

    /// Returns whether the second class inherits the first.
    #[inline]
    pub fn is_super(self) -> bool {
        self == Inheritance::Super
    }
}
