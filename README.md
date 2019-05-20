# Rosy

[![Build status][travis-badge]][travis]
![Lines of code][loc-badge]
[![crates.io][crate-badge] ![downloads][dl-badge]][crate]
[![docs.rs][docs-badge]][docs]
[![MIT or Apache 2.0][license-badge]][license]

High-level bindings of [Ruby]'s C API for [Rust].

## Installation

This crate is available [on crates.io][crate] and can be used by adding the
following to your project's [`Cargo.toml`]:

```toml
[dependencies]
rosy = "0.0.5"
```

Rosy has functionality that is only available for certain Ruby versions. The
following features can currently be enabled:

- `ruby_2_6`

For example:

```toml
[dependencies.rosy]
version = "0.0.5"
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

// The VM should be initialized before doing anything
rosy::vm::init().expect("Could not initialize Ruby");

let string = String::from("hello\r\n");
string.call("chomp!").unwrap();

assert_eq!(string, "hello");
```

## License

This project is made available under either the conditions of the
[MIT License](https://choosealicense.com/licenses/mit/) or
[Apache License 2.0](https://choosealicense.com/licenses/apache-2.0/)
at your choosing.

See [`LICENSE.md`][license].

[Ruby]:          https://www.ruby-lang.org
[Rust]:          https://www.rust-lang.org
[`Cargo.toml`]:  https://doc.rust-lang.org/cargo/reference/manifest.html

[travis]:        https://travis-ci.com/oceanpkg/rosy
[travis-badge]:  https://travis-ci.com/oceanpkg/rosy.svg?branch=master
[loc-badge]:     https://tokei.rs/b1/github/oceanpkg/rosy?category=code
[crate]:         https://crates.io/crates/rosy
[crate-badge]:   https://img.shields.io/crates/v/rosy.svg
[dl-badge]:      https://img.shields.io/crates/d/rosy.svg
[docs]:          https://docs.rs/rosy
[docs-badge]:    https://docs.rs/rosy/badge.svg
[license]:       https://github.com/oceanpkg/rosy/blob/master/LICENSE.md
[license-badge]: https://img.shields.io/badge/license-MIT%20or%20Apache%202.0-blue.svg
