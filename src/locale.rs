use std::str::FromStr;
use std::sync::OnceLock;

use fluent::FluentValue;
use fluent_templates::{static_loader, LanguageIdentifier, Loader};
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::LanguageLoader;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Localizations;

pub fn current_loader() -> &'static FluentLanguageLoader {
    static CURRENT_LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();

    #[cfg(test)]
    {
        CURRENT_LOADER.get_or_init(|| {
            let loader: FluentLanguageLoader = fluent_language_loader!();
            loader.select_languages(&["en-US".into()]);
            loader.load_languages(&Localizations, &[loader.fallback_language()])
                .unwrap();
            loader
        });
    }
    #[cfg(not(test))]
    {
        CURRENT_LOADER.get_or_init(|| {
            let loader: FluentLanguageLoader = fluent_language_loader!();
            loader.load_languages(&Localizations, &[loader.fallback_language()])
                .unwrap();
            loader
        });
    }

    CURRENT_LOADER.get()
}


static_loader! {
    pub(crate) static LOCALES = {
        locales: "./resources/locales",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

pub fn current_lang() -> &'static LanguageIdentifier {
    #[cfg(test)]
    {
        static CURRENT_LANG: OnceLock<LanguageIdentifier> = OnceLock::new();
        CURRENT_LANG.get_or_init(|| LanguageIdentifier::from_str("en").expect("No locale found"))
    }
    #[cfg(not(test))]
    {
        static CURRENT_LANG: OnceLock<LanguageIdentifier> = OnceLock::new();
        CURRENT_LANG.get_or_init(|| {
            LanguageIdentifier::from_str(
                sys_locale::get_locale()
                    .unwrap_or("en".to_string())
                    .as_str(),
            )
            .expect("No locale found")
        })
    }
}

/// Look up text identifier in the translation dictionary for the **current system** locale at runtime.
/// Supports passing arguments.
///
/// ### Parameters
/// * `text_id`: Identifier to look up into the `.ftl` translation file.
/// * `args...`: Arbitrary list of tuples ("arg-id", value)
///
/// ### Returns
///
/// Returns the translation corresponding to `text_id` translated with corresponding arguments,
/// otherwise falls back on the english translation.
///
/// ### Example
///
/// ```rust
/// tr!("simple-text-id"); // No arguments
/// tr!("text-id", ("name", value), ("name2", value2)); // With 2 arguments
/// ```
#[macro_export]
macro_rules! tr {
    ($text_id:literal) => {
        $crate::locale::tr_impl($text_id)
    };
    ($text_id:literal, $( ($key:literal, $value:expr) ),* ) => {
        $crate::locale::tr_args_impl($text_id, [ $(($key, $value.into()) ,)* ])
    };
}

pub(crate) use tr;

/// tr macro implementation without arguments
pub fn tr_impl(text_id: &str) -> String {
    LOCALES.lookup(current_lang(), text_id)
}

/// tr macro implementation with argument.
pub fn tr_args_impl<TKeys: AsRef<str> + std::cmp::Eq + std::hash::Hash, const N_ARGS: usize>(
    text_id: &str,
    args: [(TKeys, FluentValue); N_ARGS],
) -> String {
    use std::collections::HashMap;
    LOCALES.lookup_with_args(current_lang(), text_id, &HashMap::from(args))
}

#[test]
fn test_tr_macro() {
    let s = "MyFile".to_owned();
    assert_eq!(
        tr!(
            "error-open-or-parse",
            ("file", s.as_str()),
            ("error", "Test")
        ),
        "Could not open or parse input file \"MyFile\". Reason: Test."
    );
    assert_eq!(tr!("cli-merge-input-left"), "File to receive the merge.");
}
