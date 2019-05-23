<p align="center">
  <a href="https://github.com/oceanpkg/rosy">
    <img width="800" src="https://github.com/oceanpkg/rosy/raw/assets/banner.svg?sanitize=true" alt="rosy banner">
  </a>
  <br>
  <a href="https://travis-ci.com/oceanpkg/rosy">
    <img src="https://travis-ci.com/oceanpkg/rosy.svg?branch=master" alt="travis badge">
  </a>
  <img src="https://tokei.rs/b1/github/oceanpkg/rosy?category=code" alt="lines of code">
  <a href="https://crates.io/crates/rosy">
    <img src="https://img.shields.io/crates/v/rosy.svg" alt="crates.io">
    <img src="https://img.shields.io/crates/d/rosy.svg" alt="downloads">
  </a>
  <a href="https://docs.rs/rosy">
    <img src="https://docs.rs/rosy/badge.svg" alt="API docs">
  </a>
  <br>
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey.svg" alt="platforms">
  <a href="https://github.com/oceanpkg/rosy/blob/master/LICENSE.md">
    <img src="https://img.shields.io/badge/license-MIT%20or%20Apache%202.0-blue.svg" alt="MIT or Apache 2.0">
  </a>
</p>

High-level, zero (or low) cost bindings of [Ruby]'s C API for [Rust].

