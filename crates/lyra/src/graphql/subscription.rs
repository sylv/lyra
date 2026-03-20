use crate::{auth::RequestAuth, content_update::CONTENT_UPDATE};
use async_graphql::{Context, Enum, Subscription};
use futures_util::Stream;
use tokio::sync::broadcast;

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum ContentUpdateEvent {
    ContentUpdate,
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn content_updates(
        &self,
        ctx: &Context<'_>,
    ) -> Result<impl Stream<Item = ContentUpdateEvent>, async_graphql::Error> {
        let _ = ctx.data::<RequestAuth>()?;

        Ok(futures_util::stream::unfold(
            CONTENT_UPDATE.subscribe(),
            |mut receiver| async move {
                loop {
                    match receiver.recv().await {
                        Ok(()) => return Some((ContentUpdateEvent::ContentUpdate, receiver)),
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => return None,
                    }
                }
            },
        ))
    }
}
