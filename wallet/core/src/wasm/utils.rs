use crate::result::Result;
use js_sys::BigInt;
use zyanya_consensus_core::network::{NetworkType, NetworkTypeT};
use wasm_bindgen::prelude::*;
use workflow_wasm::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "bigint | number | HexString")]
    #[derive(Clone, Debug)]
    pub type ISompiToZyanya;
}

/// Convert a Zyanya string to Sompi represented by bigint.
/// This function provides correct precision handling and
/// can be used to parse user input.
/// @category Wallet SDK
#[wasm_bindgen(js_name = "zyanyaToSompi")]
pub fn zyanya_to_sompi(zyanya: String) -> Option<BigInt> {
    crate::utils::try_zyanya_str_to_sompi(zyanya).ok().flatten().map(Into::into)
}

///
/// Convert Sompi to a string representation of the amount in Zyanya.
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToZyanyaString")]
pub fn sompi_to_zyanya_string(sompi: ISompiToZyanya) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    Ok(crate::utils::sompi_to_zyanya_string(sompi))
}

///
/// Format a Sompi amount to a string representation of the amount in Zyanya with a suffix
/// based on the network type (e.g. `ZYAN` for mainnet, `TZYAN` for testnet,
/// `SZYAN` for simnet, `DZYAN` for devnet).
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToZyanyaStringWithSuffix")]
pub fn sompi_to_zyanya_string_with_suffix(sompi: ISompiToZyanya, network: &NetworkTypeT) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    let network_type = NetworkType::try_from(network)?;
    Ok(crate::utils::sompi_to_zyanya_string_with_suffix(sompi, &network_type))
}
