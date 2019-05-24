use std::ops::{
    Bound,
    Range,
    RangeInclusive,
    RangeFrom,
};

/// A type that consists of a start (inclusive) and end bound.
pub trait IntoBounds<S, E> {
    /// Returns the start (inclusive) and end bounds of `self`.
    fn into_bounds(self) -> (S, Bound<E>);
}

impl<S, E> IntoBounds<S, E> for (S, Bound<E>) {
    #[inline]
    fn into_bounds(self) -> (S, Bound<E>) {
        self
    }
}

impl<S, E> IntoBounds<S, E> for (S, E) {
    #[inline]
    fn into_bounds(self) -> (S, Bound<E>) {
        (self.0, Bound::Excluded(self.1))
    }
}

impl<A> IntoBounds<A, A> for Range<A> {
    #[inline]
    fn into_bounds(self) -> (A, Bound<A>) {
        (self.start, Bound::Excluded(self.end))
    }
}

impl<A> IntoBounds<A, A> for RangeInclusive<A> {
    #[inline]
    fn into_bounds(self) -> (A, Bound<A>) {
        let (start, end) = self.into_inner();
        (start, Bound::Included(end))
    }
}

impl<A> IntoBounds<A, A> for RangeFrom<A> {
    #[inline]
    fn into_bounds(self) -> (A, Bound<A>) {
        (self.start, Bound::Unbounded)
    }
}
