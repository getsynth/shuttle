use async_trait::async_trait;
use axum::extract::{FromRef, FromRequest};
use axum::response::{IntoResponse, Response};
use axum::BoxError;
use http::Request;
use shuttle_common::backends::subscription::{NewSubscriptionItem, SubscriptionItem};

use crate::RouterState;

/// A wrapper for [stripe::UpdateSubscriptionItems] so we can implement [FromRequest] for it.
pub struct NewSubscriptionItemExtractor(pub stripe::UpdateSubscriptionItems);

#[async_trait]
impl<S, B> FromRequest<S, B> for NewSubscriptionItemExtractor
where
    B: axum::body::HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    RouterState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the NewSubscriptionItem, the struct that other services should use when calling
        // the endpoint to add subscription items.
        let NewSubscriptionItem { quantity, item } = axum::Json::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?
            .0;

        // Access the router state to extract price IDs.
        let state = RouterState::from_ref(state);

        let price_id = match item {
            SubscriptionItem::AwsRds => state.rds_price_id,
        };

        let update_subscription_items = stripe::UpdateSubscriptionItems {
            price: Some(price_id),
            quantity: Some(quantity),
            ..Default::default()
        };

        Ok(Self(update_subscription_items))
    }
}
