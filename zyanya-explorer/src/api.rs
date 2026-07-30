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

pub async fn tools_handler() -> Html<&'static str> {
    Html(TOOLS_HTML)
}

pub async fn webmcp_js_handler() -> Response {
    ([(header::CONTENT_TYPE, "application/javascript")], WEBMCP_SCRIPT).into_response()
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
    let key_val = query.key.as_deref().map(|k| {
        let clean = k.trim();
        if let Some(rest) = clean.strip_prefix("0x").or_else(|| clean.strip_prefix("0X")) {
            u64::from_str_radix(rest, 16).unwrap_or(0)
        } else {
            clean.parse::<u64>().unwrap_or(0)
        }
    }).unwrap_or(0);

    match client.get_contract_state_key(&address, key_val).await {
        Ok(val) => Json(serde_json::json!({
            "address": address,
            "key": key_val,
            "value": val
        })).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn api_dag_handler(
    State(client): State<Arc<RpcClientManager>>,
    Query(pagination): Query<PaginationQuery>,
) -> Response {
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);
    match client.get_dag_graph(limit + offset).await {
        Ok(mut dag) => {
            dag.nodes = dag.nodes.into_iter().skip(offset).take(limit).collect();
            Json(dag).into_response()
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct TokenBalanceQuery {
    pub token: Option<String>,
    pub tokenAddress: Option<String>,
    pub holder: Option<String>,
}

pub async fn api_token_balance_handler(
    State(client): State<Arc<RpcClientManager>>,
    Query(query): Query<TokenBalanceQuery>,
) -> Response {
    let token = query.token.or(query.tokenAddress).unwrap_or_default();
    let holder = query.holder.unwrap_or_else(|| "1".to_string());
    if token.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing token address" }))).into_response();
    }
    match client.get_token_balance(&token, &holder).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_token_balance_post_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<TokenBalanceQuery>,
) -> Response {
    let token = payload.token.or(payload.tokenAddress).unwrap_or_default();
    let holder = payload.holder.unwrap_or_else(|| "1".to_string());
    if token.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing token address" }))).into_response();
    }
    match client.get_token_balance(&token, &holder).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct DexReservesQuery {
    pub dex: Option<String>,
    pub dexAddress: Option<String>,
}

pub async fn api_dex_reserves_handler(
    State(client): State<Arc<RpcClientManager>>,
    Query(query): Query<DexReservesQuery>,
) -> Response {
    let dex = query.dex.or(query.dexAddress).unwrap_or_default();
    if dex.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing DEX address" }))).into_response();
    }
    match client.get_dex_reserves(&dex).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_dex_reserves_post_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<DexReservesQuery>,
) -> Response {
    let dex = payload.dex.or(payload.dexAddress).unwrap_or_default();
    if dex.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing DEX address" }))).into_response();
    }
    match client.get_dex_reserves(&dex).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_contracts_handler(
    State(client): State<Arc<RpcClientManager>>,
    Query(pagination): Query<PaginationQuery>,
) -> Response {
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);
    match client.get_contracts().await {
        Ok(contracts) => {
            let paginated: Vec<_> = contracts.into_iter().skip(offset).take(limit).collect();
            Json(paginated).into_response()
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_tokens_handler(
    State(client): State<Arc<RpcClientManager>>,
    Query(pagination): Query<PaginationQuery>,
) -> Response {
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);
    match client.get_tokens().await {
        Ok(tokens) => {
            let paginated: Vec<_> = tokens.into_iter().skip(offset).take(limit).collect();
            Json(paginated).into_response()
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_dex_handler(
    State(client): State<Arc<RpcClientManager>>,
    Query(query): Query<DexReservesQuery>,
) -> Response {
    if let Some(dex) = query.dex.or(query.dexAddress) {
        if !dex.is_empty() {
            match client.get_dex_reserves(&dex).await {
                Ok(res) => return Json(res).into_response(),
                Err(err) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
            }
        }
    }
    match client.get_dexes().await {
        Ok(dexes) => Json(dexes).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

fn check_write_enabled() -> Result<(), Response> {
    let enabled = std::env::var("ZYANYA_EXPLORER_ENABLE_WRITE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !enabled {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "State-changing RPC endpoints are disabled on public explorer deployments. Set ZYANYA_EXPLORER_ENABLE_WRITE=1 to enable."
            })),
        ).into_response())
    } else {
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct DeployContractReq {
    pub bytecode: String,
    pub gas: Option<u64>,
}

pub async fn api_deploy_contract_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<DeployContractReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let gas = payload.gas.unwrap_or(100000);
    match client.deploy_contract(&payload.bytecode, gas).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct InvokeContractReq {
    pub contract_address: Option<String>,
    pub address: Option<String>,
    pub entry_point: Option<u16>,
    pub entryPoint: Option<u16>,
    pub calldata: Option<String>,
    pub gas: Option<u64>,
}

pub async fn api_invoke_contract_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<InvokeContractReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let address = payload.contract_address.or(payload.address).unwrap_or_default();
    if address.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing contract address" }))).into_response();
    }
    let entry_point = payload.entry_point.or(payload.entryPoint).unwrap_or(0);
    let calldata = payload.calldata.unwrap_or_default();
    let gas = payload.gas.unwrap_or(100000);
    match client.invoke_contract(&address, entry_point, &calldata, gas).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct CallContractReq {
    pub contract_address: Option<String>,
    pub address: Option<String>,
    pub calldata: Option<String>,
    pub entry_point: Option<u16>,
    pub entryPoint: Option<u16>,
    pub gas: Option<u64>,
}

pub async fn api_call_contract_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<CallContractReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let address = payload.contract_address.or(payload.address).unwrap_or_default();
    if address.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing contract address" }))).into_response();
    }
    let calldata = payload.calldata.unwrap_or_default();
    let entry_point = payload.entry_point.or(payload.entryPoint).unwrap_or(0);
    let gas = payload.gas.unwrap_or(100000);
    match client.call_contract(&address, &calldata, entry_point, gas).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct DeployTokenReq {
    pub name: Option<String>,
    pub supply: u64,
    pub owner: Option<String>,
    pub gas: Option<u64>,
}

pub async fn api_deploy_token_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<DeployTokenReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let name = payload.name.as_deref().unwrap_or("Token");
    let owner = payload.owner.as_deref().unwrap_or("1");
    let gas = payload.gas.unwrap_or(100000);
    match client.deploy_token(name, payload.supply, owner, gas).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct TokenTransferReq {
    pub token_address: Option<String>,
    pub tokenAddress: Option<String>,
    pub token: Option<String>,
    pub from: Option<String>,
    pub to: String,
    pub amount: u64,
    pub gas: Option<u64>,
}

pub async fn api_token_transfer_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<TokenTransferReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let token = payload.token_address.or(payload.tokenAddress).or(payload.token).unwrap_or_default();
    if token.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing token address" }))).into_response();
    }
    let from = payload.from.as_deref().unwrap_or("1");
    let gas = payload.gas.unwrap_or(100000);
    match client.token_transfer(&token, from, &payload.to, payload.amount, gas).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct SwapOnDexReq {
    pub dex_address: Option<String>,
    pub dexAddress: Option<String>,
    pub dex: Option<String>,
    pub token_in: Option<String>,
    pub tokenIn: Option<String>,
    pub amount_in: Option<u64>,
    pub amountIn: Option<u64>,
    pub gas: Option<u64>,
}

pub async fn api_swap_on_dex_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<SwapOnDexReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let dex = payload.dex_address.or(payload.dexAddress).or(payload.dex).unwrap_or_default();
    if dex.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing DEX address" }))).into_response();
    }
    let token_in = payload.token_in.or(payload.tokenIn).unwrap_or_else(|| "0".to_string());
    let amount_in = payload.amount_in.or(payload.amountIn).unwrap_or(0);
    let gas = payload.gas.unwrap_or(100000);
    match client.swap_on_dex(&dex, &token_in, amount_in, gas).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct CompileContractReq {
    pub source: String,
}

pub async fn api_compile_contract_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<CompileContractReq>,
) -> Response {
    match client.compile_contract(&payload.source) {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

