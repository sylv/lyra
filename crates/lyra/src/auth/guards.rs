use crate::{
    auth::{AuthError, RequestAuth},
    entities::users::UserPerms,
};
use async_graphql::{Context, Guard};

pub struct AuthenticatedGuard;

impl AuthenticatedGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Guard for AuthenticatedGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<(), async_graphql::Error> {
        let _ = ctx.data::<RequestAuth>()?;
        Ok(())
    }
}

pub struct PermissionGuard(UserPerms);

impl PermissionGuard {
    pub fn new(permissions: UserPerms) -> Self {
        Self(permissions)
    }
}

impl Guard for PermissionGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<(), async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        if !auth.has_permission(self.0) {
            return Err(AuthError::InsufficientPermissions.into());
        }

        Ok(())
    }
}
