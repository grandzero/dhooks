pub fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}
