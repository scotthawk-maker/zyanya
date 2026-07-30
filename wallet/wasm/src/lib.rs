use zyanya_cli_lib::zyanya_cli;
use wasm_bindgen::prelude::*;
use workflow_terminal::Options;
use workflow_terminal::Result;

#[wasm_bindgen]
pub async fn load_zyanya_wallet_cli() -> Result<()> {
    let options = Options { ..Options::default() };
    zyanya_cli(options, None).await?;
    Ok(())
}
