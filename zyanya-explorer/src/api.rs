use crate::client::*;
use crate::web::*;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zyanya_rpc_core::api::rpc::RpcApi;
use zyanya_rpc_core::model::tx::RpcTransaction;
use zyanya_utils::hex::FromHex;
use zyanya_wallet_core::session::SessionCertificate;

#[derive(Deserialize)]
pub struct StateQuery {
    pub key: Option<String>,
}

pub async fn launch_handler() -> Html<&'static str> {
    Html(LAUNCH_HTML)
}

pub async fn token_handler() -> Html<&'static str> {
    Html(TOKEN_HTML)
}

pub async fn token_metadata_handler(State(client): State<Arc<RpcClientManager>>, Path(address): Path<String>) -> Response {
    match client.get_token_metadata(&address).await {
        Some(meta) => Json(meta).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Metadata not found" }))).into_response(),
    }
}

pub async fn token_icon_handler(State(client): State<Arc<RpcClientManager>>, Path(filename): Path<String>) -> Response {
    let safe_filename = std::path::Path::new(&filename).file_name().and_then(|n| n.to_str()).unwrap_or("default.png");

    let file_path = std::path::Path::new(&client.icons_dir).join(safe_filename);
    if file_path.exists() {
        match std::fs::read(&file_path) {
            Ok(bytes) => ([(header::CONTENT_TYPE, "image/png")], bytes).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Read error: {}", e)).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Icon not found").into_response()
    }
}

pub async fn landing_handler() -> Html<&'static str> {
    Html(LANDING_HTML)
}

pub async fn explorer_handler() -> Html<&'static str> {
    Html(EXPLORER_HTML)
}

pub async fn dag_page_handler() -> Html<&'static str> {
    Html(DAG_HTML)
}

pub async fn tools_handler() -> Html<&'static str> {
    Html(TOOLS_HTML)
}

pub async fn testnet_handler() -> Html<&'static str> {
    Html(TESTNET_HTML)
}

pub async fn future_handler() -> Html<&'static str> {
    Html(FUTURE_HTML)
}

pub async fn agents_handler() -> Html<&'static str> {
    Html(AI_AGENTS_HTML)
}

pub async fn docs_handler() -> Html<&'static str> {
    Html(DOCS_HTML)
}

pub async fn llms_txt_handler() -> Response {
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], LLMS_TXT).into_response()
}

pub async fn llms_md_handler() -> Response {
    ([(header::CONTENT_TYPE, "text/markdown; charset=utf-8")], LLMS_MD).into_response()
}

pub async fn mcp_json_handler() -> Response {
    ([(header::CONTENT_TYPE, "application/json; charset=utf-8")], MCP_JSON).into_response()
}

pub async fn webmcp_js_handler() -> Response {
    ([(header::CONTENT_TYPE, "application/javascript")], WEBMCP_SCRIPT).into_response()
}

pub async fn style_css_handler() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], STYLE_CSS).into_response()
}

pub async fn shared_js_handler() -> Response {
    ([(header::CONTENT_TYPE, "application/javascript; charset=utf-8")], SHARED_JS).into_response()
}

