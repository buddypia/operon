//! Runtime localisation.
//!
//! The design is gettext-shaped: the Japanese copy a call site already carries
//! is the message id, and each other language owns one table keyed by exactly
//! that string. This keeps the source readable, makes Japanese the zero-cost
//! fallback (the identity mapping needs no table), and means adding a language
//! is adding one sorted table plus one `Language` variant — no call site
//! changes. `src/i18n_tables.rs` holds the data; the completeness and drift
//! tests at the bottom of `src/tests.rs` hold code and tables together.
//!
//! Two entry points:
//! - [`tr`] for a plain string;
//! - the [`tf!`](crate::tf) macro for a translated template rendered with
//!   named arguments, replacing `format!`.
//!
//! The active language lives in an atomic so any drawing frame can read it
//! without touching app state; `OperonApp` keeps the persisted copy in sync.

use crate::prelude::*;

use crate::i18n_tables::translation_for;
use crate::run_command_with_timeout;

/// Every language this build ships. The variants are persisted in the store,
/// so a variant may never be renamed or removed — only added.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Language {
    En,
    #[default]
    Ja,
    Ko,
}

impl Language {
    /// The name shown in the picker, written in the language itself so a
    /// person who cannot read the current UI can still find their own.
    pub(crate) fn native_label(self) -> &'static str {
        match self {
            // Never through `tr`: this is the one label that must not follow
            // the active language, or the person who cannot read the current
            // UI loses the only word on screen they were looking for.
            Self::En => "English",
            Self::Ja => "日本語",
            Self::Ko => "한국어",
        }
    }

    /// Picker detail line, resolved through the catalog like any other copy.
    pub(crate) fn detail(self) -> &'static str {
        match self {
            Self::En => tr("英語（English）"),
            Self::Ja => tr("日本語"),
            Self::Ko => tr("韓国語（한국어）"),
        }
    }

    pub(crate) fn all() -> [Self; 3] {
        [Self::En, Self::Ja, Self::Ko]
    }

    fn from_tag(tag: &str) -> Option<Self> {
        let lowered = tag.to_ascii_lowercase();
        let prefix = lowered.split(['-', '_', '.']).next().unwrap_or("");
        match prefix {
            "en" => Some(Self::En),
            "ja" | "jp" => Some(Self::Ja),
            "ko" | "kr" => Some(Self::Ko),
            _ => None,
        }
    }

    fn to_atomic(self) -> u8 {
        match self {
            Self::En => 0,
            Self::Ja => 1,
            Self::Ko => 2,
        }
    }

    fn from_atomic(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::En),
            1 => Some(Self::Ja),
            2 => Some(Self::Ko),
            _ => None,
        }
    }
}

/// What a fresh install opens with, before any preference exists. Deliberately
/// *not* wired into `Default`: serde fills absent store fields through
/// `Default`, and every test that parses a store would end up spawning
/// `defaults`. Detection runs exactly once, from `OperonApp` construction,
/// where it can tell a brand-new install from a migrated one.
pub(crate) fn detected_language() -> Language {
    detect_system_language()
}

/// The language every lookup reads. An atomic rather than app state, because
/// strings are drawn from deep inside helpers that have no access to
/// `OperonApp`, and because a switch has to land on the very next frame.
static ACTIVE_LANGUAGE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(1);

pub(crate) fn set_active_language(language: Language) {
    ACTIVE_LANGUAGE.store(language.to_atomic(), Ordering::Relaxed);
}

pub(crate) fn active_language() -> Language {
    Language::from_atomic(ACTIVE_LANGUAGE.load(Ordering::Relaxed)).unwrap_or(Language::Ja)
}

/// Translate a message id into the active language.
///
/// The message id is the Japanese source copy itself. Unknown ids come back
/// unchanged — a table that lags behind the code degrades to Japanese instead
/// of panicking, and the drift test keeps that from going unnoticed.
pub(crate) fn tr(message: &'static str) -> &'static str {
    match active_language() {
        Language::Ja => message,
        language => translation_for(language, message).unwrap_or(message),
    }
}

/// One named argument of [`tf`], already rendered.
///
/// The value arrives as a `String` rather than a `&dyn Display` because the
/// macro's arguments are overwhelmingly temporaries — `error`, `count + 1`, a
/// `label()` return — and a borrow of one cannot outlive the statement that
/// builds the argument list. Formatting at the call site costs one small
/// allocation per placeholder and `render` was calling `to_string` anyway.
pub(crate) type TemplateArg<'a> = (&'a str, String);

