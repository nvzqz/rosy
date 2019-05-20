# Changelog [![crates.io][crate-badge]][crate] [![docs.rs][docs-badge]][docs]
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog] and this project adheres to
[Semantic Versioning].

## [Unreleased]
### Added
- Variants of `Class::new_instance` that are `unsafe` or take arguments
- `PartialEq<[A]>` implementation for `Array<O>` where `O: PartialEq<A>`
- `Partial{Eq|Cmp}` implementation over integers for `AnyObject`

### Fixed
- Safety of `Class::new_instance`

## [0.0.4] - 2019-05-19
### Added
- `Integer` object type
  - Features `From` conversions for every native Rust integer type, including
    `u128` and `i128`
  - Supports logical bitwise operations
  - Methods:
    - `pack` and `unpack` for converting to and from words respectively
    - `to_truncated` for converting similarly to `as` with primitives
    - `to_value` for converting similarly to `TryFrom` on primitives
      - Has `can_represent` helper method
- [`Debug`] requirement for `Object` trait

## [0.0.3] - 2019-05-18
### Added
- Typed keys and values for `Hash`
- Fast encoding-checking methods to `String` that give `String::to_str` a ~7.5x
  performance improvement when the internal Ruby encoding is UTF-8
  - `encoding_is_ascii_8bit`
  - `encoding_is_utf8`
  - `encoding_is_us_ascii`
- Made some methods on `Encoding` a bit faster:
  - `is_ascii_8bit`
  - `is_utf8`
  - `is_us_ascii`
- Unsafe `protected_no_panic` variant for when the argument is guaranteed by the
  caller to not panic

### Fixed
- `Array::cast` would pass for any objects for `Array<AnyObject>`
- `protected` is now panic-safe via [`std::panic::catch_unwind`]

### Removed
- Fallback call to `is_ascii_whitespace` in `is_whitespace` on `String`

## [0.0.2] - 2019-05-17
### Added
- `_skip_linking` feature flag to hopefully get https://docs.rs/rosy up

## 0.0.1 - 2019-05-17
Initial release

[crate]:       https://crates.io/crates/rosy
[crate-badge]: https://img.shields.io/crates/v/rosy.svg
[docs]:        https://docs.rs/rosy
[docs-badge]:  https://docs.rs/rosy/badge.svg

[Keep a Changelog]:    http://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: http://semver.org/spec/v2.0.0.html

[Unreleased]: https://github.com/oceanpkg/rosy/compare/v0.0.4...HEAD
[0.0.4]: https://github.com/oceanpkg/rosy/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/oceanpkg/rosy/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/oceanpkg/rosy/compare/v0.0.1...v0.0.2

[`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
[`std::panic::catch_unwind`]: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