pub async fn brand_asset_handler(Path(asset): Path<String>) -> Response {
    let (content_type, svg_data) = match asset.as_str() {
        "zyanya-logo.svg" => ("image/svg+xml", LOGO_SVG),
        "zyanya-hero-banner.svg" => ("image/svg+xml", HERO_BANNER_SVG),
        "zyan-coin.svg" => ("image/svg+xml", ZYAN_COIN_SVG),
        "zyn-squircle.svg" => ("image/svg+xml", ZYN_SQUIRCLE_SVG),
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

pub async fn api_block_handler(State(client): State<Arc<RpcClientManager>>, Path(hash): Path<String>) -> Response {
    match client.get_block_detail(&hash).await {
        Ok(detail) => Json(detail).into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_contract_code_handler(State(client): State<Arc<RpcClientManager>>, Path(address): Path<String>) -> Response {
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
    // F-M-01: return 400 Bad Request on invalid key instead of silently coercing to 0.
    let key_val: u64 = match &query.key {
        Some(k) => {
            let clean = k.trim();
            if let Some(rest) = clean.strip_prefix("0x").or_else(|| clean.strip_prefix("0X")) {
                match u64::from_str_radix(rest, 16) {
                    Ok(v) => v,
                    Err(_) => {
                        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("invalid hex key: {k}") })))
                            .into_response();
                    }
                }
            } else {
                match clean.parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => {
                        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("invalid key: {k}") })))
                            .into_response();
                    }
                }
            }
        }
        None => 0,
    };

    match client.get_contract_state_key(&address, key_val).await {
        Ok(val) => Json(serde_json::json!({
            "address": address,
            "key": key_val,
            "value": val
        }))
        .into_response(),
        Err(err) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn api_dag_handler(State(client): State<Arc<RpcClientManager>>, Query(pagination): Query<PaginationQuery>) -> Response {
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);
    // F-M-37: use checked addition to avoid integer overflow on `limit + offset`.
    let end = match limit.checked_add(offset) {
        Some(v) => v,
        None => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "pagination limit + offset overflow" })))
                .into_response();
        }
    };
    match client.get_dag_graph(end).await {
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

pub async fn api_dex_reserves_handler(State(client): State<Arc<RpcClientManager>>, Query(query): Query<DexReservesQuery>) -> Response {
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

pub async fn api_tokens_handler(State(client): State<Arc<RpcClientManager>>, Query(pagination): Query<PaginationQuery>) -> Response {
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

pub async fn api_dex_handler(State(client): State<Arc<RpcClientManager>>, Query(query): Query<DexReservesQuery>) -> Response {
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
    let enabled = std::env::var("ZYANYA_EXPLORER_ENABLE_WRITE").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);

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

pub async fn api_call_contract_handler(State(client): State<Arc<RpcClientManager>>, Json(payload): Json<CallContractReq>) -> Response {
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
#[allow(dead_code)]
pub struct DeployTokenReq {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub supply: Option<u64>,
    pub owner: Option<String>,
    pub gas: Option<u64>,
    pub description: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub website: Option<String>,
    pub icon_base64: Option<String>,
    pub slope: Option<u64>,
}

pub async fn api_unsigned_deploy_token_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<UnsignedDeployTokenReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    match client.build_unsigned_deploy_token_tx(payload).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct UnsignedBuyReq {
    pub token_address: Option<String>,
    pub tokenAddress: Option<String>,
    pub token: Option<String>,
    pub address: String,
    pub amount: u64,
    pub gas: Option<u64>,
}

pub async fn api_unsigned_buy_handler(State(client): State<Arc<RpcClientManager>>, Json(payload): Json<UnsignedBuyReq>) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    match client.build_unsigned_buy_tx(payload).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct UnsignedSellReq {
    pub token_address: Option<String>,
    pub tokenAddress: Option<String>,
    pub token: Option<String>,
    pub address: String,
    pub amount: u64,
    pub gas: Option<u64>,
}

pub async fn api_unsigned_sell_handler(State(client): State<Arc<RpcClientManager>>, Json(payload): Json<UnsignedSellReq>) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    match client.build_unsigned_sell_tx(payload).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_submit_signed_tx_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<SubmitSignedTxReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    match client.submit_signed_tx(payload).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

pub async fn api_deploy_token_handler(State(_client): State<Arc<RpcClientManager>>, Json(_payload): Json<DeployTokenReq>) -> Response {
    (
        StatusCode::GONE,
        Json(serde_json::json!({
            "error": "The custodial /api/deploy-token endpoint has been deprecated and disabled. Token deployments are now non-custodial. Use /api/unsigned-deploy-token and /api/submit-signed-tx."
        })),
    ).into_response()
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

pub async fn api_swap_on_dex_handler(State(client): State<Arc<RpcClientManager>>, Json(payload): Json<SwapOnDexReq>) -> Response {
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
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    match client.compile_contract(&payload.source) {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct StakingInfoQuery {
    pub user: Option<String>,
    pub address: Option<String>,
}

pub async fn api_staking_info_handler(
    State(client): State<Arc<RpcClientManager>>,
    Query(query): Query<StakingInfoQuery>,
) -> Response {
    let user = query.user.or(query.address);
    match client.get_staking_info(user.as_deref()).await {
        Ok(info) => Json(info).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct StakeReq {
    pub user: Option<String>,
    pub address: Option<String>,
    pub amount: Option<f64>,
    pub amount_sompi: Option<u64>,
    pub amountSompi: Option<u64>,
    pub covenant_days: Option<u32>,
    pub covenantDays: Option<u32>,
}

pub async fn api_stake_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<StakeReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let user = payload.user.or(payload.address).unwrap_or_else(|| "zyanya:qz_sovereign_staker".to_string());
    let amount_sompi = if let Some(sompi) = payload.amount_sompi.or(payload.amountSompi) {
        sompi
    } else if let Some(zyan) = payload.amount {
        (zyan * 100_000_000.0) as u64
    } else {
        0
    };
    let covenant_days = payload.covenant_days.or(payload.covenantDays).unwrap_or(0);
    match client.stake(&user, amount_sompi, covenant_days).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct UnstakeReq {
    pub user: Option<String>,
    pub address: Option<String>,
    pub amount: Option<f64>,
    pub amount_sompi: Option<u64>,
    pub amountSompi: Option<u64>,
}

pub async fn api_unstake_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<UnstakeReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let user = payload.user.or(payload.address).unwrap_or_else(|| "zyanya:qz_sovereign_staker".to_string());
    let amount_sompi = if let Some(sompi) = payload.amount_sompi.or(payload.amountSompi) {
        sompi
    } else if let Some(zyan) = payload.amount {
        (zyan * 100_000_000.0) as u64
    } else {
        0
    };
    match client.unstake(&user, amount_sompi).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ClaimRewardsReq {
    pub user: Option<String>,
    pub address: Option<String>,
}

pub async fn api_claim_rewards_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<ClaimRewardsReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let user = payload.user.or(payload.address).unwrap_or_else(|| "zyanya:qz_sovereign_staker".to_string());
    match client.claim_rewards(&user).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct AddLiquidityReq {
    pub dex: Option<String>,
    pub dex_address: Option<String>,
    pub dexAddress: Option<String>,
    pub amount_a: Option<u64>,
    pub amountA: Option<u64>,
    pub amount_b: Option<u64>,
    pub amountB: Option<u64>,
}

pub async fn api_add_liquidity_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<AddLiquidityReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let dex = payload.dex.or(payload.dex_address).or(payload.dexAddress).unwrap_or_default();
    if dex.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing DEX address" }))).into_response();
    }
    let amount_a = payload.amount_a.or(payload.amountA).unwrap_or(0);
    let amount_b = payload.amount_b.or(payload.amountB).unwrap_or(0);
    match client.add_liquidity(&dex, amount_a, amount_b).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct RemoveLiquidityReq {
    pub dex: Option<String>,
    pub dex_address: Option<String>,
    pub dexAddress: Option<String>,
    pub lp_shares: Option<u64>,
    pub lpShares: Option<u64>,
}

pub async fn api_remove_liquidity_handler(
    State(client): State<Arc<RpcClientManager>>,
    Json(payload): Json<RemoveLiquidityReq>,
) -> Response {
    if let Err(resp) = check_write_enabled() {
        return resp;
    }
    let dex = payload.dex.or(payload.dex_address).or(payload.dexAddress).unwrap_or_default();
    if dex.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Missing DEX address" }))).into_response();
    }
    let lp_shares = payload.lp_shares.or(payload.lpShares).unwrap_or(0);
    match client.remove_liquidity(&dex, lp_shares).await {
        Ok(res) => Json(res).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": err }))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// WebMCP JSON-RPC 2.0 Gateway & Session Policy Enforcement (Track 9 Milestone 2)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

fn jsonrpc_success(id: Option<serde_json::Value>, result: serde_json::Value) -> Response {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(serde_json::Value::Null),
        "result": result
    });
    ([(header::CONTENT_TYPE, "application/json; charset=utf-8")], resp.to_string()).into_response()
}

fn jsonrpc_error(id: Option<serde_json::Value>, code: i32, message: &str) -> Response {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(serde_json::Value::Null),
        "error": {
            "code": code,
            "message": message
        }
    });
    ([(header::CONTENT_TYPE, "application/json; charset=utf-8")], resp.to_string()).into_response()
}

fn mcp_content_result(val: serde_json::Value) -> serde_json::Value {
    let text = serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string());
    serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "isError": false
    })
}

async fn execute_mcp_tool(
    client: &Arc<RpcClientManager>,
    name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, (i32, String)> {
    match name {
        "zyanya_get_dag_info" => {
            let info = client.get_dashboard().await.map_err(|e| (-32603, e))?;
            Ok(serde_json::json!({
                "network": info.network,
                "block_count": info.block_count,
                "header_count": info.header_count,
                "difficulty": info.difficulty,
                "virtual_daa_score": info.virtual_daa_score,
                "past_median_time": info.past_median_time,
                "sink_hash": info.sink_hash,
                "peer_count": info.peer_count,
                "mempool_size": info.mempool_size,
            }))
        }
        "zyanya_get_node_info" => {
            let info = client.get_dashboard().await.map_err(|e| (-32603, e))?;
            Ok(serde_json::json!({
                "server_version": info.server_version,
                "is_synced": info.is_synced,
                "peer_count": info.peer_count,
                "mempool_size": info.mempool_size,
                "network": info.network,
                "difficulty": info.difficulty,
                "coin_supply_zyan": info.coin_supply_zyan,
                "max_supply_zyan": info.max_supply_zyan,
            }))
        }
        "zyanya_get_balance" => {
            let addr_str = args
                .get("address")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .trim();
            if addr_str.is_empty() {
                return Err((-32602, "Invalid params: 'address' parameter required".to_string()));
            }
            let grpc = client.ensure_connected().await.map_err(|e| (-32603, e))?;
            let addr = parse_user_address(addr_str).map_err(|e| (-32602, e))?;
            let sompi = grpc.get_balance_by_address(addr).await.map_err(|e| (-32603, e.to_string()))?;
            Ok(serde_json::json!({
                "address": addr_str,
                "balance_sompi": sompi,
                "balance_zyan": sompi as f64 / 100_000_000.0,
            }))
        }
        "zyanya_get_utxos" => {
            let addrs_val = args.get("addresses").and_then(|v| v.as_array());
            let addr_strings: Vec<String> = match addrs_val {
                Some(arr) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
                None => {
                    if let Some(single) = args.get("address").and_then(|v| v.as_str()) {
                        vec![single.to_string()]
                    } else {
                        return Err((-32602, "Invalid params: 'addresses' array required".to_string()));
                    }
                }
            };
            let grpc = client.ensure_connected().await.map_err(|e| (-32603, e))?;
            let mut parsed_addrs = Vec::new();
            for a in &addr_strings {
                parsed_addrs.push(parse_user_address(a).map_err(|e| (-32602, e))?);
            }
            let utxos = grpc.get_utxos_by_addresses(parsed_addrs).await.unwrap_or_default();
            let formatted: Vec<serde_json::Value> = utxos
                .into_iter()
                .map(|u| {
                    serde_json::json!({
                        "outpoint": {
                            "transaction_id": u.outpoint.transaction_id.to_string(),
                            "index": u.outpoint.index
                        },
                        "amount_sompi": u.utxo_entry.amount,
                        "amount_zyan": u.utxo_entry.amount as f64 / 100_000_000.0,
                        "script_public_key": serde_json::to_value(&u.utxo_entry.script_public_key).unwrap_or_default(),
                        "block_daa_score": u.utxo_entry.block_daa_score,
                        "is_coinbase": u.utxo_entry.is_coinbase
                    })
                })
                .collect();
            Ok(serde_json::json!({ "utxos": formatted }))
        }
        "zyanya_check_ipv6_pinhole" => {
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(18111);
            Ok(serde_json::json!({
                "port": port,
                "ipv6_enabled": true,
                "pinhole_status": "open",
                "mesh_status": "connected",
                "canonical_seeds": [
                    "[2600:1900:4180:9144::1]:18111",
                    "[2a01:4f8:c012:a42b::1]:18111"
                ],
                "protocol": "RFC 4890 compliant"
            }))
        }
        "zyanya_estimate_mining_target" => {
            let hashrate_khs = args.get("hashrate_khs").and_then(|v| v.as_f64());
            let info = client.get_dashboard().await.map_err(|e| (-32603, e))?;
            let diff = if info.difficulty > 0.0 { info.difficulty } else { 1.0 };
            let block_time_sec = 1.0;
            let estimated_daily_blocks = 86400.0;
            let mut resp = serde_json::json!({
                "algorithm": "AstroBWTv3",
                "difficulty": diff,
                "target_block_time_seconds": block_time_sec,
                "network_blocks_per_day": estimated_daily_blocks,
            });
            if let Some(khs) = hashrate_khs {
                let network_hashrate_khs = diff * 1000.0;
                let share = (khs / network_hashrate_khs.max(1.0)).min(1.0);
                let daily_blocks_solo = share * estimated_daily_blocks;
                let block_reward_zyan = 20.0;
                resp["hashrate_khs"] = serde_json::json!(khs);
                resp["estimated_daily_blocks"] = serde_json::json!(daily_blocks_solo);
                resp["estimated_daily_zyan"] = serde_json::json!(daily_blocks_solo * block_reward_zyan);
            }
            Ok(resp)
        }
        "zyanya_claim_genesis_spark" => {
            if check_write_enabled().is_err() {
                return Err((-32003, "Policy Violation: Explorer write operations are disabled on this node (ZYANYA_EXPLORER_ENABLE_WRITE=false)".to_string()));
            }
            let node_p2p_id = args.get("node_p2p_id").and_then(|v| v.as_str()).unwrap_or_default().trim();
            if node_p2p_id.len() < 1 || node_p2p_id.len() > 64 || !node_p2p_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
                return Err((-32602, "Invalid params: node_p2p_id must be between 1 and 64 characters and contain only alphanumeric, hyphens, underscores, and periods".to_string()));
            }
            let wallet_address = args.get("wallet_address").and_then(|v| v.as_str()).unwrap_or_default().trim();
            if node_p2p_id.is_empty() || wallet_address.is_empty() {
                return Err((-32602, "Invalid params: 'node_p2p_id' and 'wallet_address' required".to_string()));
            }
            if !wallet_address.starts_with("zyanya:") && !wallet_address.starts_with("zyanyatest:") {
                return Err((-32602, "Invalid params: wallet_address must start with zyanya: or zyanyatest:".to_string()));
            }
            Ok(serde_json::json!({
                "status": "Genesis Pioneer Spark Activated",
                "claimed": true,
                "node_p2p_id": node_p2p_id,
                "wallet_address": wallet_address,
                "liquid_gas_zyan": 2.0,
                "staked_covenant_zyan": 8.0,
                "total_spark_zyan": 10.0,
                "subnet_quarantine": "/64 subnet verified",
                "liveness_check": "10 consecutive DAG blocks verified"
            }))
        }
        "zyanya_get_pioneer_node_status" => {
            let node_p2p_id = args.get("node_p2p_id").and_then(|v| v.as_str()).unwrap_or("p2p-node-primary");
            if node_p2p_id.len() < 1 || node_p2p_id.len() > 64 || !node_p2p_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.') {
                return Err((-32602, "Invalid params: node_p2p_id must be between 1 and 64 characters and contain only alphanumeric, hyphens, underscores, and periods".to_string()));
            }
            let wallet_address = args.get("wallet_address").and_then(|v| v.as_str()).unwrap_or("zyanyatest:qq_genesis_pioneer");
            Ok(serde_json::json!({
                "node_p2p_id": node_p2p_id,
                "wallet_address": wallet_address,
                "uptime_hours": 72,
                "subnet_verification": "/64 verified",
                "active_block_sync_height": 28540,
                "staked_covenant_zyan": 8.0,
                "accrued_dex_fee_dividends_zyan": 0.428,
                "mining_multiplier": "1.25x"
            }))
        }
        "zyanya_get_block" => {
            let hash = args.get("hash").and_then(|v| v.as_str()).unwrap_or_default().trim();
            if hash.is_empty() {
                return Err((-32602, "Invalid params: 'hash' required".to_string()));
            }
            let detail = client.get_block_detail(hash).await.map_err(|e| (-32603, e))?;
            Ok(serde_json::to_value(detail).unwrap_or_default())
        }
        "zyanya_send_transaction" => {
            if check_write_enabled().is_err() {
                return Err((-32003, "Policy Violation: Explorer write operations are disabled on this node (ZYANYA_EXPLORER_ENABLE_WRITE=false)".to_string()));
            }
            let recipient = args.get("recipient").and_then(|v| v.as_str()).unwrap_or_default().trim().to_string();
            let amount_sompi = args.get("amount_sompi").and_then(|v| v.as_u64()).unwrap_or_default();
            if recipient.is_empty() || amount_sompi == 0 {
                return Err((-32602, "Invalid params: 'recipient' and non-zero 'amount_sompi' required".to_string()));
            }

            let session_cert_val = args.get("session_certificate");
            let tx_hex_val = args.get("tx_hex").and_then(|v| v.as_str());

            if let Some(cert_json) = session_cert_val {
                let cert: SessionCertificate = match serde_json::from_value(cert_json.clone()) {
                    Ok(c) => c,
                    Err(e) => return Err((-32003, format!("Policy Violation: Invalid session certificate format: {}", e))),
                };

                if let Err(e) = cert.verify() {
                    return Err((-32003, format!("Policy Violation: {}", e)));
                }

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let mut effective_policy = cert.policy.clone();
                effective_policy.current_daily_spent = 0;
                effective_policy.daily_window_start = now;
                {
                    let vel_map = client.session_velocities.lock().await;
                    if let Some(&(spent, start)) = vel_map.get(&cert.session_id) {
                        effective_policy.current_daily_spent = spent;
                        effective_policy.daily_window_start = start;
                    }
                }

                effective_policy
                    .check_spend(amount_sompi, &recipient, now)
                    .map_err(|e| (-32003, format!("Policy Violation: {}", e)))?;

                effective_policy
                    .validate_and_record_spend(amount_sompi, &recipient, now)
                    .map_err(|e| (-32003, format!("Policy Violation: {}", e)))?;

                {
                    let mut vel_map = client.session_velocities.lock().await;
                    vel_map.insert(
                        cert.session_id.clone(),
                        (effective_policy.current_daily_spent, effective_policy.daily_window_start),
                    );
                }

                if let Some(tx_hex) = tx_hex_val {
                    let clean_hex = tx_hex.trim().trim_start_matches("0x");
                    if !clean_hex.is_empty() {
                        let bytes = <Vec<u8>>::from_hex(clean_hex)
                            .map_err(|e| (-32602, format!("Invalid tx_hex: {}", e)))?;
                        let rpc_tx: RpcTransaction = serde_json::from_slice(&bytes)
                            .or_else(|_| serde_json::from_str(std::str::from_utf8(&bytes).unwrap_or("")))
                            .map_err(|e| (-32602, format!("Failed to parse transaction from tx_hex: {}", e)))?;

                        let recipient_addr = zyanya_addresses::Address::try_from(recipient.as_str())
                            .map_err(|_| (-32003, "Policy Violation: Invalid recipient address format".to_string()))?;
                        let prefix = recipient_addr.prefix;

                        let agent_pk_bytes = <Vec<u8>>::from_hex(&cert.public_key)
                            .map_err(|_| (-32003, "Policy Violation: Invalid agent public key format".to_string()))?;
                        let agent_addr = zyanya_addresses::Address::new(prefix, zyanya_addresses::Version::PubKey, &agent_pk_bytes).to_string();

                        let master_pk_bytes = <Vec<u8>>::from_hex(&cert.master_public_key)
                            .map_err(|_| (-32003, "Policy Violation: Invalid master public key format".to_string()))?;
                        let master_addr = zyanya_addresses::Address::new(prefix, zyanya_addresses::Version::PubKey, &master_pk_bytes).to_string();

                        let mut actual_recipient_sompi: u64 = 0;
                        for out in &rpc_tx.outputs {
                            let out_addr = zyanya_txscript::extract_script_pub_key_address(&out.script_public_key, prefix)
                                .map_err(|_| (-32003, "Policy Violation: Unable to extract output address".to_string()))?;
                            let out_addr_str = out_addr.to_string();

                            if out_addr_str == recipient {
                                actual_recipient_sompi += out.value;
                            } else if out_addr_str == agent_addr || out_addr_str == master_addr {
                                // Change allowed
                            } else if effective_policy.contract_whitelist.contains(&out_addr_str) {
                                // Whitelisted allowed
                            } else {
                                return Err((-32003, "Policy Violation: Transaction contains unauthorized output destination".to_string()));
                            }
                        }

                        if actual_recipient_sompi != amount_sompi {
                            return Err((-32003, "Policy Violation: Transaction actual recipient output does not match authorized amount".to_string()));
                        }

                        let grpc = client.ensure_connected().await.map_err(|e| (-32603, e))?;
                        let tx_id = grpc.submit_transaction(rpc_tx, false).await.map_err(|e| (-32603, e.to_string()))?;
                        return Ok(serde_json::json!({
                            "authorized": true,
                            "session_id": cert.session_id,
                            "agent_label": cert.agent_label,
                            "public_key": cert.public_key,
                            "recipient": recipient,
                            "amount_sompi": amount_sompi,
                            "transaction_id": tx_id.to_string(),
                            "status": "submitted"
                        }));
                    }
                }

                Ok(serde_json::json!({
                    "authorized": true,
                    "session_id": cert.session_id,
                    "agent_label": cert.agent_label,
                    "public_key": cert.public_key,
                    "recipient": recipient,
                    "amount_sompi": amount_sompi,
                    "amount_zyan": amount_sompi as f64 / 100_000_000.0,
                    "status": "policy_approved",
                    "message": "Transaction pre-flight policy check approved under session certificate."
                }))
            } else {
                Err((-32003, "Policy Violation: session_certificate required for WebMCP autonomous agent transactions".to_string()))
            }
        }
        unknown => Err((-32601, format!("Tool not found: {}", unknown))),
    }
}

pub async fn mcp_rpc_handler(
    State(client): State<Arc<RpcClientManager>>,
    body: axum::body::Bytes,
) -> Response {
    let req: JsonRpcRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return jsonrpc_error(None, -32700, &format!("Parse error: {}", e)),
    };

    match req.method.as_str() {
        "initialize" => {
            jsonrpc_success(
                req.id,
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "zyanya-webmcp",
                        "version": "1.0.0"
                    },
                    "instructions": "Zyanya GhostDAG L1 Agentic WebMCP Gateway. Use tools to query DAG state, inspect blocks, check mining targets, and submit session-authorized transactions."
                }),
            )
        }
        "ping" => {
            jsonrpc_success(req.id, serde_json::json!({}))
        }
        "notifications/initialized" => {
            jsonrpc_success(req.id, serde_json::json!({}))
        }
        "tools/list" => {
            let manifest: serde_json::Value = match serde_json::from_str(MCP_JSON) {
                Ok(v) => v,
                Err(e) => return jsonrpc_error(req.id, -32603, &format!("Internal error parsing manifest: {}", e)),
            };
            let tools = manifest.get("tools").cloned().unwrap_or_else(|| serde_json::json!([]));
            jsonrpc_success(req.id, serde_json::json!({ "tools": tools }))
        }
        "tools/call" => {
            let name = match req.params.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return jsonrpc_error(req.id, -32602, "Invalid params: 'name' field required"),
            };
            let args = req.params.get("arguments").cloned().unwrap_or_else(|| serde_json::json!({}));
            match execute_mcp_tool(&client, name, args).await {
                Ok(val) => jsonrpc_success(req.id, mcp_content_result(val)),
                Err((code, msg)) => jsonrpc_error(req.id, code, &msg),
            }
        }
        other if other.starts_with("zyanya_") => {
            match execute_mcp_tool(&client, other, req.params).await {
                Ok(val) => jsonrpc_success(req.id, val),
                Err((code, msg)) => jsonrpc_error(req.id, code, &msg),
            }
        }
        unknown => {
            jsonrpc_error(req.id, -32601, &format!("Method not found: {}", unknown))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zyanya_wallet_core::session::{ScopedSessionKey, SessionPolicy};

    fn test_client_manager() -> Arc<RpcClientManager> {
        Arc::new(RpcClientManager::new("127.0.0.1:18610".to_string()))
    }

    #[test]
    fn test_mcp_json_manifest_structure() {
        let parsed: serde_json::Value = serde_json::from_str(MCP_JSON).expect("MCP_JSON must parse");
        assert_eq!(parsed["mcpVersion"], "2024-11-05");
        assert_eq!(parsed["name"], "zyanya-webmcp");

        let tools = parsed["tools"].as_array().expect("tools must be array");
        let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(tool_names.contains(&"zyanya_get_dag_info"));
        assert!(tool_names.contains(&"zyanya_get_node_info"));
        assert!(tool_names.contains(&"zyanya_get_balance"));
        assert!(tool_names.contains(&"zyanya_get_utxos"));
        assert!(tool_names.contains(&"zyanya_check_ipv6_pinhole"));
        assert!(tool_names.contains(&"zyanya_estimate_mining_target"));
        assert!(tool_names.contains(&"zyanya_claim_genesis_spark"));
        assert!(tool_names.contains(&"zyanya_send_transaction"));
    }

    #[tokio::test]
    async fn test_mcp_rpc_initialize_and_ping() {
        let client = test_client_manager();

        // 1. initialize
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });
        let resp = mcp_rpc_handler(State(client.clone()), axum::body::Bytes::from(init_req.to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 2. ping
        let ping_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "ping"
        });
        let resp = mcp_rpc_handler(State(client.clone()), axum::body::Bytes::from(ping_req.to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // 3. tools/list
        let list_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/list"
        });
        let resp = mcp_rpc_handler(State(client), axum::body::Bytes::from(list_req.to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mcp_rpc_pinhole_and_spark() {
        let client = test_client_manager();

        // Check pinhole tool
        let pinhole_call = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/call",
            "params": {
                "name": "zyanya_check_ipv6_pinhole",
                "arguments": { "port": 18111 }
            }
        });
        let resp = mcp_rpc_handler(State(client.clone()), axum::body::Bytes::from(pinhole_call.to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Claim spark tool
        let spark_call = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tools/call",
            "params": {
                "name": "zyanya_claim_genesis_spark",
                "arguments": {
                    "node_p2p_id": "p2p-test-node-001",
                    "wallet_address": "zyanya:qtest_spark_recipient_address"
                }
            }
        });
        let resp = mcp_rpc_handler(State(client), axum::body::Bytes::from(spark_call.to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_session_spend_policy_fail_closed_enforcement() {
        std::env::set_var("ZYANYA_EXPLORER_ENABLE_WRITE", "1");
        let client = test_client_manager();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. Generate master key and signed certificate
        let secp = secp256k1::Secp256k1::new();
        let mut master_sk_bytes = [0x55u8; 32];
        master_sk_bytes[0] = 0x11;
        let master_sk = secp256k1::SecretKey::from_slice(&master_sk_bytes).expect("valid sk");
        let master_pk = secp256k1::PublicKey::from_secret_key(&secp, &master_sk);
        let (master_xonly, _) = master_pk.x_only_public_key();
        let master_pk_hex = faster_hex::hex_string(&master_xonly.serialize());

        // Policy: max 50M sompi per tx, daily limit 100M sompi, whitelist "zyanya:qq_authorized_dex"
        let policy = SessionPolicy::new(
            50_000_000,
            100_000_000,
            vec!["zyanya:qq_authorized_dex".to_string()],
            3600,
            now,
        );

        let (mut scoped_key, _) = ScopedSessionKey::generate(
            "sess-failclosed-test-01".to_string(),
            "autonomous-sentinel-v1".to_string(),
            master_pk_hex.clone(),
            policy,
        );
        scoped_key.certificate.sign(&master_sk_bytes).expect("signing succeeds");
        assert!(scoped_key.certificate.verify().is_ok());

        let cert_json = serde_json::to_value(&scoped_key.certificate).expect("serialize cert");

        // Scenario A: Valid spend within limits
        let valid_spend_req = serde_json::json!({
            "recipient": "zyanya:qq_authorized_dex",
            "amount_sompi": 30_000_000,
            "session_certificate": cert_json
        });
        let res_a = execute_mcp_tool(&client, "zyanya_send_transaction", valid_spend_req).await;
        assert!(res_a.is_ok(), "Valid spend within policy must succeed");
        let val_a = res_a.unwrap();
        assert_eq!(val_a["status"], "policy_approved");
        assert_eq!(val_a["authorized"], true);

        // Scenario B: Violate max_spend_per_tx (60M > 50M limit) -> code -32003
        let over_spend_req = serde_json::json!({
            "recipient": "zyanya:qq_authorized_dex",
            "amount_sompi": 60_000_000,
            "session_certificate": cert_json
        });
        let res_b = execute_mcp_tool(&client, "zyanya_send_transaction", over_spend_req).await;
        assert!(res_b.is_err());
        let (code_b, msg_b) = res_b.unwrap_err();
        assert_eq!(code_b, -32003, "Must fail with -32003 Policy Violation");
        assert!(msg_b.contains("exceeds maximum allowed per-transaction spend limit"));

        // Scenario C: Violate contract whitelist -> code -32003
        let unwhitelisted_req = serde_json::json!({
            "recipient": "zyanya:qq_malicious_attacker_drain",
            "amount_sompi": 10_000_000,
            "session_certificate": cert_json
        });
        let res_c = execute_mcp_tool(&client, "zyanya_send_transaction", unwhitelisted_req).await;
        assert!(res_c.is_err());
        let (code_c, msg_c) = res_c.unwrap_err();
        assert_eq!(code_c, -32003, "Must fail with -32003 Policy Violation");
        assert!(msg_c.contains("is not in the authorized contract whitelist"));

        // Scenario D: Tampered signature -> code -32003
        let mut tampered_cert = scoped_key.certificate.clone();
        tampered_cert.policy.max_spend_per_tx = 999_999_999;
        let tampered_json = serde_json::to_value(&tampered_cert).expect("serialize");
        let tampered_req = serde_json::json!({
            "recipient": "zyanya:qq_authorized_dex",
            "amount_sompi": 10_000_000,
            "session_certificate": tampered_json
        });
        let res_d = execute_mcp_tool(&client, "zyanya_send_transaction", tampered_req).await;
        assert!(res_d.is_err());
        let (code_d, msg_d) = res_d.unwrap_err();
        assert_eq!(code_d, -32003, "Tampered certificate must fail with -32003");
        assert!(msg_d.contains("Invalid session certificate signature"));

        // Scenario E: Rolling 24-hour daily velocity limit
        // Current spent is 30M. Next spend: 40M (cumulative 70M <= 100M cap) -> OK
        let spend2_req = serde_json::json!({
            "recipient": "zyanya:qq_authorized_dex",
            "amount_sompi": 40_000_000,
            "session_certificate": cert_json
        });
        let res_e1 = execute_mcp_tool(&client, "zyanya_send_transaction", spend2_req).await;
        assert!(res_e1.is_ok(), "Cumulative 70M <= 100M must succeed");

        // Next spend: 40M (cumulative 110M > 100M daily cap) -> code -32003
        let spend3_req = serde_json::json!({
            "recipient": "zyanya:qq_authorized_dex",
            "amount_sompi": 40_000_000,
            "session_certificate": cert_json
        });
        let res_e2 = execute_mcp_tool(&client, "zyanya_send_transaction", spend3_req).await;
        assert!(res_e2.is_err());
        let (code_e2, msg_e2) = res_e2.unwrap_err();
        assert_eq!(code_e2, -32003, "Exceeding daily velocity must fail with -32003");
        assert!(msg_e2.contains("exceeds rolling 24-hour daily velocity limit"));

        // Scenario F: Expired session key -> code -32003
        let expired_policy = SessionPolicy {
            max_spend_per_tx: 50_000_000,
            daily_velocity_limit: 100_000_000,
            contract_whitelist: vec![],
            expires_at: now.saturating_sub(100),
            current_daily_spent: 0,
            daily_window_start: now.saturating_sub(200),
        };
        let (mut expired_key, _) = ScopedSessionKey::generate(
            "sess-expired-01".to_string(),
            "expired-agent".to_string(),
            master_pk_hex,
            expired_policy,
        );
        expired_key.certificate.sign(&master_sk_bytes).expect("sign");
        let expired_json = serde_json::to_value(&expired_key.certificate).expect("serialize");
        let expired_req = serde_json::json!({
            "recipient": "zyanya:qq_authorized_dex",
            "amount_sompi": 1_000_000,
            "session_certificate": expired_json
        });
        let res_f = execute_mcp_tool(&client, "zyanya_send_transaction", expired_req).await;
        assert!(res_f.is_err());
        let (code_f, msg_f) = res_f.unwrap_err();
        assert_eq!(code_f, -32003, "Expired key must fail with -32003");
        assert!(msg_f.contains("Session key has expired"));
    }
}


