//! Host guidance for Valence actor-JSON policies on shared factories.
//!
//! SSR [`crate::Higgs::unsafe_system_valence`] **explicitly mints** a System actor and must
//! remain distinct from untrusted JSON paths (enqueue / event / schedule payloads that
//! carry `actor_json`). For those external edges, install
//! [`crate::actor_policy::external_actor_json_policy`] on
//! `RouterValenceFactoryConfig` (or your host factory wrapper) with
//! `valence::ActorTrust::External`.
//!
//! # Examples
//!
//! ```ignore
//! use higgs_core::actor_policy::external_actor_json_policy;
//! use valence::RouterValenceFactoryConfig;
//!
//! let config = RouterValenceFactoryConfig::new("default")
//!     .actor_json_policy(external_actor_json_policy());
//! ```

/// Re-export of Valence's fail-closed policy for untrusted `actor_json`.
pub use valence::RejectExternalSystemActor;

/// Recommended policy for host factories that rebuild Valence from external actor JSON.
///
/// # Examples
///
/// ```ignore
/// use higgs_core::actor_policy::external_actor_json_policy;
/// use valence::RouterValenceFactoryConfig;
///
/// let config = RouterValenceFactoryConfig::new("default")
///     .actor_json_policy(external_actor_json_policy());
/// ```
#[must_use]
pub const fn external_actor_json_policy() -> RejectExternalSystemActor {
    RejectExternalSystemActor
}

#[cfg(test)]
mod tests {
    use super::*;
    use valence::{ActorJsonPolicy, ActorTrust};

    #[test]
    fn external_actor_json_policy_rejects_system_happy_path() {
        let policy = external_actor_json_policy();
        let system = serde_json::json!({"System":{"operation":"x"}});
        let err = policy
            .validate(ActorTrust::External, &system)
            .expect_err("System must be rejected for External trust");
        let _ = err;
    }

    #[test]
    fn external_actor_json_policy_allows_user_happy_path() {
        let policy = external_actor_json_policy();
        let user = serde_json::json!({"User":{"user_id":"user:1"}});
        policy
            .validate(ActorTrust::External, &user)
            .expect("User actor allowed");
    }
}
