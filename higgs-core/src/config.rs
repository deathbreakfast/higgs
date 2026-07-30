use std::sync::Arc;

use crate::HiggsError;
use crate::HiggsValenceFactory;

/// Server-side configuration holding subsystem backends.
///
/// Constructed once at startup via [`HiggsConfig::builder()`] and stored
/// as `Arc<HiggsConfig>` in Leptos context. Per-request context holds a
/// reference to this config for Valence factory access.
///
/// Runnable: `cargo run -p higgs --example config_boot`
///
/// # Examples
///
/// ```rust,no_run
/// # #[cfg(feature = "test-utils")]
/// # fn boot() -> Result<(), higgs_core::HiggsError> {
/// use higgs_core::{HiggsConfig, test_support::UnreachableValenceFactory};
///
/// let config = HiggsConfig::builder()
///     .valence_factory(UnreachableValenceFactory)
///     .build()?;
/// let _factory = config.valence_factory();
/// # let _ = config;
/// # Ok(())
/// # }
/// ```
pub struct HiggsConfig {
    pub(crate) valence_factory: Arc<dyn HiggsValenceFactory>,
}

impl HiggsConfig {
    /// Start building a [`HiggsConfig`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "test-utils")]
    /// # fn boot() -> Result<(), higgs_core::HiggsError> {
    /// use higgs_core::{HiggsConfig, test_support::UnreachableValenceFactory};
    ///
    /// let config = HiggsConfig::builder()
    ///     .valence_factory(UnreachableValenceFactory)
    ///     .build()?;
    /// # let _ = config;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> HiggsConfigBuilder {
        HiggsConfigBuilder::default()
    }

    /// The configured [`HiggsValenceFactory`], used to build request-scoped `Valence`.
    pub fn valence_factory(&self) -> &Arc<dyn HiggsValenceFactory> {
        &self.valence_factory
    }
}

/// Builder for [`HiggsConfig`].
///
/// `valence_factory` is always required.
#[derive(Default)]
pub struct HiggsConfigBuilder {
    valence_factory: Option<Arc<dyn HiggsValenceFactory>>,
}

impl HiggsConfigBuilder {
    /// Set the Valence factory (required).
    pub fn valence_factory(mut self, factory: impl HiggsValenceFactory) -> Self {
        self.valence_factory = Some(Arc::new(factory));
        self
    }

    /// Set the Valence factory from an existing `Arc` (required).
    pub fn valence_factory_arc(mut self, factory: Arc<dyn HiggsValenceFactory>) -> Self {
        self.valence_factory = Some(factory);
        self
    }

    /// Build the configuration. Fails if `valence_factory` was not set.
    ///
    /// # Errors
    ///
    /// [`HiggsError::Internal`] when [`Self::valence_factory`] /
    /// [`Self::valence_factory_arc`] was never called.
    pub fn build(self) -> Result<HiggsConfig, HiggsError> {
        let valence_factory = self
            .valence_factory
            .ok_or_else(|| HiggsError::internal("valence_factory is required"))?;

        Ok(HiggsConfig { valence_factory })
    }
}
