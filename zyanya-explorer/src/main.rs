mod api;
mod client;
mod web;

use axum::{routing::{get, post}, Router};
use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::api::*;
use crate::client::RpcClientManager;

#[derive(Parser, Debug)]
#[command(
    name = "zyanya-explorer",
    author = "Zyanya Developers",
    version,
    about = "Zyanya Block Explorer and IPv6-only Website Server"
)]
struct Cli {
    /// Server listen address (IPv6 ONLY, e.g. [::]:8098)
    #[arg(short, long, default_value = "[::]:8098")]
    listen: String,

    /// Zyanya node gRPC server address (e.g. 127.0.0.1:18610 or [::1]:18610)
    #[arg(short, long, default_value = "127.0.0.1:18610")]
    rpcserver: String,
}

fn create_ipv6_only_listener(addr_str: &str) -> Result<TcpListener, Box<dyn std::error::Error>> {
    let addr: SocketAddr = addr_str.parse()?;
    if !addr.is_ipv6() {
        return Err("Listen address MUST be an IPv6 address (e.g., [::]:8098) to enforce IPv6-only positioning!".into());
    }

    let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_only_v6(true)?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(1024)?;

    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(std_listener)?;
    Ok(listener)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("===============================================================");
    println!("  ZYANYA BLOCK EXPLORER + IPv6-ONLY WEBSITE (PHASE 3)");
    println!("  The Ghost in the IPv6 Machine • Forever. Always.");
    println!("===============================================================");
    println!(" [!] Enforcing IPv6-only socket binding on {}", cli.listen);
    println!(" [*] Connecting to Zyanya Node gRPC at {}", cli.rpcserver);

    let listener = match create_ipv6_only_listener(&cli.listen) {
        Ok(l) => {
            println!(" [✓] Successfully bound socket [::]:8098 (IPV6_V6ONLY=true, IPv4 disabled)");
            l
        }
        Err(err) => {
            eprintln!(" [✗] Failed to bind IPv6-only socket: {}", err);
            return Err(err);
        }
    };

    let client_mgr = Arc::new(RpcClientManager::new(cli.rpcserver.clone()));

    // Test connection
    match client_mgr.ensure_connected().await {
        Ok(_) => println!(" [✓] Zyanya Node gRPC connected successfully"),
        Err(e) => eprintln!(" [!] Warning: Node RPC connection pending ({})", e),
    }

    let app = Router::new()
        .route("/", get(landing_handler))
        .route("/explorer", get(explorer_handler))
        .route("/dag", get(dag_page_handler))
        .route("/launch", get(launch_handler))
        .route("/token/:address", get(token_handler))
        .route("/token-icons/:filename", get(token_icon_handler))
        .route("/tools", get(tools_handler))
        .route("/testnet", get(testnet_handler))
        .route("/future", get(future_handler))
        .route("/agents", get(agents_handler))
        .route("/docs", get(docs_handler))
        .route("/llms.txt", get(llms_txt_handler))
        .route("/llms.md", get(llms_md_handler))
        .route("/webmcp.js", get(webmcp_js_handler))
        .route("/brand/:asset", get(brand_asset_handler))
        .route("/api/info", get(api_info_handler))
        .route("/api/blocks", get(api_blocks_handler))
        .route("/api/block/:hash", get(api_block_handler))
        .route("/api/contract/:address/code", get(api_contract_code_handler))
        .route("/api/contract/:address/state", get(api_contract_state_handler))
        .route("/api/contracts", get(api_contracts_handler))
        .route("/api/tokens", get(api_tokens_handler))
        .route("/api/token/:address/metadata", get(token_metadata_handler))
        .route("/api/dex", get(api_dex_handler).post(api_dex_reserves_post_handler))
        .route("/api/dexes", get(api_dex_handler))
        .route("/api/dag", get(api_dag_handler))
        .route("/api/token-balance", get(api_token_balance_handler).post(api_token_balance_post_handler))
        .route("/api/dex-reserves", get(api_dex_reserves_handler).post(api_dex_reserves_post_handler))
        .route("/api/deploy-contract", post(api_deploy_contract_handler))
        .route("/api/invoke-contract", post(api_invoke_contract_handler))
        .route("/api/call-contract", post(api_call_contract_handler))
        .route("/api/deploy-token", post(api_deploy_token_handler))
        .route("/api/unsigned-deploy-token", post(api_unsigned_deploy_token_handler))
        .route("/api/unsigned-buy", post(api_unsigned_buy_handler))
        .route("/api/unsigned-sell", post(api_unsigned_sell_handler))
        .route("/api/submit-signed-tx", post(api_submit_signed_tx_handler))
        .route("/api/token-transfer", post(api_token_transfer_handler))
        .route("/api/swap-on-dex", post(api_swap_on_dex_handler))
        .route("/api/compile-contract", post(api_compile_contract_handler))
        .with_state(client_mgr);

    println!(" [*] Server running at http://{}/", cli.listen);
    println!("===============================================================");

    axum::serve(listener, app).await?;

    Ok(())
}
