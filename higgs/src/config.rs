//! Process-wide platform Higgs configuration.

use std::sync::Arc;

use crate::HiggsError;
use crate::HiggsValenceFactory;

/// Server-side configuration holding Valence factory plus optional subsystem backends.
///
/// Constructed once at startup via [`HiggsConfig::builder()`] and stored as
/// `Arc<HiggsConfig>` in Leptos context. See the [SSR example](crate#ssr--leptos-server-functions)
/// and [Workers](crate#workers--chronon--boson--photon)
/// for boot wiring.
///
/// Runnable: `cargo run -p higgs --example config_boot`
///
/// # Examples
///
/// ```rust,no_run
/// # #[cfg(feature = "test-utils")]
/// # fn boot() -> Result<(), higgs::HiggsError> {
/// use higgs::{HiggsConfig, test_support::UnreachableValenceFactory};
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
    core: Arc<higgs_core::HiggsConfig>,

    #[cfg(feature = "chronon")]
    pub(crate) scheduler: Option<Arc<chronon_coordinator::Scheduler>>,
    #[cfg(feature = "chronon")]
    pub(crate) chronon_backend: Option<Arc<dyn chronon_coordinator::ChrononCoordinatorBackend>>,
    #[cfg(feature = "chronon")]
    pub(crate) script_registry: Option<Arc<chronon_coordinator::ScriptRegistry>>,

    #[cfg(feature = "photon")]
    pub(crate) photon: Option<Arc<photon::Photon>>,

    #[cfg(feature = "boson")]
    pub(crate) boson_backend: Option<Arc<dyn boson_coordinator::BosonCoordinatorBackend>>,
}

impl HiggsConfig {
    /// Start building a [`HiggsConfig`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "test-utils")]
    /// # fn boot() -> Result<(), higgs::HiggsError> {
    /// use higgs::{HiggsConfig, test_support::UnreachableValenceFactory};
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

    /// Shared thin-hub config (Valence factory only).
    pub fn core(&self) -> &higgs_core::HiggsConfig {
        &self.core
    }

    /// Shared thin-hub config as an `Arc`.
    pub fn core_arc(&self) -> Arc<higgs_core::HiggsConfig> {
        Arc::clone(&self.core)
    }

    /// The configured Valence factory.
    pub fn valence_factory(&self) -> &Arc<dyn HiggsValenceFactory> {
        self.core.valence_factory()
    }

    /// Chronon scheduler handle, when configured.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not registered on the builder.
    #[cfg(feature = "chronon")]
    pub fn scheduler(&self) -> Result<&chronon_coordinator::Scheduler, HiggsError> {
        self.scheduler
            .as_deref()
            .ok_or(HiggsError::SubsystemNotConfigured("chronon"))
    }

    /// Chronon coordinator backend, when configured.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not registered on the builder.
    #[cfg(feature = "chronon")]
    pub fn chronon_backend(
        &self,
    ) -> Result<&dyn chronon_coordinator::ChrononCoordinatorBackend, HiggsError> {
        self.chronon_backend
            .as_deref()
            .ok_or(HiggsError::SubsystemNotConfigured("chronon"))
    }

    /// Chronon script registry, when configured.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not registered on the builder.
    #[cfg(feature = "chronon")]
    pub fn script_registry(&self) -> Result<&chronon_coordinator::ScriptRegistry, HiggsError> {
        self.script_registry
            .as_deref()
            .ok_or(HiggsError::SubsystemNotConfigured("chronon"))
    }

    /// Photon handle, when configured.
    ///
    /// Available on crate feature `photon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Photon was not registered on the builder.
    #[cfg(feature = "photon")]
    pub fn photon(&self) -> Result<&photon::Photon, HiggsError> {
        self.photon
            .as_deref()
            .ok_or(HiggsError::SubsystemNotConfigured("photon"))
    }

    /// Boson coordinator backend, when configured.
    ///
    /// Available on crate feature `boson`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Boson was not registered on the builder.
    #[cfg(feature = "boson")]
    pub fn boson_backend(
        &self,
    ) -> Result<&dyn boson_coordinator::BosonCoordinatorBackend, HiggsError> {
        self.boson_backend
            .as_deref()
            .ok_or(HiggsError::SubsystemNotConfigured("boson"))
    }

    /// Shared Chronon scheduler arc.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not registered on the builder.
    #[cfg(feature = "chronon")]
    pub fn scheduler_arc(&self) -> Result<Arc<chronon_coordinator::Scheduler>, HiggsError> {
        self.scheduler
            .clone()
            .ok_or(HiggsError::SubsystemNotConfigured("chronon"))
    }

    /// Shared Chronon backend arc.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not registered on the builder.
    #[cfg(feature = "chronon")]
    pub fn chronon_backend_arc(
        &self,
    ) -> Result<Arc<dyn chronon_coordinator::ChrononCoordinatorBackend>, HiggsError> {
        self.chronon_backend
            .clone()
            .ok_or(HiggsError::SubsystemNotConfigured("chronon"))
    }

