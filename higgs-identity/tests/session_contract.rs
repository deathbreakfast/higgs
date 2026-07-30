//! Integration tests for the public session identity contract.
//!
//! Hosts implement [`higgs_identity::SessionIdentity`] and insert
//! [`higgs_identity::SessionSnapshot`] into Axum extensions; these tests lock that
//! surface from outside the crate.

use higgs_identity::{SessionIdentity, SessionSnapshot, SessionUserId};

struct StubUser {
    id: SessionUserId,
    hash: Vec<u8>,
}

#[async_trait::async_trait]
impl SessionIdentity for StubUser {
    fn session_user_id(&self) -> &SessionUserId {
        &self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.hash
    }
}

#[test]
fn session_identity_to_snapshot_happy_path() {
    let user = StubUser {
        id: "user:integ".into(),
        hash: b"auth-hash".to_vec(),
    };
    let snap = user.to_snapshot();
    assert_eq!(snap, SessionSnapshot::new("user:integ", b"auth-hash"));
    assert_eq!(snap.user_id, "user:integ");
    assert_eq!(snap.auth_hash, b"auth-hash");
}

#[test]
fn session_snapshot_new_happy_path() {
    let snap = SessionSnapshot::new("user:42", [1_u8, 2, 3]);
    assert_eq!(snap.user_id, "user:42");
    assert_eq!(snap.auth_hash, vec![1, 2, 3]);
}

#[test]
fn session_snapshot_serde_roundtrip_happy_path() {
    let snap = SessionSnapshot::new("user:ser", b"abc");
    let json = serde_json::to_string(&snap).expect("serialize");
    let back: SessionSnapshot = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, snap);
}

#[test]
fn session_snapshot_inequality_sad() {
    let a = SessionSnapshot::new("user:a", b"hash-a");
    let b = SessionSnapshot::new("user:b", b"hash-a");
    let c = SessionSnapshot::new("user:a", b"hash-b");
    assert_ne!(a, b);
    assert_ne!(a, c);
}
