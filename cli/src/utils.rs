use crate::error::Error;
use crate::result::Result;
use zyanya_consensus_core::constants::SOMPI_PER_ZYANYA;
use std::fmt::Display;

pub fn try_parse_required_nonzero_zyanya_as_sompi_u64<S: ToString + Display>(zyanya_amount: Option<S>) -> Result<u64> {
    if let Some(zyanya_amount) = zyanya_amount {
        let sompi_amount = zyanya_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied Zyanya amount is not valid: '{zyanya_amount}'")))?
            * SOMPI_PER_ZYANYA as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Zyanya amount is not valid: '{zyanya_amount}'"))
        } else {
            let sompi_amount = sompi_amount as u64;
            if sompi_amount == 0 {
                Err(Error::custom("Supplied required zyanya amount must not be a zero: '{zyanya_amount}'"))
            } else {
                Ok(sompi_amount)
            }
        }
    } else {
        Err(Error::custom("Missing Zyanya amount"))
    }
}

pub fn try_parse_required_zyanya_as_sompi_u64<S: ToString + Display>(zyanya_amount: Option<S>) -> Result<u64> {
    if let Some(zyanya_amount) = zyanya_amount {
        let sompi_amount = zyanya_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied Zyanya amount is not valid: '{zyanya_amount}'")))?
            * SOMPI_PER_ZYANYA as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Zyanya amount is not valid: '{zyanya_amount}'"))
        } else {
            Ok(sompi_amount as u64)
        }
    } else {
        Err(Error::custom("Missing Zyanya amount"))
    }
}

pub fn try_parse_optional_zyanya_as_sompi_i64<S: ToString + Display>(zyanya_amount: Option<S>) -> Result<Option<i64>> {
    if let Some(zyanya_amount) = zyanya_amount {
        let sompi_amount = zyanya_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_e| Error::custom(format!("Supplied Zyanya amount is not valid: '{zyanya_amount}'")))?
            * SOMPI_PER_ZYANYA as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Zyanya amount is not valid: '{zyanya_amount}'"))
        } else {
            Ok(Some(sompi_amount as i64))
        }
    } else {
        Ok(None)
    }
}
