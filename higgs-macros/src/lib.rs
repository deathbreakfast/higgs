//! # higgs-macros — server-function wiring
//!
//! Proc macros that standardize operation context for Leptos server functions used
//! with platform crate `higgs`.
//!
//! ## Capabilities
//!
//! - [`server`] — wraps Leptos `#[server]`, sets the task-local operation name before
//!   the body runs (read by `higgs_host::current_operation` /
//!   `higgs::Higgs::unsafe_system_valence`)
//! - `#[server(auth)]` — also requires a session via `higgs::require_session`
//!
//! `permission = "..."` enforcement lands after the `gauge` crate extracts; it is
//! **not** available in this release.
//!
//! # Organized by task
//!
//! | Task | Start here |
//! |------|------------|
//! | Attribute a Leptos `#[server]` fn with operation context | [`server`] (attribute macro) — [example](#quick-example) |
//! | Permission-gated server fns | not yet — see `permission = "..."` note above |
//!
//! # Typical usage
//!
//! 1. Depend on `higgs` (package name **`higgs`**, feature `ssr`) and `higgs-macros`.
//! 2. Annotate server functions with Leptos `#[server]` and [`server`].
//! 3. Call `higgs::Higgs::from_request` inside the body; `unsafe_system_valence` picks up
//!    the function name as the operation (`higgs::with_operation` → `higgs_host` task-local).
//!
//! # Quick example
//!
//! ```ignore
//! use higgs::Higgs;
//! use leptos::prelude::*;
//!
//! #[server]
//! #[higgs_macros::server]
//! pub async fn public_ping() -> Result<(), ServerFnError> {
//!     Ok(())
//! }
//!
//! #[server]
//! #[higgs_macros::server(auth)]
//! pub async fn private_action() -> Result<(), ServerFnError> {
//!     let ctx = Higgs::from_request().await?;
//!     let _valence = ctx.valence().map_err(ServerFnError::new)?;
//!     Ok(())
//! }
//! ```
//!
//! See also platform crate `higgs` crate-root docs for boot wiring and feature flags.
use proc_macro::TokenStream;

mod server;

/// Wrapper around Leptos `#[server]` that sets operation context before running
/// the function body.
///
/// Available without Cargo features on this crate. Pair with platform `higgs`
/// feature `ssr` so `Higgs::from_request` compiles in the host.
///
/// # Usage
///
/// - `#[higgs_macros::server]` — public or self-authorized endpoints
/// - `#[higgs_macros::server(auth)]` — requires `SessionSnapshot` (signed-in)
///
/// `permission = "..."` is **not** available yet.
///
/// # Examples
///
/// ```ignore
/// use higgs::Higgs;
/// use leptos::prelude::*;
///
/// #[server]
/// #[higgs_macros::server]
/// pub async fn public_ping() -> Result<(), ServerFnError> {
///     Ok(())
/// }
///
/// #[server]
/// #[higgs_macros::server(auth)]
/// pub async fn private_action() -> Result<(), ServerFnError> {
///     let ctx = Higgs::from_request().await?;
///     let _valence = ctx.valence().map_err(ServerFnError::new)?;
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn server(attr: TokenStream, input: TokenStream) -> TokenStream {
    server::expand_server(attr, input)
}
