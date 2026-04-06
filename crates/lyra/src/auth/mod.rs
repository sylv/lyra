mod error;
mod extractors;
mod guards;
mod libraries;
mod login;
mod sessions;

pub(super) const TOKEN_REFRESH_WITHIN_EXPIRY_DAYS: u64 = 21; // refresh within 3 weeks of expiry
pub(super) const LAST_SEEN_UPDATE_INTERVAL_SECONDS: i64 = 12 * 60 * 60; // 12 hours
pub(super) const TOKEN_EXPIRY_DAYS: u64 = 45; // sessions expire after 45 days of inactivity

pub use error::AuthError;
pub use extractors::{LazyRequestAuth, RequestAuth};
pub use guards::{AuthenticatedGuard, PermissionGuard};
pub use libraries::{accessible_library_ids, ensure_library_access};
pub use login::{find_pending_invite_user, post_login};
pub use sessions::{create_session_for_user, get_set_cookie_headers_for_session};
