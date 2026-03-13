mod item;
mod root;
mod season;

use crate::auth::RequestAuth;
use async_graphql::Context;

fn current_user_id(ctx: &Context<'_>) -> Option<String> {
    let auth = ctx.data_opt::<RequestAuth>()?;
    let user = auth.get_user_or_err().ok()?;
    Some(user.id.clone())
}

fn saturating_i32_from_u64(value: u64) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
