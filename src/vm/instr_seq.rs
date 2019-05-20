use std::{fmt, io};
use crate::{
    object::NonNullObject,
    prelude::*,
};

/// An instance of Ruby's `RubyVM::InstructionSequence` class.
///
/// **Note:** The binary data that comes from an instruction sequence is not
/// portable and should not be used in another version or architecture of Ruby.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct InstrSeq(NonNullObject);

impl AsRef<AnyObject> for InstrSeq {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<InstrSeq> for AnyObject {
    #[inline]
    fn from(obj: InstrSeq) -> Self { obj.0.into() }
}

impl PartialEq<AnyObject> for InstrSeq {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

unsafe impl Object for InstrSeq {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some((!0) - 1)
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        if obj.class().inherits(Class::of::<Self>()) {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }
}

impl fmt::Display for InstrSeq {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl InstrSeq {
    #[inline]
    fn _compile(args: &[AnyObject]) -> Result<Self> {
        Class::instr_seq().call_with("compile", args).map(|obj| unsafe {
            Self::cast_unchecked(obj)
        })
    }

    /// Compiles `script` into an instruction sequence.
    #[inline]
    pub fn compile(script: impl Into<String>) -> Result<Self> {
        Self::_compile(&[script.into().into()])
    }

    /// Compiles `script` with `options` into an instruction sequence.
    #[inline]
    pub fn compile_with(
        script: impl Into<String>,
        options: impl Into<Hash>,
    ) -> Result<Self> {
        Self::_compile(&[script.into().into(), options.into().into()])
    }

    #[inline]
    fn _compile_file(args: &[AnyObject]) -> Result<Self> {
        Class::instr_seq().call_with("compile_file", args).map(|obj| unsafe {
            Self::cast_unchecked(obj)
        })
    }

    /// Compiles the contents of a file at `path` into an instruction sequence.
    #[inline]
    pub fn compile_file(path: impl Into<String>) -> Result<Self> {
        Self::_compile_file(&[path.into().into()])
    }

    /// Compiles the contents of a file at `path` with `options into an
    /// instruction sequence.
    #[inline]
    pub fn compile_file_with(
        path: impl Into<String>,
        options: impl Into<Hash>,
    ) -> Result<Self> {
        Self::_compile_file(&[path.into().into(), options.into().into()])
    }

    /// Loads an instruction sequence from a binary formatted string created by
    /// [`to_binary`](#method.to_binary).
    ///
    /// # Safety
    ///
    /// This loader does not have a verifier, so loading broken/modified binary
    /// causes critical problems.
    ///
    /// # Examples
    ///
    /// This is equivalent to calling
    /// `RubyVM::InstructionSequence.load_from_binary`:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// use rosy::{vm::InstrSeq, String};
    ///
    /// let script = "'hi' * 3";
    ///
    /// let seq1 = InstrSeq::compile(script).expect("Invalid script");
    /// let seq2 = unsafe { InstrSeq::from_binary(seq1.to_binary()) };
    ///
    /// assert_eq!(String::from("hihihi"), seq2.eval().unwrap());
    /// # }).unwrap();
    /// ```
    #[inline]
    pub unsafe fn from_binary(binary: impl Into<String>) -> Self {
        Self::cast_unchecked(Class::instr_seq().call_with_unchecked(
            "load_from_binary",
            &[binary.into()]
        ))
    }

    /// Evaluates `self` and returns the result.
    ///
    /// # Examples
    ///
    /// This is equivalent to calling `eval` in a protected context:
    ///
    /// ```
    /// use rosy::{vm::InstrSeq, String};
    /// # rosy::vm::init().unwrap();
    ///
    /// let script = "'hi' * 3";
    /// let instr_seq = InstrSeq::compile(script).expect("Invalid script");
    ///
    /// let result = instr_seq.eval().unwrap();
    /// assert_eq!(String::from("hihihi"), result);
    /// ```
    #[inline]
    pub fn eval(self) -> Result<AnyObject> {
        self.call("eval")
    }

    /// Returns the serialized binary data.
    #[inline]
    pub fn to_binary(self) -> String {
        unsafe { String::cast_unchecked(self.call_unchecked("to_binary")) }
    }

    /// Writes the serialized binary data of `self` to `w`.
    ///
    /// This makes it easy to write the contents of `self` to a
    /// [`File`](https://doc.rust-lang.org/std/fs/struct.File.html) or any other
    /// common I/O type.
    #[inline]
    pub fn write_binary(self, mut w: impl io::Write) -> io::Result<()> {
        let binary = self.to_binary();
        let bytes = unsafe { binary.as_bytes() };
        w.write_all(bytes)
    }

    /// Returns a human-readable form of `self`.
    #[inline]
    pub fn disassemble(self) -> String {
        unsafe { String::cast_unchecked(self.call_unchecked("disasm")) }
    }

    /// Returns the file path of `self`, or `<compiled>` if it was compiled from
    /// a string.
    #[inline]
    pub fn path(self) -> String {
        unsafe { String::cast_unchecked(self.call_unchecked("path")) }
    }

    /// Returns the absolute path of `self` if it was compiled from a file.
    #[inline]
    pub fn absolute_path(self) -> Option<String> {
        unsafe {
            let path = self.call_unchecked("absolute_path");
            if path.is_nil() {
                None
            } else {
                Some(String::cast_unchecked(path))
            }
        }
    }
}