    /// Shared Chronon script registry arc.
    ///
    /// Available on crate feature `chronon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Chronon was not registered on the builder.
    #[cfg(feature = "chronon")]
    pub fn script_registry_arc(
        &self,
    ) -> Result<Arc<chronon_coordinator::ScriptRegistry>, HiggsError> {
        self.script_registry
            .clone()
            .ok_or(HiggsError::SubsystemNotConfigured("chronon"))
    }

    /// Shared Photon arc.
    ///
    /// Available on crate feature `photon`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Photon was not registered on the builder.
    #[cfg(feature = "photon")]
    pub fn photon_arc(&self) -> Result<Arc<photon::Photon>, HiggsError> {
        self.photon
            .clone()
            .ok_or(HiggsError::SubsystemNotConfigured("photon"))
    }

    /// Shared Boson backend arc.
    ///
    /// Available on crate feature `boson`.
    ///
    /// # Errors
    ///
    /// [`HiggsError::SubsystemNotConfigured`] if Boson was not registered on the builder.
    #[cfg(feature = "boson")]
    pub fn boson_backend_arc(
        &self,
    ) -> Result<Arc<dyn boson_coordinator::BosonCoordinatorBackend>, HiggsError> {
        self.boson_backend
            .clone()
            .ok_or(HiggsError::SubsystemNotConfigured("boson"))
    }
}

/// Builder for [`HiggsConfig`].
#[derive(Default)]
pub struct HiggsConfigBuilder {
    core: higgs_core::HiggsConfigBuilder,

    #[cfg(feature = "chronon")]
    scheduler: Option<Arc<chronon_coordinator::Scheduler>>,
    #[cfg(feature = "chronon")]
    chronon_backend: Option<Arc<dyn chronon_coordinator::ChrononCoordinatorBackend>>,
    #[cfg(feature = "chronon")]
    script_registry: Option<Arc<chronon_coordinator::ScriptRegistry>>,

    #[cfg(feature = "photon")]
    photon: Option<Arc<photon::Photon>>,

    #[cfg(feature = "boson")]
    boson_backend: Option<Arc<dyn boson_coordinator::BosonCoordinatorBackend>>,
}

impl HiggsConfigBuilder {
    /// Set the Valence factory (required).
    pub fn valence_factory(mut self, factory: impl HiggsValenceFactory) -> Self {
        self.core = self.core.valence_factory(factory);
        self
    }

    /// Set the Valence factory from an existing `Arc` (required).
    pub fn valence_factory_arc(mut self, factory: Arc<dyn HiggsValenceFactory>) -> Self {
        self.core = self.core.valence_factory_arc(factory);
        self
    }

    /// Provide Chronon subsystem backends.
    ///
    /// Available on crate feature `chronon`.
    #[cfg(feature = "chronon")]
    pub fn chronon(
        mut self,
        scheduler: Arc<chronon_coordinator::Scheduler>,
        backend: Arc<dyn chronon_coordinator::ChrononCoordinatorBackend>,
        registry: Arc<chronon_coordinator::ScriptRegistry>,
    ) -> Self {
        self.scheduler = Some(scheduler);
        self.chronon_backend = Some(backend);
        self.script_registry = Some(registry);
        self
    }

    /// Provide the Photon subsystem backend.
    ///
    /// Available on crate feature `photon`.
    #[cfg(feature = "photon")]
    pub fn photon(mut self, photon: Arc<photon::Photon>) -> Self {
        self.photon = Some(photon);
        self
    }

    /// Provide the Boson subsystem backend.
    ///
    /// Available on crate feature `boson`.
    #[cfg(feature = "boson")]
    pub fn boson(mut self, backend: Arc<dyn boson_coordinator::BosonCoordinatorBackend>) -> Self {
        self.boson_backend = Some(backend);
        self
    }

    /// Build the configuration. Fails if `valence_factory` was not set.
    ///
    /// # Errors
    ///
    /// [`HiggsError::Internal`] when [`Self::valence_factory`] /
    /// [`Self::valence_factory_arc`] was never called.
    pub fn build(self) -> Result<HiggsConfig, HiggsError> {
        let core = Arc::new(self.core.build().map_err(HiggsError::from)?);

        Ok(HiggsConfig {
            core,
            #[cfg(feature = "chronon")]
            scheduler: self.scheduler,
            #[cfg(feature = "chronon")]
            chronon_backend: self.chronon_backend,
            #[cfg(feature = "chronon")]
            script_registry: self.script_registry,
            #[cfg(feature = "photon")]
            photon: self.photon,
            #[cfg(feature = "boson")]
            boson_backend: self.boson_backend,
        })
    }
}
