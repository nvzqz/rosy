use std::fmt;
use crate::ruby::value_type;

/// A Ruby virtual type.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ty(i32);

impl From<value_type> for Ty {
    #[inline]
    fn from(ty: value_type) -> Self {
        Ty(ty as i32)
    }
}

impl fmt::Debug for Ty {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self.name().unwrap_or("Unknown");
        f.debug_tuple("Ty").field(&name).finish()
    }
}

impl Ty {
    pub(crate) const _UNKNOWN: Ty = Ty(value_type::_Unknown as i32);

    /// None type.
    pub const NONE:     Ty = Ty(value_type::NONE as i32);
    /// Object type.
    pub const OBJECT:   Ty = Ty(value_type::OBJECT as i32);
    /// Class type.
    pub const CLASS:    Ty = Ty(value_type::CLASS as i32);
    /// Module type.
    pub const MODULE:   Ty = Ty(value_type::MODULE as i32);
    /// Float type.
    pub const FLOAT:    Ty = Ty(value_type::FLOAT as i32);
    /// String type.
    pub const STRING:   Ty = Ty(value_type::STRING as i32);
    /// Regexp type.
    pub const REGEXP:   Ty = Ty(value_type::REGEXP as i32);
    /// Array type.
    pub const ARRAY:    Ty = Ty(value_type::ARRAY as i32);
    /// Hash type.
    pub const HASH:     Ty = Ty(value_type::HASH as i32);
    /// Struct type.
    pub const STRUCT:   Ty = Ty(value_type::STRUCT as i32);
    /// Bignum type.
    pub const BIGNUM:   Ty = Ty(value_type::BIGNUM as i32);
    /// File type.
    pub const FILE:     Ty = Ty(value_type::FILE as i32);
    /// Data type.
    pub const DATA:     Ty = Ty(value_type::DATA as i32);
    /// Match type.
    pub const MATCH:    Ty = Ty(value_type::MATCH as i32);
    /// Complex type.
    pub const COMPLEX:  Ty = Ty(value_type::COMPLEX as i32);
    /// Rational type.
    pub const RATIONAL: Ty = Ty(value_type::RATIONAL as i32);
    /// Nil type.
    pub const NIL:      Ty = Ty(value_type::NIL as i32);
    /// True type.
    pub const TRUE:     Ty = Ty(value_type::TRUE as i32);
    /// False type.
    pub const FALSE:    Ty = Ty(value_type::FALSE as i32);
    /// Symbol type.
    pub const SYMBOL:   Ty = Ty(value_type::SYMBOL as i32);
    /// Fixnum type.
    pub const FIXNUM:   Ty = Ty(value_type::FIXNUM as i32);
    /// Undef type.
    pub const UNDEF:    Ty = Ty(value_type::UNDEF as i32);
    /// IMemo type.
    pub const IMEMO:    Ty = Ty(value_type::IMEMO as i32);
    /// Node type.
    pub const NODE:     Ty = Ty(value_type::NODE as i32);
    /// IClass type.
    pub const ICLASS:   Ty = Ty(value_type::ICLASS as i32);
    /// Zombie type.
    pub const ZOMBIE:   Ty = Ty(value_type::ZOMBIE as i32);

    /// Returns the numerical identifier for the type.
    #[inline]
    pub const fn id(self) -> u32 {
        self.0 as u32
    }

    /// Returns a name describing the type.
    #[inline]
    pub fn name<'a>(self) -> Option<&'a str> {
        match self {
            Ty::NONE     => Some("None"),
            Ty::OBJECT   => Some("Object"),
            Ty::CLASS    => Some("Class"),
            Ty::MODULE   => Some("Module"),
            Ty::FLOAT    => Some("Float"),
            Ty::STRING   => Some("String"),
            Ty::REGEXP   => Some("Regexp"),
            Ty::ARRAY    => Some("Array"),
            Ty::HASH     => Some("Hash"),
            Ty::STRUCT   => Some("Struct"),
            Ty::BIGNUM   => Some("Bignum"),
            Ty::FILE     => Some("File"),
            Ty::DATA     => Some("Data"),
            Ty::MATCH    => Some("Match"),
            Ty::COMPLEX  => Some("Complex"),
            Ty::RATIONAL => Some("Rational"),
            Ty::NIL      => Some("Nil"),
            Ty::TRUE     => Some("True"),
            Ty::FALSE    => Some("False"),
            Ty::SYMBOL   => Some("Symbol"),
            Ty::FIXNUM   => Some("Fixnum"),
            Ty::UNDEF    => Some("Undef"),
            Ty::IMEMO    => Some("IMemo"),
            Ty::NODE     => Some("Node"),
            Ty::ICLASS   => Some("IClass"),
            Ty::ZOMBIE   => Some("Zombie"),
            _            => None,
        }
    }
}
