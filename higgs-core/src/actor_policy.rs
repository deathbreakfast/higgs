//! Host guidance for Valence actor-JSON policies on shared factories.
//!
//! SSR `Higgs::unsafe_system_valence` **explicitly mints** a System actor and must
//! remain distinct from untrusted JSON paths (enqueue / event / schedule payloads that
//! carry `actor_json`). For those external edges, install
//! [`crate::actor_policy::external_actor_json_policy`] on
//! `RouterValenceFactoryConfig` (or your host factory wrapper) with
//! `valence::ActorTrust::External`.
//!
//! Prerequisites: a host factory that rebuilds Valence from enqueue / event / schedule
//! `actor_json`. First success: attach the policy when building the shared factory, then
//! rebuild Valence from external actor JSON — `System` and `ServiceUser` JSON fail closed. A
//! service user never comes from a browser session, so external JSON claiming one is treated
//! as forged; build those actors on the server with `ActorTrust::Internal`. Runnable demo:
//! `cargo run -p higgs --example shared_factory --features ssr` (prints that external
//! System was rejected).
//!
//! # Examples
//!
//! Install the fail-closed policy on the shared factory config, then validate external
//! actor JSON (System and ServiceUser rejected, User allowed).
//!
//! ```ignore
//! use higgs_core::actor_policy::external_actor_json_policy;
//! use valence::{ActorJsonPolicy, ActorTrust, RouterValenceFactoryConfig};
//!
//! // Host keeps `config` on the shared factory when rebuilding from actor JSON.
//! let config = RouterValenceFactoryConfig::new("default")
//!     .actor_json_policy(external_actor_json_policy());
//! let policy = external_actor_json_policy();
//! assert!(
//!     policy
//!         .validate(
//!             ActorTrust::External,
//!             &serde_json::json!({"System":{"operation":"x"}})
//!         )
//!         .is_err(),
//!     "System must fail closed for External"
//! );
//! assert!(
//!     policy
//!         .validate(
//!             ActorTrust::External,
//!             &serde_json::json!({"ServiceUser":{"service_name":"svc:x"}})
//!         )
//!         .is_err(),
//!     "ServiceUser must fail closed for External"
//! );
//! assert!(
//!     policy
//!         .validate(
//!             ActorTrust::External,
//!             &serde_json::json!({"User":{"user_id":"user:1"}})
//!         )
//!         .is_ok(),
//!     "User actor allowed"
//! );
//! // `config` is what the host stores on the factory; policy checks above are the oracle.
//! let installed = format!("{config:?}");
//! assert!(!installed.is_empty());
//! ```
//!
//! Next: package `higgs` SSR / worker docs for where the factory Arc is registered.

/// Re-export of Valence's fail-closed policy for untrusted `actor_json`.
///
/// On `ActorTrust::External` it rejects `System` and `ServiceUser` actors; `User` and
/// `Anonymous` pass, and `ActorTrust::Internal` is never rejected. Valence now names the
/// type `RejectExternalPrivilegedActor` and keeps this name as a deprecated alias.
pub use valence::RejectExternalSystemActor;

/// Recommended policy for host factories that rebuild Valence from external actor JSON.
///
/// Rejects `System` and `ServiceUser` actors for `ActorTrust::External`.
///
/// # Examples
///
/// ```ignore
/// use higgs_core::actor_policy::external_actor_json_policy;
/// use valence::{ActorJsonPolicy, ActorTrust, RouterValenceFactoryConfig};
///
/// let config = RouterValenceFactoryConfig::new("default")
///     .actor_json_policy(external_actor_json_policy());
/// let policy = external_actor_json_policy();
/// assert!(
///     policy
///         .validate(
///             ActorTrust::External,
///             &serde_json::json!({"System":{"operation":"x"}})
///         )
///         .is_err()
/// );
/// assert!(!format!("{config:?}").is_empty());
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
    fn external_actor_json_policy_rejects_service_user_sad_path() {
        let policy = external_actor_json_policy();
        let service = serde_json::json!({"ServiceUser":{"service_name":"svc:x"}});
        policy
            .validate(ActorTrust::External, &service)
            .expect_err("ServiceUser must be rejected for External trust");
        policy
            .validate(ActorTrust::Internal, &service)
            .expect("Internal trust may carry a server-built ServiceUser");
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
