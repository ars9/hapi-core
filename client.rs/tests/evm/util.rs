use ethers::{types::H160, utils::to_checksum as ethers_to_checksum};
use regex::Regex;

pub fn is_tx_match(value: &serde_json::Value) -> bool {
    Regex::new(r"^0x[0-9a-fA-F]{64}$").unwrap().is_match(
        value
            .get("tx")
            .expect("`tx` key not found")
            .as_str()
            .expect("`tx` is not a string"),
    )
}

pub fn to_checksum(value: &str) -> String {
    ethers_to_checksum(&value.parse::<H160>().expect("invalid address"), None)
}