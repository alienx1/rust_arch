//! Language of the request being served.
//!
//! The API middleware reads `Accept-Language` and runs the rest of the request inside
//! [`scope`]; query code that turns localized labels into a single display name calls
//! [`pick`] / [`current`] instead of always taking the Thai text. Outside a scope (background
//! jobs, tests) the language is Thai.

use std::future::Future;

tokio::task_local! {
    static REQUEST_LANGUAGE: Language;
}

/// UI languages with seeded labels
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    Thai,
    English,
}

impl Language {
    /// First tag of an `Accept-Language` value (`en-US,en;q=0.9` -> English); Thai otherwise
    pub fn from_accept_language(header: &str) -> Self {
        let primary = header
            .split(',')
            .next()
            .and_then(|tag| tag.split(';').next())
            .and_then(|tag| tag.trim().split('-').next())
            .unwrap_or_default();
        if primary.eq_ignore_ascii_case("en") { Self::English } else { Self::Thai }
    }
}

/// Runs `future` with `language` as the language of the request
pub async fn scope<F: Future>(language: Language, future: F) -> F::Output {
    REQUEST_LANGUAGE.scope(language, future).await
}

/// Language of the request being served (Thai outside a request)
pub fn current() -> Language {
    REQUEST_LANGUAGE.try_with(|l| *l).unwrap_or_default()
}

/// The text of the request's language, falling back to the other one when it is empty
pub fn pick<'a>(thai: &'a str, english: &'a str) -> &'a str {
    pick_for(current(), thai, english)
}

/// [`pick`] for an explicit language
pub fn pick_for<'a>(language: Language, thai: &'a str, english: &'a str) -> &'a str {
    let (first, second) = match language {
        Language::Thai => (thai, english),
        Language::English => (english, thai),
    };
    if first.is_empty() { second } else { first }
}
