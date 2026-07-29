use zyanya_cli_lib::{zyanya_cli, TerminalOptions};

#[tokio::main]
async fn main() {
    let result = zyanya_cli(TerminalOptions::new().with_prompt("$ "), None).await;
    if let Err(err) = result {
        println!("{err}");
    }
}
