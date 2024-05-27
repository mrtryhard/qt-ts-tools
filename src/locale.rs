use std::collections::HashMap;
use std::str::FromStr;

use fluent::FluentValue;
use fluent_templates::{static_loader, LanguageIdentifier, Loader};
use lazy_static::lazy_static;

static_loader! {
    pub(crate) static LOCALES = {
        locales: "./resources/locales",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

lazy_static! {
    pub(crate) static ref CURRENT_LANG: LanguageIdentifier = {
        LanguageIdentifier::from_str(
            sys_locale::get_locale()
                .unwrap_or("en".to_string())
                .as_str(),
        )
        .expect("No locale found")
    };
}

/// Look up text identifier in the translation dictionary for the **current system** locale.
///
/// ### Parameters
/// * `text_id`: Identifier to look up into the `.ftl` translation file.
///
/// ### Returns
///
/// Returns the translation corresponding to `text_id` translated, otherwise falls back on the english
/// translation.
///
/// ### Example
///
/// ```rust
/// tr("some-text-id")
/// ```
pub fn tr(text_id: &str) -> String {
    LOCALES.lookup(&CURRENT_LANG, text_id)
}

/// Look up text identifier in the translation dictionary for the **current system** locale.
/// Supports passing arguments.
///
/// ### Parameters
/// * `text_id`: Identifier to look up into the `.ftl` translation file.
/// * `args`: Argument to substitute in the translated string.
///
/// ### Returns
///
/// Returns the translation corresponding to `text_id` translated, otherwise falls back on the english
/// translation.
///
/// ### Example
///
/// ```rust
/// tr_args("some-text-id", [("variablename", value.into())].into())
/// ```
pub fn tr_args<TArgs: AsRef<str>>(text_id: &str, args: HashMap<TArgs, FluentValue>) -> String {
    LOCALES.lookup_with_args(&CURRENT_LANG, text_id, &args)
}
