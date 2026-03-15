use std::collections::HashMap;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{static_loader, LanguageIdentifier};
use crate::l10n::message_id::MessageId;

pub mod message_id;

static_loader! {
    static LOCALES = {
        locales: "locales",
        fallback_language: "en-US",
        customise: |bundle| {
            bundle
                .add_builtins()
                .expect("failed to add builtins to Fluent localization bundle");
        }
    };
}

pub trait Localizable {
    fn loc_key(&self) -> MessageId;
    fn loc_args(&self) -> HashMap<&'static str, FluentValue<'_>> {
        HashMap::new()
    }
}

pub fn get_localized(
    language_identifier: &LanguageIdentifier,
    lookup_key: MessageId,
) -> String {
    LOCALES
        .lookup_single_language::<&'static str>(language_identifier, lookup_key.get(), None)
        .expect(&format!("failed to load key {}", lookup_key.get()))
}

pub fn get_localized_with_args(
    language_identifier: &LanguageIdentifier,
    lookup_key: MessageId,
    args: HashMap<&'static str, FluentValue>
) -> String {
    LOCALES
        .lookup_single_language(language_identifier, lookup_key.get(), Some(&args))
        .expect(&format!("failed to load key {}", lookup_key.get()))
}

#[macro_export]
macro_rules! loc {
    ($save:ident, $key:literal) => {
        crate::l10n::get_localized(&$save.language_identifier, $key)
    };
    ($save:ident, $path:expr) => {
        crate::l10n::get_localized(&$save.language_identifier, $path)
    };
    ($save:ident, $key:literal, $args:expr) => {
        crate::l10n::get_localized_with_args(&$save.language_identifier, $key, $args)
    };
    ($save:ident, $path:expr, $args:expr) => {
        crate::l10n::get_localized_with_args(&$save.language_identifier, $path, $args)
    };
}
