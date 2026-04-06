use async_graphql::Error as GraphQLError;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use serde_json::json;

pub enum AuthError {
    UnknownUserOrWrongCredentials,
    InsufficientPermissions,
    Unauthenticated,
    TooManyAttempts,
    InternalError,
    UserStillPending,
}

impl AuthError {
    fn to_response(&self) -> (StatusCode, &'static str) {
        match self {
            AuthError::Unauthenticated => (StatusCode::UNAUTHORIZED, "Unauthenticated"),
            AuthError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
            AuthError::UnknownUserOrWrongCredentials => (
                StatusCode::UNAUTHORIZED,
                "Unknown user or wrong credentials",
            ),
            AuthError::InsufficientPermissions => {
                (StatusCode::FORBIDDEN, "Insufficient permissions")
            }
            AuthError::TooManyAttempts => (StatusCode::TOO_MANY_REQUESTS, "Too many attempts"),
            AuthError::UserStillPending => (
                StatusCode::UNAUTHORIZED,
                "User is still pending account setup",
            ),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = self.to_response();
        let body = Json(json!({
            "status_code": status.as_u16(),
            "error_message": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<AuthError> for GraphQLError {
    fn from(value: AuthError) -> Self {
        GraphQLError::new(value.to_response().1)
    }
}
