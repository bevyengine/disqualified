use std::{borrow::Cow, fmt, str};

const SPECIAL_TYPE_CHARS: [u8; 9] = *b" <>()[],;";

/// Lazily shortens a type name to remove all module paths.
///
/// The short name of a type is its full name as returned by
/// [`core::any::type_name`], but with the prefix of all paths removed. For
/// example, the short name of `alloc::vec::Vec<core::option::Option<u32>>`
/// would be `Vec<Option<u32>>`.
///
/// Shortening is performed lazily without allocation.
#[cfg_attr(
    feature = "alloc",
    doc = r#" To get a [`String`] from this type, use the [`to_string`](`alloc::string::ToString::to_string`) method."#
)]
///
/// # Examples
///
/// ```rust
/// # use disqualified::ShortName;
/// #
/// # mod foo {
/// #     pub mod bar {
/// #         pub struct Baz;
/// #     }
/// # }
/// let short_name = ShortName::of::<foo::bar::Baz>(); // Baz
/// ```
#[derive(Clone, Copy)]
pub struct ShortName<'a>(pub &'a str);

impl ShortName<'static> {
    /// Gets a shortened version of the name of the type `T`.
    pub fn of<T: ?Sized>() -> Self {
        Self(core::any::type_name::<T>())
    }
}

impl<'a> ShortName<'a> {
    /// Gets the original name before shortening.
    pub const fn original(&self) -> &'a str {
        self.0
    }
}

impl<'a> From<&'a str> for ShortName<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

impl<'a> fmt::Debug for ShortName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut remaining = f.as_bytes();
        let mut parsed_name = Vec::new();
        let mut complex_type = false;

        loop {
            // Collapse everything up to the next special character,
            // then skip over it
            let is_special = |c| SPECIAL_TYPE_CHARS.contains(c);
            if let Some(next_special_index) = remaining.iter().position(is_special) {
                complex_type = true;
                if parsed_name.is_empty() {
                    parsed_name.reserve(remaining.len());
                }
                let (pre_special, post_special) = remaining.split_at(next_special_index + 1);
                parsed_name.extend_from_slice(collapse_type_name(pre_special));
                match pre_special.last().unwrap() {
                    b'>' | b')' | b']' if post_special.get(..2) == Some(b"::") => {
                        parsed_name.extend_from_slice(b"::");
                        // Move the index past the "::"
                        remaining = &post_special[2..];
                    }
                    // Move the index just past the special character
                    _ => remaining = post_special,
                }
            } else if !complex_type {
                let collapsed = collapse_type_name(remaining);
                // SAFETY: We only split on ASCII characters, and the input is valid UTF8, since
                // it was a &str
                let str = unsafe { str::from_utf8_unchecked(collapsed) };
                return Cow::Borrowed(str);
            } else {
                // If there are no special characters left, we're done!
                parsed_name.extend_from_slice(collapse_type_name(remaining));
                // SAFETY: see above
                let utf8_name = unsafe { String::from_utf8_unchecked(parsed_name) };
                return Cow::Owned(utf8_name);
            }
        }
    }
}

impl<'a> fmt::Display for ShortName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

/// Wrapper around `AsRef<str>` that uses the [`get_short_name`] format when
/// displayed.
pub struct DisplayShortName<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> fmt::Display for DisplayShortName<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let as_short_name = ShortName(self.0.as_ref());
        write!(f, "{as_short_name}")
    }
}

#[inline(always)]
fn collapse_type_name(string: &[u8]) -> &[u8] {
    let find = |(index, window)| (window == b"::").then_some(index + 2);
    let split_index = string.windows(2).enumerate().rev().find_map(find);
    &string[split_index.unwrap_or(0)..]
}

#[cfg(all(test, feature = "alloc"))]
mod name_formatting_tests {
    use super::ShortName;

    #[test]
    fn trivial() {
        assert_eq!(ShortName("test_system").to_string(), "test_system");
    }

    #[test]
    fn path_separated() {
        assert_eq!(
            ShortName("bevy_prelude::make_fun_game").to_string(),
            "make_fun_game"
        );
    }

    #[test]
    fn tuple_type() {
        assert_eq!(
            ShortName("(String, String)").to_string(),
            "(String, String)"
        );
    }

    #[test]
    fn array_type() {
        assert_eq!(ShortName("[i32; 3]").to_string(), "[i32; 3]");
    }

    #[test]
    fn trivial_generics() {
        assert_eq!(ShortName("a<B>").to_string(), "a<B>");
    }

    #[test]
    fn multiple_type_parameters() {
        assert_eq!(ShortName("a<B, C>").to_string(), "a<B, C>");
    }

    #[test]
    fn enums() {
        assert_eq!(ShortName("Option::None").to_string(), "Option::None");
        assert_eq!(ShortName("Option::Some(2)").to_string(), "Option::Some(2)");
        assert_eq!(
            ShortName("bevy_render::RenderSet::Prepare").to_string(),
            "RenderSet::Prepare"
        );
    }

    #[test]
    fn generics() {
        assert_eq!(
            ShortName("bevy_render::camera::camera::extract_cameras<bevy_render::camera::bundle::Camera3d>").to_string(),
            "extract_cameras<Camera3d>"
        );
    }

    #[test]
    fn utf8_generics() {
        assert_eq!(
            fmt("bévï::camérą::łørđ::_öñîòñ<ķràźÿ::Москва::東京>"),
            "_öñîòñ<東京>".to_string()
        );
    }

    #[test]
    fn nested_generics() {
        assert_eq!(
            ShortName("bevy::mad_science::do_mad_science<mad_science::Test<mad_science::Tube>, bavy::TypeSystemAbuse>").to_string(),
            "do_mad_science<Test<Tube>, TypeSystemAbuse>"
        );
    }

    #[test]
    fn sub_path_after_closing_bracket() {
        assert_eq!(
            ShortName("bevy_asset::assets::Assets<bevy_scene::dynamic_scene::DynamicScene>::asset_event_system").to_string(),
            "Assets<DynamicScene>::asset_event_system"
        );
        assert_eq!(
            ShortName("(String, String)::default").to_string(),
            "(String, String)::default"
        );
        assert_eq!(
            ShortName("[i32; 16]::default").to_string(),
            "[i32; 16]::default"
        );
    }
}
