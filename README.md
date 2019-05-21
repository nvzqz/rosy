# Rosy

[![Build status][travis-badge]][travis]
![Lines of code][loc-badge]
[![crates.io][crate-badge] ![downloads][dl-badge]][crate]
[![docs.rs][docs-badge]][docs]
[![MIT or Apache 2.0][license-badge]][license]

High-level, zero (or low) cost bindings of [Ruby]'s C API for [Rust].

## Index
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Defining Ruby Methods](#defining-ruby-methods)
  - [Defining Ruby Classes](#defining-ruby-classes)
  - [Defining Ruby Subclasses](#defining-ruby-subclasses)
  - [Catching Ruby Exceptions](#catching-ruby-exceptions)
- [Library Comparison](#library-comparison)
  - [Rosy vs Helix](#rosy-vs-helix)
  - [Rosy vs Rutie](#rosy-vs-rutie)
- [Authors](#authors)
- [License](#license)

## Features

- Zero or very-low cost abstractions over Ruby's C API.

  If speed is of the utmost importance, Rosy has functionality that cannot be
  beaten performance-wise when using the C API directly. However, this may
  require carefully writing some `unsafe` code.

  - For example, [`Object::call`] will catch any raised Ruby exceptions via
    the [`protected`] family of functions. On the other hand,
    [`Object::call_unchecked`] will allow any thrown exception propagate
    (which causes a segmentation fault in Rust-land) unless [`protected`].

    Checking for exceptions via [`protected`] has a cost associated
    with it and so it may be best to wrap multiple instances of
    exception-throwing code with it rather than just one.

  - If it is known that no [`panic!`] will occur anywhere within
    exception-checked code, then calling [`protected_no_panic`] will emit
    fewer instructions at the cost of safety. The [`FnOnce`] passed into this
    function is called within an FFI context, and [panicking here is undefined
    behavior][panic-ffi-ub]. Panics in a normal [`protected`] call are safely
    caught with the stack unwinding properly.

- Bindings that leverage Rust's type system to the fullest:

  - Rosy makes certain Ruby types generic over enclosing types:

    - [`Array`] is generic over [`Object`] types that it contains, defaulting to
      [`AnyObject`].

    - [`Hash`] is generic over [`Object`] keys and values, both defaulting to
      [`AnyObject`].

    - [`Class`] is generic over an [`Object`] type that it may instantiate via
      [`Class::new_instance`].

  - When defining methods via [`Class::def_method`] or [`def_method!`]:

    - The receiver is statically typed as the generic [`Object`] type that the
      [`Class`] is meant for.

    - Arguments (excluding the receiver) are generic up to and including 15
      [`AnyObject`]s. It may also take either an [`Array`] or an [`AnyObject`]
      pointer paired with a length. These allow for passing in a variable number
      of arguments.

- Safety wherever possible.

  Unfortunately, due to the inherent nature of Ruby's C API, this isn't easily
  achievable without a few compromises in performance. A few factors that cause
  this are:

  - Ruby's garbage collector de-allocating objects whose references don't live
    on the stack if they are not [`mark`]ed. This may lead to a possible
    [use after free].

  - _Many_ Ruby functions can throw exceptions. 😓

    These cause a segmentation fault in Rust code that's not being called
    originally from a Ruby context. Functions that may throw an exception are
    marked as `unsafe` or have a safe exception-checking equivalent.

## Installation

This crate is available [on crates.io][crate] and can be used by adding the
following to your project's [`Cargo.toml`]:

```toml
[dependencies]
rosy = "0.0.6"
```

Rosy has functionality that is only available for certain Ruby versions. The
following features can currently be enabled:

- `ruby_2_6`

For example:

```toml
[dependencies.rosy]
version = "0.0.6"
features = ["ruby_2_6"]
```

Finally add this to your crate root (`main.rs` or `lib.rs`):

```rust
extern crate rosy;
```

## Usage

Rosy allows you to perform _many_ operations over Ruby objects in a way that
feels very natural in Rust.

```rust
use rosy::String;

// The VM must be initialized before doing anything
rosy::vm::init().expect("Could not initialize Ruby");

let string = String::from("hello\r\n");
string.call("chomp!").unwrap();

assert_eq!(string, "hello");
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
[built-in class](https://docs.rs/rosy/0.0.6/rosy/struct.Class.html#impl-1),
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

### Library Comparison

Like with most technologies, Rosy isn't the first of its kind.

#### Rosy vs Helix

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

#### Rosy vs Rutie

[Rutie] is a Rust library that tries to be less magical than Helix. It takes an
excellent approach to wrapping Ruby's C API in Rust by exposing Ruby classes as
Rust `struct`s. This inspired the layout and design of Rosy to some extent.
Rutie is actually a continuation of the work done on [ruru], which is no longer
maintained as of the end of 2017. Rutie even exports its low-level C bindings in
a module that can be used to write functionality that Rutie have.

Unlike Rutie, Rosy doesn't expose the lower-level C library. The reasoning is
that if certain functionality is missing from Rosy, it should be added to the
core library by either requesting it through [an issue][issues] or submitting a
[pull request][pulls] with an implementation.

Rosy is also designed to enable you to write the most performant code possible
such that it can't be beaten by using the C API directly. This aligns with the
notion of
[zero-cost abstractions](https://boats.gitlab.io/blog/post/zero-cost-abstractions).
However, not all of Rosy's _safe_ abstractions are zero-cost. Sometimes this is
only possible by writing `unsafe` code since Rosy can't be made aware of certain
aspects of the program state. For example, unlike Rutie, all functionality in
Rosy that may throw an exception is marked as `unsafe`. Just like with Rutie,
however, Rosy allows Rust code to be [`protected`] against raised exceptions.
This, in combination with calling unchecked functions, allows for reducing the
number of speed bumps in your code.

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

[Back to top](#top)

[Ruby]:           https://www.ruby-lang.org
[Rust]:           https://www.rust-lang.org
[`Cargo.toml`]:   https://doc.rust-lang.org/cargo/reference/manifest.html
[Helix]:          https://usehelix.com
[Rutie]:          https://github.com/danielpclark/rutie
[ruru]:           https://github.com/d-unseductable/ruru

[DSL]:            https://en.wikipedia.org/wiki/Domain-specific_language
[panic-ffi-ub]:   https://doc.rust-lang.org/nomicon/ffi.html#ffi-and-panics
[use after free]: https://cwe.mitre.org/data/definitions/416.html
[UTF-8]:          https://en.wikipedia.org/wiki/UTF-8

[issues]:         https://github.com/oceanpkg/rosy/issues
[pulls]:          https://github.com/oceanpkg/rosy/pulls
[travis]:         https://travis-ci.com/oceanpkg/rosy
[travis-badge]:   https://travis-ci.com/oceanpkg/rosy.svg?branch=master
[loc-badge]:      https://tokei.rs/b1/github/oceanpkg/rosy?category=code
[crate]:          https://crates.io/crates/rosy
[crate-badge]:    https://img.shields.io/crates/v/rosy.svg
[dl-badge]:       https://img.shields.io/crates/d/rosy.svg
[docs]:           https://docs.rs/rosy
[docs-badge]:     https://docs.rs/rosy/badge.svg
[license]:        https://github.com/oceanpkg/rosy/blob/master/LICENSE.md
[license-badge]:  https://img.shields.io/badge/license-MIT%20or%20Apache%202.0-blue.svg

[`FnOnce`]: https://doc.rust-lang.org/std/ops/trait.FnOnce.html
[`panic!`]: https://doc.rust-lang.org/stable/std/macro.panic.html

[`AnyObject`]:              https://docs.rs/rosy/0.0.6/rosy/struct.AnyObject.html
[`Array`]:                  https://docs.rs/rosy/0.0.6/rosy/struct.Array.html
[`Class::def_method`]:      https://docs.rs/rosy/0.0.6/rosy/struct.Class.html#method.def_method
[`Class::get_or_def`]:      https://docs.rs/rosy/0.0.6/rosy/struct.Class.html#method.get_or_def
[`Class::get`]:             https://docs.rs/rosy/0.0.6/rosy/struct.Class.html#method.get
[`Class::new_instance`]:    https://docs.rs/rosy/0.0.6/rosy/struct.Class.html#method.new_instances
[`Class::subclass`]:        https://docs.rs/rosy/0.0.6/rosy/struct.Class.html#method.subclass
[`Class`]:                  https://docs.rs/rosy/0.0.6/rosy/struct.Class.html
[`def_method_unchecked!`]:  https://docs.rs/rosy/0.0.6/rosy/macro.def_method_unchecked.html
[`def_method!`]:            https://docs.rs/rosy/0.0.6/rosy/macro.def_method.html
[`Hash`]:                   https://docs.rs/rosy/0.0.6/rosy/struct.Hash.html
[`mark`]:                   https://docs.rs/rosy/0.0.6/rosy/gc/fn.mark.html
[`Mixin::def_class`]:       https://docs.rs/rosy/0.0.6/rosy/trait.Mixin.html#method.def_class
[`Mixin::def_subclass`]:    https://docs.rs/rosy/0.0.6/rosy/trait.Mixin.html#method.def_subclass
[`Object::call_unchecked`]: https://docs.rs/rosy/0.0.6/rosy/trait.Object.html#method.call_unchecked
[`Object::call`]:           https://docs.rs/rosy/0.0.6/rosy/trait.Object.html#method.call
[`Object`]:                 https://docs.rs/rosy/0.0.6/rosy/trait.Object.html
[`protected`]:              https://docs.rs/rosy/0.0.6/rosy/fn.protected.html
[`protected_no_panic`]:     https://docs.rs/rosy/0.0.6/rosy/fn.protected_no_panic.html
[`vm::init`]:               https://docs.rs/rosy/0.0.6/rosy/vm/fn.init.html