/// Render a translated template, filling `{name}` placeholders from `args`.
///
/// The grammar mirrors `format!`'s for the subset this app uses: `{{` and
/// `}}` escape a literal brace, `{name}` names an argument. A placeholder with
/// no matching argument is copied through verbatim so a template/argument
/// mismatch is visible on screen rather than silently dropped.
pub(crate) fn render(template: &str, args: &[TemplateArg]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}') {
            let name = &after[..close];
            if name.is_empty() {
                // `{{` was already handled by finding the next brace as text,
                // so an empty placeholder is a bare `{}`: no such argument is
                // nameable, so keep it visible.
                out.push_str("{}");
            } else if let Some(value) = args.iter().find(|(key, _)| *key == name) {
                out.push_str(&value.1);
            } else {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
            rest = &after[close + 1..];
        } else {
            out.push_str(&rest[open..]);
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

/// Best effort language detection for a first launch, before any store
/// exists. macOS graphical apps rarely inherit `LANG`, so the user's own
/// preference list is read through `defaults`; env vars remain the first
/// check for launches from a shell. Anything unrecognised falls back to
/// Japanese, the language every string in the source is written in.
fn detect_system_language() -> Language {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(tag) = std::env::var_os(key)
            .and_then(|value| value.into_string().ok())
            .map(|value| value.split('.').next().unwrap_or("").to_owned())
            .filter(|value| !value.is_empty())
        {
            if let Some(language) = Language::from_tag(&tag) {
                return language;
            }
        }
    }
    // `defaults` is an outside process, so it runs under the same timeout
    // budget as every other child: a hanging lookup must fall back to
    // Japanese, never hang first launch.
    if let Ok(output) = run_command_with_timeout(
        Command::new("defaults").args(["read", "-g", "AppleLanguages"]),
        Duration::from_secs(5),
    ) {
        if output.status.success() {
            let listing = String::from_utf8_lossy(&output.stdout);
            if let Some(language) = preferred_language_from_defaults(&listing) {
                return language;
            }
        }
    }
    if let Ok(output) = run_command_with_timeout(
        Command::new("defaults").args(["read", "-g", "AppleLocale"]),
        Duration::from_secs(5),
    ) {
        if output.status.success() {
            let locale = String::from_utf8_lossy(&output.stdout);
            if let Some(language) = Language::from_tag(locale.trim()) {
                return language;
            }
        }
    }
    Language::Ja
}

/// `( "ja-JP",\n    en-US,\n)` is what `defaults read -g AppleLanguages`
/// prints: quoted or bare tags, one per line, best first.
fn preferred_language_from_defaults(listing: &str) -> Option<Language> {
    listing
        .lines()
        .map(|line| {
            line.trim()
                .trim_matches(|c| c == '(' || c == ')' || c == ',' || c == '"')
        })
        .filter_map(|entry| {
            let trimmed = entry.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .find_map(Language::from_tag)
}

/// Translated-template `format!`: the template is looked up in the active
/// language first, then rendered with named arguments.
///
/// ```ignore
/// tf!("{} を保存しました。", name = file_name)
/// ```
///
/// Placeholders are always named — including ones `format!` would have
/// numbered — because translations reorder sentences freely, and a positional
/// hole means nothing to a translator working from the Japanese alone. The
/// completeness test extracts every template here and checks that its
/// placeholder names survive into both other tables.
#[macro_export]
macro_rules! tf {
    ($template:expr $(,)?) => {
        $crate::i18n::tr($template).to_owned()
    };
    ($template:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let __tf_template: &'static str = $crate::i18n::tr($template);
        // Each value is formatted here rather than borrowed into the list.
        // Nearly every call site passes a temporary — `error`, `count + 1`, a
        // `label()` return — and a reference to one dies at the end of the
        // statement that builds the list. Binding them to locals first is not
        // an option either: the argument names are the caller's identifiers,
        // and a screaming-case one like `ICON_ADD` is a *constant pattern* in
        // `let` position, which binds nothing.
        let __tf_args: ::std::vec::Vec<$crate::i18n::TemplateArg> =
            ::std::vec![ $((
                stringify!($name),
                ::std::string::ToString::to_string(&$value),
            )),+ ];
        $crate::i18n::render(__tf_template, &__tf_args)
    }};
}
