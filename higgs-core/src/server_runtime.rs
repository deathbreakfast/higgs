//! Permission-denied payload encode/decode helpers for server functions.
//!
//! `higgs-macros`-generated `#[server(permission = "...")]` wrappers encode permission
//! failures into the `ServerFnError` message string using these prefixes, so callers can
//! recover structured [`PermissionErrorPayload`](crate::server_runtime::PermissionErrorPayload)
//! via [`parse_permission_error_payload`](crate::server_runtime::parse_permission_error_payload).
//!
//! # Examples
//!
//! ```rust
//! use higgs_core::server_runtime::{
//!     parse_permission_error_payload, permission_denied_payload, PermissionErrorPayload,
//! };
//!
//! let msg = permission_denied_payload("gauge.admin");
//! assert_eq!(
//!     parse_permission_error_payload(&msg),
//!     Some(PermissionErrorPayload::Denied {
//!         permission: "gauge.admin".into(),
//!     })
//! );
//! ```

use std::future::Future;

tokio::task_local! {
    static CURRENT_OPERATION: Option<&'static str>;
}

/// Message prefix for a denied-permission payload (see [`permission_denied_payload`]).
pub const PERMISSION_DENIED_PREFIX: &str = "permission_denied::";
/// Message prefix for a permission-check-failed payload (see [`permission_check_failed_payload`]).
pub const PERMISSION_CHECK_FAILED_PREFIX: &str = "permission_check_failed::";

/// Structured permission failure recovered from a server function error message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PermissionErrorPayload {
    /// The actor was denied the named permission.
    Denied {
        /// Permission name that was denied.
        permission: String,
    },
    /// The permission check itself failed (as opposed to a clean denial).
    CheckFailed {
        /// Permission name that was being checked.
        permission: String,
        /// Client-safe details (may be empty; full detail is logged server-side).
        details: String,
    },
}

/// Encode a denied-permission payload for `permission` as a `ServerFnError` message string.
pub fn permission_denied_payload(permission: &str) -> String {
    format!("{PERMISSION_DENIED_PREFIX}{permission}")
}

/// Encode a permission-check-failed payload.
///
/// `details` are logged server-side and **not** included in the client-facing string.
pub fn permission_check_failed_payload(permission: &str, details: &str) -> String {
    if !details.is_empty() {
        log::warn!(
            target: "higgs",
            "permission check failed permission={permission} details={details}"
        );
    }
    format!("{PERMISSION_CHECK_FAILED_PREFIX}{permission}")
}

/// Recover a [`PermissionErrorPayload`] from a server function error message, if it was
/// encoded by [`permission_denied_payload`] or [`permission_check_failed_payload`].
pub fn parse_permission_error_payload(message: &str) -> Option<PermissionErrorPayload> {
    if let Some(permission) = message.strip_prefix(PERMISSION_DENIED_PREFIX) {
        let permission = permission.trim();
        if !permission.is_empty() {
            return Some(PermissionErrorPayload::Denied {
                permission: permission.to_string(),
            });
        }
    }

    if let Some(rest) = message.strip_prefix(PERMISSION_CHECK_FAILED_PREFIX) {
        let mut parts = rest.splitn(2, "::");
        let permission = parts.next().unwrap_or_default().trim().to_string();
        let details = parts.next().unwrap_or_default().trim().to_string();
        if !permission.is_empty() {
            return Some(PermissionErrorPayload::CheckFailed {
                permission,
                details,
            });
        }
    }

    None
}

/// The current task-local operation name set by [`with_operation`], if any.
pub fn current_operation() -> Option<&'static str> {
    CURRENT_OPERATION
        .try_with(|operation| *operation)
        .ok()
        .flatten()
}

/// Run `fut` tagged with `operation` for attribution.
pub async fn with_operation<F, R>(operation: &'static str, fut: F) -> R
where
    F: Future<Output = R>,
{
    CURRENT_OPERATION.scope(Some(operation), fut).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_denied_round_trip_happy_path() {
        let msg = permission_denied_payload("gauge.admin");
        assert_eq!(
            parse_permission_error_payload(&msg),
            Some(PermissionErrorPayload::Denied {
                permission: "gauge.admin".into(),
            })
        );
    }

    #[test]
    fn permission_check_failed_omits_details_happy_path() {
        let msg = permission_check_failed_payload("gauge.admin", "db timeout secret=xyz");
        assert_eq!(msg, "permission_check_failed::gauge.admin");
        assert!(!msg.contains("secret"));
        assert!(!msg.contains("timeout"));
        assert_eq!(
            parse_permission_error_payload(&msg),
            Some(PermissionErrorPayload::CheckFailed {
                permission: "gauge.admin".into(),
                details: String::new(),
            })
        );
    }

    #[test]
    fn parse_rejects_unrelated_and_empty_payloads_sad() {
        assert!(parse_permission_error_payload("plain error").is_none());
        assert!(parse_permission_error_payload(PERMISSION_DENIED_PREFIX).is_none());
        assert!(parse_permission_error_payload(PERMISSION_CHECK_FAILED_PREFIX).is_none());
    }

    #[tokio::test]
    async fn with_operation_sets_task_local_operation() {
        assert!(current_operation().is_none());
        let seen = with_operation("ops.create", async { current_operation() }).await;
        assert_eq!(seen, Some("ops.create"));
        assert!(current_operation().is_none());
    }
}
