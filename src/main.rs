pub mod stats;
pub mod token;
pub mod user;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::stats::{LadderItem, StatsCollector};
use crate::token::Token;
use crate::user::User;

const PUBLIC_URL: &str = "http://localhost:3000";
const SERVER_PORT: u16 = 3000;

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

impl From<Token<'_>> for TokenResponseData {
    fn from(value: Token) -> Self {
        let hex = value.as_hex();
        let url = format!("{PUBLIC_URL}/crawl/{hex}/");
        Self { token: hex, url }
    }
}

#[derive(Serialize)]
pub struct TokenResponse {
    root: TokenResponseData,
    children: Vec<TokenResponseData>,
}

impl From<Token<'_>> for TokenResponse {
    fn from(value: Token<'_>) -> Self {
        let children = value.iter_children().map(Into::into).collect();
        let root = value.into();
        Self { root, children }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(StatsCollector::default());

    let app = Router::new()
        .route("/crawl/", get(get_crawl))
        .route("/crawl/{token}/", get(get_crawl_with_token))
        .route("/ladder/", get(get_ladder))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{SERVER_PORT}"))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn get_ladder(State(stats): State<Arc<StatsCollector>>) -> Json<Vec<LadderItem>> {
    stats.ladder().into()
}

async fn get_crawl(user: User) -> (StatusCode, Json<TokenResponse>) {
    let token = Token::from_user(&user);
    (StatusCode::OK, TokenResponse::from(token).into())
}

async fn get_crawl_with_token(
    user: User,
    Path(token): Path<String>,
    State(stats): State<Arc<StatsCollector>>,
) -> (StatusCode, Json<TokenResponse>) {
    let token = Token::validate_from_hex(&user, token.as_bytes()).unwrap();
    token.compute().await;
    let resp = TokenResponse::from(token);
    stats.made_request(user);
    (StatusCode::OK, resp.into())
}
