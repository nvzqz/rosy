# Changelog [![crates.io][crate-badge]][crate] [![docs.rs][docs-badge]][docs]
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog] and this project adheres to
[Semantic Versioning].

## [Unreleased]
### Added
- Fast encoding-checking methods to `String` that give `String::to_str` a ~7.5x
  performance improvement when the internal Ruby encoding is UTF-8
  - `encoding_is_ascii_8bit`
  - `encoding_is_utf8`
  - `encoding_is_us_ascii`
- Made some methods on `Encoding` a bit faster:
  - `is_ascii_8bit`
  - `is_utf8`
  - `is_us_ascii`

### Fixed
- `Array::cast` would pass for any objects for `Array<AnyObject>`

### Removed
- Fallback call to `is_ascii_whitespace` in `is_whitespace` on `String`

## 0.0.2 - 2019-05-17
### Added
- `_skip_linking` feature flag to hopefully get https://docs.rs/rosy up

## [0.0.1] - 2019-05-17
Initial release

[crate]:       https://crates.io/crates/rosy
[crate-badge]: https://img.shields.io/crates/v/rosy.svg
[docs]:        https://docs.rs/rosy
[docs-badge]:  https://docs.rs/rosy/badge.svg

[Keep a Changelog]:    http://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: http://semver.org/spec/v2.0.0.html

[Unreleased]: https://github.com/oceanpkg/rosy/compare/v0.0.2...HEAD
[0.0.1]: https://github.com/nvzqz/static-assertions-rs/compare/v0.0.1...v0.0.2
