use std::{
    mem::{self, ManuallyDrop},
    panic,
    ptr,
    thread::Result,
};
use crate::{
    AnyException,
    AnyObject,
    Object,
    ruby,
};

/// Calls `f` and returns its output or an exception if one is raised in `f`.
///
/// # Examples
///
/// This is great for calling methods that may not exist:
///
/// ```
/// # rosy::vm::init();
/// use rosy::{Object, String, protected};
///
/// let string = String::from("¡Hola!");
/// let result = protected(|| unsafe { string.call_unchecked("likes_pie?") });
///
/// assert!(result.is_err());
/// ```
///
/// Calls can even be nested like so:
///
/// ```
/// # rosy::vm::init();
/// use rosy::{Object, String, protected};
///
/// let string = String::from("Hiii!!!");
///
/// let outer = protected(|| {
///     protected(|| unsafe {
///         string.call_unchecked("likes_pie?")
///     }).unwrap_err();
///     string
/// });
///
/// assert_eq!(outer.unwrap(), string);
/// ```
pub fn protected<F, O>(f: F) -> crate::Result<O>
    where F: FnOnce() -> O
{
    unsafe extern "C" fn wrapper<F, O>(ctx: ruby::VALUE) -> ruby::VALUE
        where F: FnOnce() -> O
    {
        let (f, out) = &mut *(ctx as *mut (Option<F>, &mut Result<O>));

        // Get the `F` out of `Option<F>` to call by-value, which is required by
        // the `FnOnce` trait
        let f = f.take().unwrap_or_else(|| std::hint::unreachable_unchecked());

        ptr::write(*out, panic::catch_unwind(panic::AssertUnwindSafe(f)));

        AnyObject::nil().raw()
    }
    unsafe {
        // These shenanigans allow us to pass in a pointer to `f`, with a
        // pointer to its uninitialized output, into `rb_protect` to make them
        // accessible from `wrapper`
        let mut out = ManuallyDrop::new(mem::uninitialized::<Result<O>>());
        let mut ctx = (Some(f), &mut *out);
        let ctx = &mut ctx as *mut (Option<F>, &mut _) as ruby::VALUE;

        let mut err = 1;
        ruby::rb_protect(Some(wrapper::<F, O>), ctx, &mut err);
        match err {
            0 => match ManuallyDrop::into_inner(out) {
                Ok(out) => Ok(out),
                Err(panic_info) => panic::resume_unwind(panic_info),
            },
            _ => Err(AnyException::_take_current()),
        }
    }
}

/// Calls the non-panicking function `f` and returns its output or an exception
/// if one is raised in `f`.
///
/// See [`protected`](fn.protected.html) for usage information.
///
/// This function is allowed to perform certain optimizations what wouldn't be
/// possible if it needed to take panics into consideration. This can result in
/// a large reduction of instructions emitted.
///
/// # Safety
///
/// Because `f` is called within the context of a foreign C function, panicking
/// will cause undefined behavior.
pub unsafe fn protected_no_panic<F, O>(f: F) -> crate::Result<O>
    where F: FnOnce() -> O
{
    if crate::util::matches_ruby_size_align::<O>() {
        return protected_no_panic_size_opt(f);
    }

    unsafe extern "C" fn wrapper<F, O>(ctx: ruby::VALUE) -> ruby::VALUE
        where F: FnOnce() -> O
    {
        let (f, out) = &mut *(ctx as *mut (Option<F>, &mut O));

        // Get the `F` out of `Option<F>` to call by-value, which is required by
        // the `FnOnce` trait
        let f = f.take().unwrap_or_else(|| std::hint::unreachable_unchecked());

        ptr::write(*out, f());

        AnyObject::nil().raw()
    }

    let mut out = ManuallyDrop::new(mem::uninitialized::<O>());
    let mut ctx = (Some(f), &mut *out);
    let ctx = &mut ctx as *mut (Option<F>, &mut O) as ruby::VALUE;

    let mut err = 0;
    ruby::rb_protect(Some(wrapper::<F, O>), ctx, &mut err);
    match err {
        0 => Ok(ManuallyDrop::into_inner(out)),
        _ => Err(AnyException::_take_current()),
    }
}

// A version of `protected` that makes use of the size and layout of `O`
// matching that of `ruby::VALUE`. This slightly reduces the number of emitted
// instructions and removes the need for stack-allocating `ctx`.
#[inline]
unsafe fn protected_no_panic_size_opt<F, O>(f: F) -> crate::Result<O>
    where F: FnOnce() -> O
{
    use mem::ManuallyDrop;

    unsafe extern "C" fn wrapper<F, O>(ctx: ruby::VALUE) -> ruby::VALUE
        where F: FnOnce() -> O
    {
        let f: &mut Option<F> = &mut *(ctx as *mut Option<F>);

        // Get the `F` out of `Option<F>` to call by-value, which is required by
        // the `FnOnce` trait
        let f = f.take().unwrap_or_else(|| std::hint::unreachable_unchecked());

        let value = ManuallyDrop::new(f());
        ptr::read(&value as *const ManuallyDrop<O> as *const ruby::VALUE)
    }

    let mut ctx = Some(f);
    let ctx = &mut ctx as *mut Option<F> as ruby::VALUE;

    let mut err = 0;
    let val = ruby::rb_protect(Some(wrapper::<F, O>), ctx, &mut err);
    match err {
        0 => Ok(ptr::read(&val as *const ruby::VALUE as *const O)),
        _ => Err(AnyException::_take_current()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic() {
        crate::vm::init().unwrap();

        struct DropAndPanic;

        impl Drop for DropAndPanic {
            fn drop(&mut self) {
                panic!("This was never instantiated and shouldn't be dropped");
            }
        }

        let message = "panic happened";

        panic::catch_unwind(|| {
            protected(|| -> DropAndPanic { panic!("{}", message); }).unwrap();
        }).unwrap_err();
    }
}
