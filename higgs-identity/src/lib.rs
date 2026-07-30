//! # higgs-identity — session contract for hosts
//!
//! Abstract auth/session identity surface shared by host extractors and request
//! public crates. Concrete Valence user models live in product crates (e.g.
//! `lepton-identity`); adapters implement [`SessionIdentity`] and register
//! [`SessionSnapshot`] in Axum extensions for host middleware to consume.
//!
//! ## Capabilities
//!
//! - [`SessionUserId`] — stable session user identifier type
//! - [`SessionSnapshot`] — minimal authenticated session, stored in Axum request
//!   extensions by host middleware and read by `higgs-host` / `higgs` to build Valence
//!   actors
//! - [`SessionIdentity`] — implemented by concrete user types to produce a
//!   [`SessionSnapshot`] via [`SessionIdentity::to_snapshot`]
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | Stable session user id type | [`SessionUserId`] |
//! | Minimal authenticated session for Axum extensions | [`SessionSnapshot`], [`SessionSnapshot::new`] |
//! | Adapt a concrete user type to the session contract | [`SessionIdentity`], [`SessionIdentity::to_snapshot`] — [example](#quick-example) |
//!
//! # Typical host flow
//!
//! 1. Implement [`SessionIdentity`] on your product user type.
//! 2. Middleware calls [`SessionIdentity::to_snapshot`] and inserts
//!    `Extension<SessionSnapshot>` on authenticated requests.
//! 3. `higgs-host::host_ctx` / `higgs::Higgs::from_request` read the snapshot and map
//!    it to a Valence `User` actor (or `Anonymous` when absent).
//!
//! Live session invalidation (comparing auth hashes after credential change) is the
//! responsibility of host session middleware (e.g. axum-login). This crate provides
//! [`SessionSnapshot::auth_hash_eq`] for hosts that reload and compare hashes.
//!
//! # Quick example
//!
//! ```rust
//! use higgs_identity::{SessionIdentity, SessionSnapshot, SessionUserId};
//!
//! struct StubUser {
//!     id: SessionUserId,
//!     hash: Vec<u8>,
//! }
//!
//! #[async_trait::async_trait]
//! impl SessionIdentity for StubUser {
//!     fn session_user_id(&self) -> &SessionUserId {
//!         &self.id
//!     }
//!     fn session_auth_hash(&self) -> &[u8] {
//!         &self.hash
//!     }
//! }
//!
//! let user = StubUser {
//!     id: "user:1".into(),
//!     hash: b"abc".to_vec(),
//! };
//! let snap: SessionSnapshot = user.to_snapshot();
//! assert_eq!(snap.user_id, "user:1");
//! ```
//!
//! # Notes
//!
//! This crate defines only the contract — no Valence schemas, Leptos, or Axum
//! wiring. Extraction lives in `higgs-host`; composed request context lives in `higgs`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

/// Stable session user identifier (typically a Surreal record id string).
///
/// # Examples
///
/// ```rust
/// use higgs_identity::SessionUserId;
///
/// let id: SessionUserId = "user:1".into();
/// assert_eq!(id, "user:1");
/// ```
pub type SessionUserId = String;

/// Minimal authenticated session snapshot for host/request layers.
///
/// Host middleware (e.g. `lepton-host-adapter`) populates this in Axum extensions
/// so crates like `higgs` can build Valence actors without importing concrete user models.
///
/// Prefer [`SessionSnapshot::new`] over [`Default`] — empty `auth_hash` is rarely meaningful.
///
/// # Examples
///
/// ```rust
/// use higgs_identity::SessionSnapshot;
///
/// let snap = SessionSnapshot::new("user:1", b"abc");
/// assert_eq!(snap.user_id, "user:1");
/// assert!(snap.auth_hash_eq(b"abc"));
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSnapshot {
    /// The authenticated user's stable session id.
    pub user_id: SessionUserId,
    /// Opaque auth-hash bytes used to invalidate sessions when credentials change.
    pub auth_hash: Vec<u8>,
}

