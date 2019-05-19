//! Ruby integers.

use std::{
    fmt,
    ffi::c_void,
    mem,
    os::raw::c_int,
    slice,
};
use crate::{
    prelude::*,
    object::{NonNullObject, Ty},
    ruby::self,
};

/// An instance of Ruby's `Integer` class.
#[derive(Clone, Copy, Debug)]
pub struct Integer(NonNullObject);

impl AsRef<AnyObject> for Integer {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Integer> for AnyObject {
    #[inline]
    fn from(obj: Integer) -> Self { obj.0.into() }
}

impl<O: Object> PartialEq<O> for Integer {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        self.as_any_object() == other
    }
}

unsafe impl Object for Integer {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!((Ty::Fixnum as u128) | ((Ty::Bignum as u128) << 8)))
    }

    #[inline]
    fn cast<A: Object>(object: A) -> Option<Self> {
        if object.into_any_object().is_integer() {
            unsafe { Some(Self::cast_unchecked(object)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty {
        if self.is_fixnum() {
            Ty::Fixnum
        } else {
            Ty::Bignum
        }
    }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool {
        self.ty() == ty
    }
}

impl From<usize> for Integer {
    #[inline]
    fn from(int: usize) -> Self {
        unsafe { Self::from_raw(ruby::rb_uint2inum(int)) }
    }
}

impl From<isize> for Integer {
    #[inline]
    fn from(int: isize) -> Self {
        unsafe { Self::from_raw(ruby::rb_int2inum(int)) }
    }
}

impl From<u128> for Integer {
    #[inline]
    fn from(int: u128) -> Self {
        Self::unpack(slice::from_ref(&int))
    }
}

impl From<i128> for Integer {
    #[inline]
    fn from(int: i128) -> Self {
        Self::unpack(slice::from_ref(&int))
    }
}

impl From<u64> for Integer {
    #[inline]
    fn from(int: u64) -> Self {
        if mem::size_of::<u64>() == mem::size_of::<usize>() {
            (int as usize).into()
        } else {
            Self::unpack(slice::from_ref(&int))
        }
    }
}

impl From<i64> for Integer {
    #[inline]
    fn from(int: i64) -> Self {
        if mem::size_of::<i64>() == mem::size_of::<isize>() {
            (int as isize).into()
        } else {
            Self::unpack(slice::from_ref(&int))
        }
    }
}

impl From<u32> for Integer {
    #[inline]
    fn from(int: u32) -> Self {
        (int as usize).into()
    }
}

impl From<i32> for Integer {
    #[inline]
    fn from(int: i32) -> Self {
        (int as isize).into()
    }
}

impl From<u16> for Integer {
    #[inline]
    fn from(int: u16) -> Self {
        (int as usize).into()
    }
}

impl From<i16> for Integer {
    #[inline]
    fn from(int: i16) -> Self {
        (int as isize).into()
    }
}

impl From<u8> for Integer {
    #[inline]
    fn from(int: u8) -> Self {
        (int as usize).into()
    }
}

impl From<i8> for Integer {
    #[inline]
    fn from(int: i8) -> Self {
        (int as isize).into()
    }
}

macro_rules! forward_from {
    ($($t:ty)+) => { $(
        impl From<$t> for AnyObject {
            #[inline]
            fn from(int: $t) -> Self {
                Integer::from(int).into()
            }
        }
    )+ }
}

forward_from! {
    usize u128 u64 u32 u16 u8
    isize i128 i64 i32 i16 i8
}

impl fmt::Display for Integer {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl Integer {
    /// Unpacks the contents of `buf` into a new instance, making the result
    /// negative if `is_negative`.
    #[inline]
    pub fn unpack<W: Word>(buf: &[W]) -> Self {
        Self::unpack_using(buf, PackOptions::default())
    }

    /// Unpacks the contents of `buf` into a new instance using `options`,
    /// making the result negative if `is_negative`.
    #[inline]
    pub fn unpack_using<W: Word>(buf: &[W], options: PackOptions) -> Self {
        use ruby::integer_flags::*;

        let ptr = buf.as_ptr() as *const c_void;
        let len = buf.len();
        let size = mem::size_of::<W>();

        let two = (W::IS_SIGNED as c_int) * PACK_2COMP;
        let neg = (options.is_negative as c_int) * PACK_NEGATIVE;
        let flags = options.flags() | two | neg;

        unsafe {
            Self::from_raw(ruby::rb_integer_unpack(ptr, len, size, 0, flags))
        }
    }

    /// Returns whether `self >= 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::Integer;
    ///
    /// let big = Integer::from(u128::max_value());
    /// let fix = Integer::from(isize::max_value() / 2);
    /// # assert!(big.is_bignum());
    /// # assert!(fix.is_fixnum());
    ///
    /// assert!(big.is_positive());
    /// assert!(fix.is_positive());
    /// ```
    #[inline]
    pub fn is_positive(self) -> bool {
        !self.is_negative()
    }

    /// Returns whether `self < 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::Integer;
    ///
    /// let big = Integer::from(i128::min_value());
    /// let fix = Integer::from(isize::min_value() / 2);
    /// # assert!(big.is_bignum());
    /// # assert!(fix.is_fixnum());
    ///
    /// assert!(big.is_negative());
    /// assert!(fix.is_negative());
    /// ```
    #[inline]
    pub fn is_negative(self) -> bool {
        if self.is_fixnum() {
            (self.raw() as isize) < 0
        } else {
            unsafe { ruby::rb_big_sign(self.raw()) == 0 }
        }
    }

    /// Returns whether `self` is a variable-sized integer.
    #[inline]
    pub fn is_bignum(self) -> bool {
        !self.is_fixnum()
    }

    /// Returns whether `self` is a fixed-sized integer.
    #[inline]
    pub fn is_fixnum(self) -> bool {
        crate::util::value_is_fixnum(self.raw())
    }

    /// Returns the value of the fixed-width integer stored in `self`.
    #[inline]
    pub fn fixnum_value(self) -> Option<i64> {
        if self.is_fixnum() {
            Some(crate::util::value_to_fixnum(self.raw()) as i64)
        } else {
            None
        }
    }

    /// Converts `self` to its inner value as `W`, truncating on too large or
    /// small of a value.
    ///
    /// # Examples
    ///
    /// This has the same exact behavior as an
    /// [`as` cast](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#type-cast-expressions)
    /// between integer primitives in Rust:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// let val = u16::max_value();
    /// let int = rosy::Integer::from(val);
    ///
    /// assert_eq!(int.to_truncated::<u16>(), val);
    /// assert_eq!(int.to_truncated::<u8>(), 255);
    /// assert_eq!(int.to_truncated::<i8>(), -1);
    /// # }).unwrap();
    /// ```
    #[inline]
    pub fn to_truncated<W: Word>(self) -> W {
        let mut val = W::ZERO;
        self.pack(slice::from_mut(&mut val));
        val
    }

    /// Packs the contents of `self` into `buf` with the platform's native byte
    /// order.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// use std::slice;
    /// use rosy::Integer;
    ///
    /// let value = u128::max_value() / 0xF00F;
    /// let integer = Integer::from(value);
    ///
    /// let mut buf = [0u128; 2];
    /// integer.pack(&mut buf);
    /// assert_eq!(buf[0], value);
    /// # }).unwrap();
    /// ```
    #[inline]
    pub fn pack<W: Word>(self, buf: &mut [W]) -> PackSign {
        self.pack_using(PackOptions::default(), buf)
    }

    /// Packs the contents of `self` into `buf` using `options`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// use std::slice;
    /// use rosy::integer::{Integer, PackOptions};
    ///
    /// let value = u128::max_value() / 0xF00F;
    /// let integer = Integer::from(value);
    ///
    /// let mut be_buf = [0u128; 1];
    /// integer.pack_using(PackOptions::big_endian(), &mut be_buf);
    /// assert_eq!(be_buf[0], value.to_be());
    ///
    /// let mut le_buf = [0u128; 1];
    /// integer.pack_using(PackOptions::little_endian(), &mut le_buf);
    /// assert_eq!(le_buf[0], value.to_le());
    /// # }).unwrap();
    /// ```
    #[inline]
    pub fn pack_using<W: Word>(
        self,
        options: PackOptions,
        buf: &mut [W],
    ) -> PackSign {
        use ruby::integer_flags::*;

        let raw = self.raw();
        let ptr = buf.as_mut_ptr() as *mut c_void;
        let num = buf.len();
        let size = mem::size_of::<W>();

        let flags = options.flags() | ((W::IS_SIGNED as c_int) * PACK_2COMP);

        match unsafe { ruby::rb_integer_pack(raw, ptr, num, size, 0, flags) } {
            02 => PackSign::Positive { did_overflow: true },
            01 => PackSign::Positive { did_overflow: false },
            00 => PackSign::Zero,
            -1 => PackSign::Negative { did_overflow: false },
            _  => PackSign::Negative { did_overflow: true },
        }
    }
}

/// Options to use when packing/unpacking.
#[derive(Clone, Copy, Debug)]
pub struct PackOptions {
    byte_order: Order,
    word_order: Order,
    is_negative: bool,
}

impl Default for PackOptions {
    #[inline]
    fn default() -> Self {
        PackOptions {
            word_order: Order::Least,

            #[cfg(target_endian = "little")]
            byte_order: Order::Least,

            #[cfg(target_endian = "big")]
            byte_order: Order::Most,

            is_negative: false,
        }
    }
}

impl PackOptions {
    #[inline]
    fn flags(self) -> c_int {
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
/// [`pack_using`](struct.Integer.html#method.pack_using).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// Least-significant first.
    Least,
    /// Most-significant first.
    Most,
}

/// The sign of an [`Integer`](struct.Integer.html) value returned after
/// [`pack`](struct.Integer.html#method.pack)ing one into a buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackSign {
    /// Packing resulted in a value equal to 0.
    Zero,
    /// Packing resulted in a positive value.
    Positive {
        /// An overflow occurred when packing an
        /// [`Integer`](struct.Integer.html) into a buffer.
        did_overflow: bool,
    },
    /// Packing resulted in a negative value.
    Negative {
        /// An overflow occurred when packing an
        /// [`Integer`](struct.Integer.html) into a buffer.
        did_overflow: bool,
    },
}

impl PackSign {
    /// Returns whether an overflow occurred when packing an
    /// [`Integer`](struct.Integer.html) into a buffer.
    #[inline]
    pub fn did_overflow(&self) -> bool {
        use PackSign::*;
        match *self {
            Zero => false,
            Positive { did_overflow } |
            Negative { did_overflow } => did_overflow,
        }
    }

    /// Returns whether the sign is negative.
    #[inline]
    pub fn is_negative(&self) -> bool {
        if let PackSign::Negative { .. } = *self {
            true
        } else {
            false
        }
    }
}

/// A type whose bytes can be directly used as a word when packing an
/// [`Integer`](struct.Integer.html).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        crate::vm::init().unwrap();

        macro_rules! test {
            ($($t:ty)+) => { $({
                let values = [
                    0,
                    <$t>::min_value(),
                    <$t>::max_value(),
                ];
                for &value in &values {
                    let int = Integer::from(value);
                    assert_eq!(int.to_s(), value.to_string());

                    let mut buf: [$t; 1] = [0];
                    let sign = int.pack(&mut buf);
                    assert!(
                        !sign.did_overflow(),
                        "Packing {} from {} overflowed as {:?}",
                        int,
                        value,
                        sign,
                    );
                    assert_eq!(buf[0], value);
                }
            })+ }
        }

        crate::protected(|| {
            test! {
                usize u128 u64 u32 u16 u8
                isize i128 i64 i32 i16 i8
            }
        }).unwrap();
    }
}
