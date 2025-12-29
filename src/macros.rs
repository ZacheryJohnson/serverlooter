use std::collections::HashMap;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, LanguageIdentifier};

static_loader! {
    static LOCALES = {
        locales: "locales",
        fallback_language: "en-US",
        customise: |bundle| {
            bundle.add_builtins().expect("failed to add builtins to Fluent localization bundle");
        }
    };
}

#[macro_export]
/// Locks an `Arc<Mutex<T>>` and returns `<T>::clone()`.
macro_rules! lock_and_clone {
    ($arc_mutex:expr, $value_to_clone:ident) => {
        $arc_mutex.lock().unwrap().$value_to_clone.clone()
    };
    ($arc_mutex:expr, $inner_arc_mutex:ident, $value_to_clone:ident) => {
        lock_and_clone!(lock_and_clone!($arc_mutex, $inner_arc_mutex), $value_to_clone)
    };
}

#[macro_export]
macro_rules! loc {
    ($save:ident, $key:literal) => {
        get_localized(&$save.language_identifier, $key, None)
    };
    ($save:ident, $key:literal, $args:expr) => {
        get_localized(&$save.language_identifier, $key, Some(&$args))
    };
}

#[inline]
pub fn get_localized(
    language_identifier: &LanguageIdentifier,
    lookup_key: &str,
    args: Option<&HashMap<String, FluentValue>>
) -> String {
    LOCALES.lookup_single_language(language_identifier, lookup_key, args).unwrap_or_default()
}