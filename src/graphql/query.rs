use crate::TestFile;
use async_graphql::{Context, Object};
use std::sync::Arc;

pub struct Query;

#[Object]
impl Query {
    async fn file_list(&self, ctx: &Context<'_>) -> Result<Vec<TestFile>, async_graphql::Error> {
        let files = ctx.data::<Arc<Vec<TestFile>>>()?;
        Ok(files.to_vec())
    }
}
