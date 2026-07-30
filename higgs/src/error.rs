//! Errors from platform Higgs config / context operations.

/// Errors from Higgs context operations.
#[derive(Debug, thiserror::Error)]
pub enum HiggsError {
    /// No `Arc<HiggsConfig>` was found in Leptos context.
    #[error(
        "HiggsConfig not found in Leptos context — \
         was provide_context(Arc<HiggsConfig>) called?"
    )]
    ConfigNotInContext,

    /// The named optional subsystem was not configured at build time.
    #[error("subsystem `{0}` was not configured in HiggsConfig")]
    SubsystemNotConfigured(&'static str),

    /// Internal failure. Display is opaque; construct via [`HiggsError::internal`].
    #[error("internal higgs error")]
    Internal,
}

impl HiggsError {
    /// Build an opaque [`Self::Internal`] and log `detail` server-side.
    pub fn internal(detail: impl std::fmt::Display) -> Self {
        log::error!(target: "higgs", "internal higgs error: {detail}");
        Self::Internal
    }
}

impl From<higgs_core::HiggsError> for HiggsError {
    fn from(value: higgs_core::HiggsError) -> Self {
        match value {
            higgs_core::HiggsError::ConfigNotInContext => Self::ConfigNotInContext,
            higgs_core::HiggsError::SubsystemNotConfigured(name) => {
                Self::SubsystemNotConfigured(name)
            }
            higgs_core::HiggsError::Internal => Self::Internal,
        }
    }
}
