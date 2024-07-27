use std::str::FromStr;
use std::sync::OnceLock;

use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::unic_langid::LanguageIdentifier;
use i18n_embed::{DefaultLocalizer, LanguageLoader, Localizer};
use log::debug;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "resources/locales/"]
struct Localizations;

fn current_lang() -> LanguageIdentifier {
    LanguageIdentifier::from_str(
        sys_locale::get_locale()
            .unwrap_or("en".to_string())
            .as_str(),
    )
    .expect("No locale found")
}

pub fn current_loader() -> &'static FluentLanguageLoader {
    static CURRENT_LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();

    CURRENT_LOADER.get_or_init(|| {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        loader
            .load_fallback_language(&Localizations)
            .expect("Expected to have fallback language");
        // Required for tests
        loader.set_use_isolating(false);
        loader
    })
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
/// tr!("text-id", name = value, name2 = value2); // With 2 arguments
/// ```
#[macro_export]
macro_rules! tr {
    ($text_id:literal) => {{
        i18n_embed_fl::fl!($crate::locale::current_loader(), $text_id)
    }};
    ($text_id:literal, $( ($key:literal, $value:expr) ),* ) => {{
        let mut args: std::collections::HashMap<&str, fluent_bundle::FluentValue> = std::collections::HashMap::new();
        $(args.insert($key, $value.into());)*
        i18n_embed_fl::fl!($crate::locale::current_loader(), $text_id, args)
    }};
}

pub(crate) use tr;

/// Initializes the locale for the application by selecting the system's current locale.
pub fn initialize_locale() -> Box<dyn Localizer> {
    let localizer = Box::from(DefaultLocalizer::new(current_loader(), &Localizations));
    let requested_languages = current_lang();

    debug!("Using languages {:?}", &requested_languages);

    if let Err(error) = localizer.select(&[requested_languages]) {
        eprintln!("Error while loading languages for library_fluent {error}");
    }

    // Required for the application
    current_loader().set_use_isolating(false);

    localizer
}

#[test]
fn test_tr_macro() {
    let s = "MyFile".to_owned();
    assert_eq!(
        tr!("error-open-or-parse", ("file", s), ("error", "Test")),
        "Could not open or parse input file \"MyFile\". Reason: Test."
    );
    assert_eq!(tr!("cli-merge-input-left"), "File to receive the merge.");
}
