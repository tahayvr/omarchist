use std::io;

use thiserror::Error as ThisError;

/// Result alias for every fallible operation in the `system` and `shell`
/// layers. UI code displays errors with `to_string()`; nothing needs to
/// match on a message string.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    /// A filesystem or process operation failed. `what` says which one,
    /// e.g. "Failed to write colors.toml".
    #[error("{what}: {source}")]
    Io {
        what: String,
        #[source]
        source: io::Error,
    },

    /// A JSON document (theme manifest, settings, hyprctl output) could
    /// not be parsed or serialized.
    #[error("{what}: {source}")]
    Json {
        what: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Theme '{0}' not found")]
    ThemeNotFound(String),

    #[error("Theme '{0}' already exists")]
    ThemeExists(String),

    /// The theme has no `omarchist.json`: Omarchy's own themes, themes
    /// installed from elsewhere, and hand-made folders are never edited.
    #[error("'{0}' was not created with Omarchist, so Omarchist does not edit it")]
    NotOmarchistTheme(String),

    /// A well-known directory (home, themes, backgrounds) could not be
    /// resolved from the environment.
    #[error("Could not determine {0} directory")]
    UnknownDirectory(&'static str),

    /// A request to GitHub for Omarchy release data failed.
    #[error("{0}")]
    Network(String),

    /// Anything else: malformed input, a validation failure, a command that
    /// ran but reported an error.
    #[error("{0}")]
    Invalid(String),
}

impl Error {
    pub fn io(what: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            what: what.into(),
            source,
        }
    }

    pub fn json(what: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            what: what.into(),
            source,
        }
    }
}
