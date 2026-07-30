//! Preflight status and result types.

/// Outcome of a single preflight check.
#[derive(Debug, Clone)]
pub enum PreflightStatus {
    /// Check succeeded.
    Passed {
        /// Short message for logs / UI.
        message: String,
    },
    /// Non-fatal issue (server may continue; setup wizard can surface this).
    Warning {
        /// Short message for logs / UI.
        message: String,
        /// Additional detail lines for logs / UI.
        details: Vec<String>,
    },
    /// Fatal for this check (caller may still continue other checks).
    Failed {
        /// Short message for logs / UI.
        message: String,
        /// Additional detail lines for logs / UI.
        details: Vec<String>,
    },
}

/// One check’s result.
#[derive(Debug, Clone)]
pub struct PreflightResult {
    /// Same as [`super::PreflightCheck::name`].
    pub check_name: String,
    /// Outcome of the check.
    pub status: PreflightStatus,
}
