//! [`Integer`](../struct.Integer.html) (un)packing.

use std::os::raw::c_int;
use crate::ruby;

/// A type whose bytes can be directly used as a word when (un)packing an
/// [`Integer`](../struct.Integer.html).
pub unsafe trait Word: Copy {
    /// Whether the type is a signed integer.
    const IS_SIGNED: bool;

    /// `Self` instantiated as 0.
    const ZERO: Self;
}

macro_rules! impl_word {
    ($signed:expr => $($t:ty)+) => { $(
        unsafe impl Word for $t {
            const IS_SIGNED: bool = $signed;

            const ZERO: Self = 0;
        }
    )+ }
}

impl_word! { false => usize u128 u64 u32 u16 u8 }
impl_word! { true  => isize i128 i64 i32 i16 i8 }

/// Options to use when (un)packing.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub(super) byte_order: Order,
    pub(super) word_order: Order,
    pub(super) is_negative: bool,
}

impl Default for Options {
    #[inline]
    fn default() -> Self {
        Options {
            word_order: Order::Least,

            #[cfg(target_endian = "little")]
            byte_order: Order::Least,

            #[cfg(target_endian = "big")]
            byte_order: Order::Most,

            is_negative: false,
        }
    }
}

impl Options {
    #[inline]
    pub(super) fn flags(self) -> c_int {
        use ruby::integer_flags::*;

        let byte_order = match self.byte_order {
            Order::Least => PACK_LSBYTE_FIRST,
            Order::Most  => PACK_MSBYTE_FIRST,
        };
        let word_order = match self.word_order {
            Order::Least => PACK_LSWORD_FIRST,
            Order::Most  => PACK_MSWORD_FIRST,
        };

        word_order | byte_order
    }

    /// Returns a new instance for big-endian byte order.
    #[inline]
    pub fn big_endian() -> Self {
        Self::default().byte_order(Order::Most)
    }

    /// Returns a new instance for little-endian byte order.
    #[inline]
    pub fn little_endian() -> Self {
        Self::default().byte_order(Order::Least)
    }

    /// Sets the [endianness](https://en.wikipedia.org/wiki/Endianness) for each
    /// word.
    ///
    /// The default is the platform's native byte order:
    #[cfg_attr(target_endian = "little", doc = "**little-endian**.")]
    #[cfg_attr(target_endian = "big",    doc = "**big-endian**.")]
    #[inline]
    #[must_use]
    pub fn byte_order(mut self, order: Order) -> Self {
        self.byte_order = order;
        self
    }

    /// Sets the order in which words should be packed.
    ///
    /// The default is least-significant first.
    #[inline]
    #[must_use]
    pub fn word_order(mut self, order: Order) -> Self {
        self.word_order = order;
        self
    }

    /// Makes the `Integer` instance negative. This is only used when unpacking.
    #[inline]
    pub fn is_negative(mut self) -> Self {
        self.is_negative = true;
        self
    }
}

/// An order for arranging words and the bytes of those words when calling
/// [`pack_using`](../struct.Integer.html#method.pack_using).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// Least-significant first.
    Least,
    /// Most-significant first.
    Most,
}

/// The sign of an [`Integer`](../struct.Integer.html) value returned after
/// [`pack`](../struct.Integer.html#method.pack)ing one into a buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    /// Packing resulted in a value equal to 0.
    Zero,
    /// Packing resulted in a positive value.
    Positive {
        /// An overflow occurred when packing an
        /// [`Integer`](../struct.Integer.html) into a buffer.
        did_overflow: bool,
    },
    /// Packing resulted in a negative value.
    Negative {
        /// An overflow occurred when packing an
        /// [`Integer`](../struct.Integer.html) into a buffer.
        did_overflow: bool,
    },
}

impl Sign {
    /// Returns whether an overflow occurred when packing an
    /// [`Integer`](../struct.Integer.html) into a buffer.
    #[inline]
    pub fn did_overflow(&self) -> bool {
        use Sign::*;
        match *self {
            Zero => false,
            Positive { did_overflow } |
            Negative { did_overflow } => did_overflow,
        }
    }

    /// Returns whether the sign is negative.
    #[inline]
    pub fn is_negative(&self) -> bool {
        if let Sign::Negative { .. } = *self {
            true
        } else {
            false
        }
    }
}
