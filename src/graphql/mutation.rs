use async_graphql::Object;

pub struct Mutation;

#[Object]
impl Mutation {
    async fn placeholder(&self, username: String) -> Result<String, async_graphql::Error> {
        Ok(format!("Hello, {}!", username))
    }
}
