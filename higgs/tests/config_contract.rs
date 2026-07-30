//! Integration tests for platform `higgs` config builder and error mapping.

use std::sync::Arc;

use higgs::{HiggsConfig, HiggsError};
use higgs_core::test_support::UnreachableValenceFactory;

#[test]
fn platform_config_builder_happy_path() {
    let config = HiggsConfig::builder()
        .valence_factory(UnreachableValenceFactory)
        .build()
        .expect("factory set");
    let _ = config.valence_factory();
    let _ = config.core();
    let _ = config.core_arc();
}

#[test]
fn platform_config_builder_arc_factory_happy_path() {
    let factory: Arc<dyn higgs::HiggsValenceFactory> =
        higgs_core::test_support::unreachable_valence_factory();
    let config = HiggsConfig::builder()
        .valence_factory_arc(factory)
        .build()
        .expect("factory set");
    let _ = config.valence_factory();
}

#[test]
fn platform_config_builder_missing_factory_sad() {
    match HiggsConfig::builder().build() {
        Err(HiggsError::Internal) => {}
        Err(e) => panic!("expected Internal missing factory, got {e}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}

#[test]
fn higgs_error_from_core_maps_variants_happy_path() {
    let mapped = HiggsError::from(higgs_core::HiggsError::ConfigNotInContext);
    assert!(matches!(mapped, HiggsError::ConfigNotInContext));
    let mapped = HiggsError::from(higgs_core::HiggsError::SubsystemNotConfigured("chronon"));
    assert!(matches!(
        mapped,
        HiggsError::SubsystemNotConfigured("chronon")
    ));
    let mapped = HiggsError::from(higgs_core::HiggsError::Internal);
    assert!(matches!(mapped, HiggsError::Internal));
}

#[cfg(feature = "chronon")]
#[test]
fn chronon_accessors_without_config_sad() {
    let config = HiggsConfig::builder()
        .valence_factory(UnreachableValenceFactory)
        .build()
        .expect("factory set");
    match config.scheduler() {
        Err(HiggsError::SubsystemNotConfigured("chronon")) => {}
        Err(e) => panic!("expected SubsystemNotConfigured, got {e}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
    match config.chronon_backend() {
        Err(HiggsError::SubsystemNotConfigured("chronon")) => {}
        Err(e) => panic!("expected SubsystemNotConfigured, got {e}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}

#[cfg(feature = "boson")]
#[test]
fn boson_accessor_without_config_sad() {
    let config = HiggsConfig::builder()
        .valence_factory(UnreachableValenceFactory)
        .build()
        .expect("factory set");
    match config.boson_backend() {
        Err(HiggsError::SubsystemNotConfigured("boson")) => {}
        Err(e) => panic!("expected SubsystemNotConfigured, got {e}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}

#[cfg(feature = "photon")]
#[test]
fn photon_accessor_without_config_sad() {
    let config = HiggsConfig::builder()
        .valence_factory(UnreachableValenceFactory)
        .build()
        .expect("factory set");
    match config.photon() {
        Err(HiggsError::SubsystemNotConfigured("photon")) => {}
        Err(e) => panic!("expected SubsystemNotConfigured, got {e}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}
