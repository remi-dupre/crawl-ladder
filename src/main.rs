pub mod stats;
pub mod token;
pub mod user;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::stats::{LadderItem, StatsCollector};
use crate::token::Token;
use crate::user::User;

const SERVER_PORT: u16 = 3000;

#[derive(Clone, Default)]
pub struct AppState {
    pub public_url: String,
    pub stats: Arc<StatsCollector>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}

#[derive(Serialize)]
pub struct TokenResponseData {
    token: String,
    url: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    root: TokenResponseData,
    children: Vec<TokenResponseData>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let public_url = std::env::var("PUBLIC_URL").expect("missing env variable 'PUBLIC_URL'");

    let state = AppState {
        public_url,
        stats: Default::default(),
    };

    let app = Router::new()
        .route("/crawl/", get(get_crawl))
        .route("/crawl/{token}/", get(get_crawl_with_token))
        .route("/ladder/", get(get_ladder))
        .with_state(state);

    let bind_address = format!("0.0.0.0:{SERVER_PORT}");
    tracing::info!("Listening on address {bind_address}");
    let listener = tokio::net::TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn build_token_response(token: Token, state: &AppState) -> TokenResponse {
    let token_response_data = |token: Token| {
        let hex = token.as_hex();
        let url = format!("{}/crawl/{hex}/", state.public_url);
        TokenResponseData { token: hex, url }
    };

    let children = token.iter_children().map(token_response_data).collect();
    let root = token_response_data(token);
    TokenResponse { root, children }
}

async fn get_ladder(State(state): State<AppState>) -> Json<Vec<LadderItem>> {
    state.stats.ladder().into()
}

async fn get_crawl(user: User, State(state): State<AppState>) -> (StatusCode, Json<TokenResponse>) {
    let token = Token::from_user(&user);
    (StatusCode::OK, build_token_response(token, &state).into())
}

async fn get_crawl_with_token(
    user: User,
    Path(token_str): Path<String>,
    State(state): State<AppState>,
) -> axum::response::Response {
    let token = Token::validate_from_hex(&user, token_str.as_bytes()).unwrap();
    token.compute().await;
    let resp = build_token_response(token, &state);

    if state.stats.made_request(user, token) {
        (StatusCode::GONE, "Token is already used").into_response()
    } else {
        (StatusCode::OK, Json(resp)).into_response()
    }
}
