use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
    http::{header, StatusCode},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::client::RpcClientManager;
use crate::web::*;

#[derive(Deserialize)]
pub struct StateQuery {
    pub key: Option<String>,
}

pub async fn landing_handler() -> Html<&'static str> {
    Html(LANDING_HTML)
}

pub async fn explorer_handler() -> Html<&'static str> {
    Html(EXPLORER_HTML)
}

pub async fn brand_asset_handler(Path(asset): Path<String>) -> Response {
    let (content_type, svg_data) = match asset.as_str() {
        "zyanya-logo.svg" => ("image/svg+xml", LOGO_SVG),
        "zyanya-hero-banner.svg" => ("image/svg+xml", HERO_BANNER_SVG),
        "zyan-coin.svg" => ("image/svg+xml", ZYAN_COIN_SVG),
        "ghost-token.svg" => ("image/svg+xml", GHOST_TOKEN_SVG),
        "gas-burn-icon.svg" => ("image/svg+xml", GAS_BURN_SVG),
        "zyanya-token-set.svg" => ("image/svg+xml", TOKEN_SET_SVG),
        _ => return (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    };

    ([(header::CONTENT_TYPE, content_type)], svg_data).into_response()
}

pub async fn api_info_handler(State(client): State<Arc<RpcClientManager>>) -> Response {
    match client.get_dashboard().await {
        Ok(info) => Json(info).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_blocks_handler(State(client): State<Arc<RpcClientManager>>) -> Response {
    match client.get_recent_blocks(20).await {
        Ok(blocks) => Json(blocks).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_block_handler(
    State(client): State<Arc<RpcClientManager>>,
    Path(hash): Path<String>,
) -> Response {
    match client.get_block_detail(&hash).await {
        Ok(detail) => Json(detail).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_contract_code_handler(
    State(client): State<Arc<RpcClientManager>>,
    Path(address): Path<String>,
) -> Response {
    match client.get_contract_code(&address).await {
        Ok(info) => Json(info).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_contract_state_handler(
    State(client): State<Arc<RpcClientManager>>,
    Path(address): Path<String>,
    Query(query): Query<StateQuery>,
) -> Response {
    let key_val = query.key.as_deref()
        .map(|k| k.trim_start_matches("0x").parse::<u64>().unwrap_or(0))
        .unwrap_or(0);

    match client.get_contract_state_key(&address, key_val).await {
        Ok(val) => Json(serde_json::json!({
            "address": address,
            "key": key_val,
            "value": val
        })).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_dag_handler(State(client): State<Arc<RpcClientManager>>) -> Response {
    match client.get_dag_graph(20).await {
        Ok(dag) => Json(dag).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}