## Index
- [Features](#features)
- [Installation](#installation)
- [Building](#building)
- [Usage](#usage)
  - [Managing Ruby's Virtual Machine](#managing-rubys-virtual-machine)
  - [Calling Ruby Methods](#calling-ruby-methods)
  - [Defining Ruby Methods](#defining-ruby-methods)
  - [Defining Ruby Classes](#defining-ruby-classes)
  - [Defining Ruby Subclasses](#defining-ruby-subclasses)
  - [Catching Ruby Exceptions](#catching-ruby-exceptions)
- [Platform Support](#platform-support)
- [Library Comparison](#library-comparison)
  - [Rosy vs Helix](#rosy-vs-helix)
  - [Rosy vs Rutie](#rosy-vs-rutie)
- [Authors](#authors)
- [License](#license)

## Features

- **Performance:**

  Rosy enables you to write the most performant code possible, such that using
  the C API directly would not improve performance. In other words, it presents
  [zero-cost abstractions](https://boats.gitlab.io/blog/post/zero-cost-abstractions).
  However, not all of Rosy's _safe_ abstractions are zero-cost. Sometimes this
  is only possible by writing some `unsafe` code since Rosy can't be made aware
  of certain aspects of the program state.

  - For example, [`Object::call`] will catch any raised Ruby exceptions via
    the [`protected`] family of functions. On the other hand,
    [`Object::call_unchecked`] will allow any thrown exception propagate
    (which causes a segmentation fault in Rust-land) unless [`protected`].

    Checking for exceptions via [`protected`] has a cost associated with it.
    It is best to wrap multiple instances of unchecked exception-throwing
    functions. This allows for reducing the number of speed bumps in your code.

  - If it is known that no [`panic!`] will occur anywhere within
    exception-checked code, then calling [`protected_no_panic`] will emit fewer
    instructions at the cost of safety. The [`FnOnce`] passed into this function
    is called within an FFI context; because of that, [panicking here is
    undefined behavior][panic-ffi-ub]. Panics in a normal [`protected`] call are
    safely caught with the stack unwinding properly.

  Note that `unsafe` functions suffixed with `_unchecked` always have a safe
  counterpart. Before reaching for `unsafe` functions, consider using these
  instead and profiling your code to find out whether it's actually necessary.

- **Powerful Types:**

  Rosy leverages Rust's type system to the fullest.

  - Rosy makes certain Ruby types generic over enclosing types:

    - [`Array<O>`][`Array`] is generic over [`Object`] types that it contains,
      defaulting to [`AnyObject`].

    - [`Hash<K, V>`] is generic over [`Object`] keys and values, both defaulting
      to [`AnyObject`].

    - [`Class<O>`][`Class`] is generic over an [`Object`] type that it may
      instantiate via [`Class::new_instance`].

  - When defining methods via [`Class::def_method`] or [`def_method!`]:

    - The receiver is statically typed as the generic [`Object`] type that the
      [`Class`] is meant for.

    - Arguments (excluding the receiver) are generic up to and including 15
      [`AnyObject`]s. It may also take either an [`Array`] or an [`AnyObject`]
      pointer paired with a length. These allow for passing in a variable number
      of arguments.

- **Safety:** <sup>*where possible</sup>

  Rosy exposes safe abstractions over most of Ruby's C API. Wherever this isn't
  possible, such functionality is marked as `unsafe` with a documented
  explanation on safe usage.

  Unfortunately, due to the inherent nature of Ruby's C API, safety is often not
  easily achievable without a few compromises.

  - _Many_ Ruby functions can raise exceptions,
    which trigger a [segmentation fault] in Rust-land. 😓

    Functions that may raise an exception are marked as `unsafe` or have a safe
    exception-checking equivalent via [`protected`]. However, checking for an
    exception has a cost in performance.

  - Ruby's garbage collector de-deallocates objects whose references don't live
    on the stack, unless they are [`mark`]ed. This may lead to a possible
    [use after free]. When wrapping Rust data, it is important to implement
    [`Rosy::mark`] correctly.

## Installation

Rosy requires [`cargo`] and an existing Ruby installation:

- `cargo` can be installed via [`rustup`](https://rustup.rs)

- Ruby can be installed [`rvm`], [`rbenv`], or your favorite package manager

The `rosy` crate is available [on crates.io][crate] and can be used by adding
the following to your project's [`Cargo.toml`]:

```toml
[dependencies]
rosy = "0.0.8"
```

Rosy has functionality that is only available for certain Ruby versions. The
following features can currently be enabled:

- `ruby_2_6`

For example:

```toml
[dependencies.rosy]
version = "0.0.8"
features = ["ruby_2_6"]
```

Finally add this to your crate root (`main.rs` or `lib.rs`):

```rust
extern crate rosy;
```

## Building

Rosy can be compiled by simply running:

```sh
cargo build
```

It will automatically try to find the dynamic library via the current `ruby`
installation.

To enable static linking, specify the `static` feature flag:

```toml
[dependencies.rosy]
version = "0.0.8"
features = ["static"]
```

To use a specific Ruby installation, you can do either of the following:

- Set `ROSY_RUBY=path/to/ruby`

  This must point to an executable.

- Set `ROSY_RUBY=client:version`. For example:

  - `ROSY_RUBY=rvm:2.6.0`

  - `ROSY_RUBY=rbenv:2.5.0`

  If the `:version` portion is not provided, then `ROSY_RUBY_VERSION` is used to
  get the version number. For example:

  ```sh
  ROSY_RUBY=rvm ROSY_RUBY_VERSION=2.6.0 cargo build
  ```

## Usage

Rosy allows you to perform _many_ operations over Ruby objects in a way that
feels very natural in Rust.

### Managing Ruby's Virtual Machine

The virtual machine must be initialized via [`vm::init`] before doing anything:

```rust
rosy::vm::init().expect("Could not initialize Ruby");
```

Once finished with Ruby, you can clean up its resources permanently via
[`vm::destroy`]:

```rust
if let Err(code) = unsafe { rosy::vm::destroy() } {
    std::process::exit(code);
}
```

Note that you can no longer use Ruby once the VM has been destroyed.

### Calling Ruby Methods

Using [`Object::call`], any method can be invoked on the receiving object:

```rust
use rosy::String;

let string = String::from("hello\r\n");
string.call("chomp!").unwrap();

assert_eq!(string, "hello");
```

To pass in arguments, use [`Object::call_with`]:

```rust
use rosy::{Array, Integer, Object};

let array: Array<Integer> = (10..20).collect();

let args: &[Integer] = &[1.into(), 5.into(), 9.into()];

let values = array.call_with("values_at", args).unwrap();

assert_eq!(values, [11, 15, 19][..]);
```

### Defining Ruby Methods

To define a [UTF-8]-aware method `blank?` on Ruby's `String` class, one can very
simply use the [`def_method!`] macro. This allows for defining a function that
takes the typed object (in this case `String`) for the class as its receiver.

```rust
use rosy::prelude::*;

let class = Class::of::<String>();

rosy::def_method!(class, "blank?", |this: String| {
    this.is_whitespace()
}).unwrap();

let string = String::from(" \r\n");
let result = string.call("blank?");

assert_eq!(result.unwrap(), true);
```

Although the macro may feel somewhat magical, it's actually just a zero-cost
wrapper around [`Class::def_method`], which itself is a low-cost abstraction
over `rb_define_method_id`. To bring the abstraction cost down to absolute zero,
use [`def_method_unchecked!`].

### Defining Ruby Classes

Defining a new class is rather straightforward:

```rust
let my_object = Class::def("MyObject").unwrap();
```

Attempting to define an existing class will result in an error:

```rust
let array = Class::def("Array")
    .unwrap_err()
    .existing_class()
    .unwrap();

assert_eq!(array, Class::array());
```

To get an existing named class if it's not a
[built-in class](https://docs.rs/rosy/0.0.8/rosy/struct.Class.html#impl-1),
one should call [`Class::get`]:

```rust
let my_object = Class::get("MyObject").unwrap();
```

And if it's ambiguous as to whether the class already exists, there's the best
of both worlds: [`Class::get_or_def`]. This will define a class with the given
name if it doesn't already exist.

```rust
let my_object = Class::get_or_def("MyObject").unwrap();
```

To define a class within the namespace of a class or module, use
[`Mixin::def_class`].

### Defining Ruby Subclasses

The [`Class::subclass`] method allows for creating a new class that inherits
from the method receiver's class.

```rust
let sub_object = my_object.subclass("MyObjectChild").unwrap();

assert!(sub_object < my_object);
```

To define a subclass within the namespace of a class or module, use
[`Mixin::def_subclass`].

### Catching Ruby Exceptions

Rust code can be [`protected`] from Ruby exceptions very easily.

```rust
use rosy::{Object, String, protected};

let string = String::from("¡Hola!");

let result = protected(|| unsafe {
    string.call_unchecked("likes_pie?")
});

assert!(result.unwrap_err().is_no_method_error());
```

## Platform Support

- [x] [Linux](https://github.com/oceanpkg/rosy/issues/1)

- [x] [macOS](https://github.com/oceanpkg/rosy/issues/2)

- [ ] [Windows](https://github.com/oceanpkg/rosy/issues/3)

Rosy uses [`aloxide`] to find and link Ruby during its build phase. Because of
that, Rosy's platform support is totally dependent on it. Changes that fix
issues with linking (or in the future, building) Ruby should be submitted to
that library for use in this one.

To work locally on `aloxide` and Rosy in combination with each other, change
Rosy's [`Cargo.toml`] like so:

```toml
[build-dependencies]
aloxide = { path = "path/to/aloxide", version = "0.0.8", default-features = false }
```

## Library Comparison

Like with most technologies, Rosy isn't the first of its kind.

### Rosy vs Helix

[Helix] is a Rust library built on top of macros. Interaction with the Ruby
runtime is done via a `ruby!` macro which features a [DSL] that's a mix between
Rust and Ruby syntax. To those coming from Ruby, they'll feel right at home.
However, those coming from Rust may feel that the macro is a little _too_
magical.

Unlike Helix, for each of Rosy's macros, there's an alternative approach that
can be taken purely through types, traits, and functions. Rosy is designed to be
convenient and high-level while trying not to hide the low-level details that
can allow you to write better-optimized code. This is parallel to the way that
Rust acts as a high-level language.

### Rosy vs Rutie

[Rutie] is a Rust library that tries to be less magical than Helix. It is a
continuation of the work done on [ruru], which is no longer maintained as of the
end of 2017. Rutie takes an excellent approach to wrapping Ruby's C API in Rust
by exposing Ruby classes as Rust `struct`s. This inspired the layout and design
of Rosy to some extent.

However, unlike Rutie, Rosy doesn't expose the lower-level C bindings. The
reasoning is that if certain functionality is missing from Rosy, it should be
added to the core library by either requesting it through an [issue][issues] or
submitting a [pull request][pulls] with an implementation.

Also, unlike Rutie, Rosy marks all exception-throwing functions as `unsafe`. Not
handling a Ruby exception from Rust-land results in a [segmentation fault]. One
of the major reasons that some people choose to write Rust over C is to get away
from these. The Rust philosophy is that safe code should not be able to trigger
a segmentation fault. Just like with Rutie, Rosy allows Rust code to be
[`protected`] against raised exceptions.

## Authors

- **Creator:** [Nikolai Vazquez](https://github.com/nvzqz)

  <a href="https://www.patreon.com/nvzqz" target="_blank" rel="noopener noreferrer">
    <img src="https://c5.patreon.com/external/logo/become_a_patron_button.png" alt="Become a Patron!" height="30">
  </a>
  <a href="https://www.paypal.me/nvzqz" target="_blank" rel="noopener noreferrer">
    <img src="https://buymecoffee.intm.org/img/button-paypal-white.png" alt="Buy me a coffee" height="30">
  </a>

## License

This project is made available under either the conditions of the
[MIT License](https://choosealicense.com/licenses/mit/) or
[Apache License 2.0](https://choosealicense.com/licenses/apache-2.0/)
at your choosing.

See [`LICENSE.md`][license].

----

Congrats on making it this far! ʕﾉ•ᴥ•ʔﾉ🌹

[Back to top](#top)

[`aloxide`]:      https://github.com/oceanpkg/aloxide
[Ruby]:           https://www.ruby-lang.org
[Rust]:           https://www.rust-lang.org
[`cargo`]:        https://doc.rust-lang.org/cargo/
[`Cargo.toml`]:   https://doc.rust-lang.org/cargo/reference/manifest.html
[`rvm`]:          https://rvm.io
[`rbenv`]:        https://github.com/rbenv/rbenv
[Helix]:          https://usehelix.com
[Rutie]:          https://github.com/danielpclark/rutie
[ruru]:           https://github.com/d-unseductable/ruru

[DSL]:                https://en.wikipedia.org/wiki/Domain-specific_language
[panic-ffi-ub]:       https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-panics
[segmentation fault]: https://en.wikipedia.org/wiki/Segmentation_fault
[use after free]:     https://cwe.mitre.org/data/definitions/416.html
[UTF-8]:              https://en.wikipedia.org/wiki/UTF-8

[issues]:  https://github.com/oceanpkg/rosy/issues
[pulls]:   https://github.com/oceanpkg/rosy/pulls
[crate]:   https://crates.io/crates/rosy
[license]: https://github.com/oceanpkg/rosy/blob/master/LICENSE.md

[`FnOnce`]: https://doc.rust-lang.org/std/ops/trait.FnOnce.html
[`panic!`]: https://doc.rust-lang.org/stable/std/macro.panic.html

[`AnyObject`]:              https://docs.rs/rosy/0.0.8/rosy/struct.AnyObject.html
[`Array`]:                  https://docs.rs/rosy/0.0.8/rosy/struct.Array.html
[`Class::def_method`]:      https://docs.rs/rosy/0.0.8/rosy/struct.Class.html#method.def_method
[`Class::get_or_def`]:      https://docs.rs/rosy/0.0.8/rosy/struct.Class.html#method.get_or_def
[`Class::get`]:             https://docs.rs/rosy/0.0.8/rosy/struct.Class.html#method.get
[`Class::new_instance`]:    https://docs.rs/rosy/0.0.8/rosy/struct.Class.html#method.new_instances
[`Class::subclass`]:        https://docs.rs/rosy/0.0.8/rosy/struct.Class.html#method.subclass
[`Class`]:                  https://docs.rs/rosy/0.0.8/rosy/struct.Class.html
[`def_method_unchecked!`]:  https://docs.rs/rosy/0.0.8/rosy/macro.def_method_unchecked.html
[`def_method!`]:            https://docs.rs/rosy/0.0.8/rosy/macro.def_method.html
[`Hash<K, V>`]:             https://docs.rs/rosy/0.0.8/rosy/struct.Hash.html
[`mark`]:                   https://docs.rs/rosy/0.0.8/rosy/gc/fn.mark.html
[`Mixin::def_class`]:       https://docs.rs/rosy/0.0.8/rosy/trait.Mixin.html#method.def_class
[`Mixin::def_subclass`]:    https://docs.rs/rosy/0.0.8/rosy/trait.Mixin.html#method.def_subclass
[`Object::call_unchecked`]: https://docs.rs/rosy/0.0.8/rosy/trait.Object.html#method.call_unchecked
[`Object::call_with`]:      https://docs.rs/rosy/0.0.8/rosy/trait.Object.html#method.call_with
[`Object::call`]:           https://docs.rs/rosy/0.0.8/rosy/trait.Object.html#method.call
[`Object::call`]:           https://docs.rs/rosy/0.0.8/rosy/trait.Object.html#method.call
[`Object`]:                 https://docs.rs/rosy/0.0.8/rosy/trait.Object.html
[`protected_no_panic`]:     https://docs.rs/rosy/0.0.8/rosy/fn.protected_no_panic.html
[`protected`]:              https://docs.rs/rosy/0.0.8/rosy/fn.protected.html
[`Rosy::mark`]:             https://docs.rs/rosy/0.0.8/rosy/trait.Rosy.html#tymethod.mark
[`vm::destroy`]:            https://docs.rs/rosy/0.0.8/rosy/vm/fn.destroy.html
[`vm::init`]:               https://docs.rs/rosy/0.0.8/rosy/vm/fn.init.html

