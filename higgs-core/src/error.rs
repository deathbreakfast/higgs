/// Errors from Higgs context operations.
#[derive(Debug, thiserror::Error)]
pub enum HiggsError {
    /// No `Arc<HiggsConfig>` was found in Leptos context; host startup must call
    /// `provide_context(Arc<HiggsConfig>)` before server functions run.
    #[error(
        "HiggsConfig not found in Leptos context — \
         was provide_context(Arc<HiggsConfig>) called?"
    )]
    ConfigNotInContext,

    /// The named optional subsystem (e.g. `chronon`, `boson`, `photon`) was not configured
    /// on [`HiggsConfig`](crate::HiggsConfig) at build time.
    #[error("subsystem `{0}` was not configured in HiggsConfig")]
    SubsystemNotConfigured(&'static str),

    /// Internal failure. Display is intentionally opaque for client-facing paths;
    /// details are logged when constructed via [`HiggsError::internal`].
    #[error("internal higgs error")]
    Internal,
}

impl HiggsError {
    /// Build an opaque [`Self::Internal`] and log `detail` at error level (server-side only).
    pub fn internal(detail: impl std::fmt::Display) -> Self {
        log::error!(target: "higgs", "internal higgs error: {detail}");
        Self::Internal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_error_display_opaque_happy_path() {
        let err = HiggsError::internal("factory boom: secret=xyz");
        let display = err.to_string();
        assert_eq!(display, "internal higgs error");
        assert!(!display.contains("secret"));
        assert!(!display.contains("factory boom"));
    }
}