impl SessionSnapshot {
    /// Construct a snapshot from a user id and auth-hash bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use higgs_identity::SessionSnapshot;
    ///
    /// let snap = SessionSnapshot::new("user:1", b"abc");
    /// assert_eq!(snap.user_id, "user:1");
    /// assert_eq!(snap.auth_hash, b"abc");
    /// ```
    pub fn new(user_id: impl Into<SessionUserId>, auth_hash: impl AsRef<[u8]>) -> Self {
        Self {
            user_id: user_id.into(),
            auth_hash: auth_hash.as_ref().to_vec(),
        }
    }

    /// Constant-time equality of auth hashes (same length required).
    ///
    /// Returns `false` when lengths differ. Hosts that reload the current user hash
    /// after credential change can use this to decide whether a stored session is stale.
    #[must_use]
    pub fn auth_hash_eq(&self, expected: &[u8]) -> bool {
        if self.auth_hash.len() != expected.len() {
            return false;
        }
        bool::from(self.auth_hash.as_slice().ct_eq(expected))
    }
}

/// Adapter-facing identity contract (implemented in product repos, not here).
///
/// # Examples
///
/// ```rust
/// use higgs_identity::{SessionIdentity, SessionSnapshot, SessionUserId};
///
/// struct StubUser {
///     id: SessionUserId,
///     hash: Vec<u8>,
/// }
///
/// #[async_trait::async_trait]
/// impl SessionIdentity for StubUser {
///     fn session_user_id(&self) -> &SessionUserId {
///         &self.id
///     }
///     fn session_auth_hash(&self) -> &[u8] {
///         &self.hash
///     }
/// }
///
/// let user = StubUser {
///     id: "user:1".into(),
///     hash: b"abc".to_vec(),
/// };
/// let snap: SessionSnapshot = user.to_snapshot();
/// assert_eq!(snap.user_id, "user:1");
/// ```
#[async_trait]
pub trait SessionIdentity: Send + Sync {
    /// The implementor's stable session user id.
    fn session_user_id(&self) -> &SessionUserId;
    /// The implementor's opaque auth-hash bytes.
    fn session_auth_hash(&self) -> &[u8];

    /// Produce a [`SessionSnapshot`] from [`Self::session_user_id`] and
    /// [`Self::session_auth_hash`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use higgs_identity::{SessionIdentity, SessionSnapshot, SessionUserId};
    ///
    /// struct StubUser {
    ///     id: SessionUserId,
    ///     hash: Vec<u8>,
    /// }
    ///
    /// #[async_trait::async_trait]
    /// impl SessionIdentity for StubUser {
    ///     fn session_user_id(&self) -> &SessionUserId { &self.id }
    ///     fn session_auth_hash(&self) -> &[u8] { &self.hash }
    /// }
    ///
    /// let user = StubUser { id: "user:2".into(), hash: b"xyz".to_vec() };
    /// assert_eq!(user.to_snapshot(), SessionSnapshot::new("user:2", b"xyz"));
    /// ```
    fn to_snapshot(&self) -> SessionSnapshot {
        SessionSnapshot::new(self.session_user_id(), self.session_auth_hash())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubUser {
        id: SessionUserId,
        hash: Vec<u8>,
    }

    #[async_trait]
    impl SessionIdentity for StubUser {
        fn session_user_id(&self) -> &SessionUserId {
            &self.id
        }

        fn session_auth_hash(&self) -> &[u8] {
            &self.hash
        }
    }

    #[test]
    fn snapshot_new_copies_fields_happy_path() {
        let snap = SessionSnapshot::new("user:1", b"abc");
        assert_eq!(snap.user_id, "user:1");
        assert_eq!(snap.auth_hash, b"abc");
    }

    #[test]
    fn session_identity_to_snapshot_happy_path() {
        let user = StubUser {
            id: "user:2".into(),
            hash: b"xyz".to_vec(),
        };
        assert_eq!(user.to_snapshot(), SessionSnapshot::new("user:2", b"xyz"));
    }

    #[test]
    fn auth_hash_eq_happy_path() {
        let snap = SessionSnapshot::new("user:1", b"secret-hash");
        assert!(snap.auth_hash_eq(b"secret-hash"));
    }

    #[test]
    fn auth_hash_eq_sad_mismatch() {
        let snap = SessionSnapshot::new("user:1", b"secret-hash");
        assert!(!snap.auth_hash_eq(b"other-hash"));
        assert!(!snap.auth_hash_eq(b"short"));
    }
}
