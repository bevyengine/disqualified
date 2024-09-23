<div class="rustdoc-hidden">

# `disqualified`

</div>

![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![Crates.io](https://img.shields.io/crates/v/disqualified.svg)](https://crates.io/crates/disqualified)
[![Downloads](https://img.shields.io/crates/d/disqualified.svg)](https://crates.io/crates/disqualified)
[![Docs](https://docs.rs/disqualified/badge.svg)](https://docs.rs/disqualified/latest/disqualified/)

Lazily shortens a ["fully qualified"](https://en.wikipedia.org/wiki/Fully_qualified_name) type name to remove all module paths. The short name of a type is its full name as returned by [`core::any::type_name`], but with the prefix of all paths removed. For example, the short name of `alloc::vec::Vec<core::option::Option<u32>>` would be `Vec<Option<u32>>`. Shortening is performed lazily without allocation.

## Contributing

This crate is maintained by the Bevy organization, and is intended to be tiny, stable, zero-dependency, and broadly useful.
[Issues](https://github.com/bevyengine/disqualified/issues) and [pull requests](https://github.com/bevyengine/disqualified/pulls) are genuinely welcome!
