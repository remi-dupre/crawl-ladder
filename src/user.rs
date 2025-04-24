use aide::openapi::{Operation, Parameter, ParameterData};
use aide::operation::{OperationInput, add_parameters};
use axum::Json;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::{HeaderMap, ToStrError};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::Serialize;

use crate::ErrorResponse;

const USER_SUB_HEADERS: [&str; 2] = ["X-User", "X-Forwarded-User"];

pub enum UserError {
    InvalidId(ToStrError),
    MissingAuthHeader,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let resp = match self {
            Self::InvalidId(err) => ErrorResponse {
                error: format!("User ID is not well formatted: {err}"),
                message: "The user ID must be a valid UTF-8 string".to_string(),
            },
            Self::MissingAuthHeader => ErrorResponse {
                error: "Missing authentification header".to_string(),
                message: format!(
                    "You must specify a user ID through any of the following headers: {:?}.",
                    USER_SUB_HEADERS,
                ),
            },
        };

        (StatusCode::UNAUTHORIZED, Json(resp)).into_response()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, JsonSchema)]
pub struct User {
    pub id: String,
}

impl OperationInput for User {
    fn operation_input(ctx: &mut aide::generate::GenContext, operation: &mut Operation) {
        let s = ctx.schema.subschema_for::<String>();

        add_parameters(
            ctx,
            operation,
            [Parameter::Header {
                parameter_data: ParameterData {
                    name: "X-User".to_string(),
                    description: None,
                    required: true,
                    format: aide::openapi::ParameterSchemaOrContent::Schema(
                        aide::openapi::SchemaObject {
                            json_schema: s,
                            example: None,
                            external_docs: None,
                        },
                    ),
                    extensions: Default::default(),
                    deprecated: None,
                    example: None,
                    examples: Default::default(),
                    explode: None,
                },
                style: aide::openapi::HeaderStyle::Simple,
            }],
        );
    }
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.id)
    }
}

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = UserError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let headers = HeaderMap::from_request_parts(parts, state)
            .await
            .expect("infaillible");

        let raw_id = USER_SUB_HEADERS
            .into_iter()
            .find_map(|header| headers.get(header))
            .ok_or(UserError::MissingAuthHeader)?;

        let id = raw_id.to_str().map_err(UserError::InvalidId)?.to_string();
        Ok(Self { id })
    }
}
