pub mod stats;
pub mod token;
pub mod user;

use std::sync::Arc;

use aide::NoApi;
use aide::axum::routing::get;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::openapi::{Info, OpenApi};
use aide::swagger::Swagger;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::Serialize;

use crate::stats::StatsCollector;
use crate::token::Token;
use crate::user::User;

const SERVER_PORT: u16 = 3000;

#[derive(Clone, Default)]
pub struct AppState {
    pub public_url: String,
    pub stats: Arc<StatsCollector>,
}

#[derive(Serialize, JsonSchema)]
pub struct ErrorResponse {
    error: String,
    message: String,
}

#[derive(Serialize, JsonSchema)]
pub struct TokenResponseData {
    token: String,
    url: String,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum TokenResponse {
    Valid {
        root: TokenResponseData,
        children: Vec<TokenResponseData>,
    },
    AlreadyUsed,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let public_url = std::env::var("PUBLIC_URL").expect("missing env variable 'PUBLIC_URL'");

    let state = AppState {
        public_url,
        stats: Default::default(),
    };

    let app = ApiRouter::new()
        .api_route("/crawl/", get(get_crawl))
        .api_route("/crawl/{token}/", get(get_crawl_with_token))
        .api_route("/ladder/", get(get_ladder))
        .route("/openapi.json", get(serve_api))
        .route("/docs/", Swagger::new("/openapi.json").axum_route())
        .with_state(state);

    let mut api = OpenApi {
        info: Info {
            description: Some("Some toy crawlable API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let bind_address = format!("0.0.0.0:{SERVER_PORT}");
    tracing::info!("Listening on address {bind_address}");
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();

    axum::serve(
        listener,
        app.finish_api(&mut api)
            .layer(Extension(api))
            .into_make_service(),
    )
    .await
    .unwrap();
}

fn build_token_response(token: Token, state: AppState) -> TokenResponse {
    let token_response_data = |token: Token| {
        let hex = token.as_hex();
        let url = format!("{}/crawl/{hex}/", state.public_url);
        TokenResponseData { token: hex, url }
    };

    let children = token.iter_children().map(token_response_data).collect();
    let root = token_response_data(token);
    TokenResponse::Valid { root, children }
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> NoApi<Json<OpenApi>> {
    NoApi(Json(api))
}

async fn get_ladder(State(state): State<AppState>) -> impl IntoApiResponse {
    Json(state.stats.ladder())
}

async fn get_crawl(user: User, State(state): State<AppState>) -> Json<TokenResponse> {
    let token = Token::from_user(&user);
    Json(build_token_response(token, state))
}

async fn get_crawl_with_token(
    user: User,
    Path(token_str): Path<String>,
    State(state): State<AppState>,
) -> Json<TokenResponse> {
    let token = Token::validate_from_hex(&user, token_str.as_bytes()).unwrap();
    token.compute().await;
    Json(build_token_response(token, state.clone()))
}
