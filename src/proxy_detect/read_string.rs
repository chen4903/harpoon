#[derive(Debug)]
pub struct ReadStringError(String);

impl std::fmt::Display for ReadStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ReadStringError {}

/// Decodes an ABI-encoded hex string from JSON-RPC response to UTF-8 string
pub fn read_string(hex: &str) -> Result<String, ReadStringError> {
    if !hex.starts_with("0x") {
        return Err(ReadStringError("Hex string must start with 0x".to_string()));
    }

    let clean_hex = &hex[2..];

    if clean_hex.is_empty() {
        return Ok(String::new());
    }

    if clean_hex.len() % 2 != 0 {
        return Err(ReadStringError("Invalid hex string length".to_string()));
    }

    // First 32 bytes contain the offset to string data
    if clean_hex.len() < 64 {
        return Err(ReadStringError("Invalid string offset".to_string()));
    }

    let offset_hex = &clean_hex[0..64];
    let offset =
        u64::from_str_radix(offset_hex, 16).map_err(|_| ReadStringError("Invalid string offset".to_string()))?;

    if offset != 32 {
        return Err(ReadStringError("Invalid string offset".to_string()));
    }

    // Next 32 bytes contain the length of the string in bytes
    if clean_hex.len() < 128 {
        return Err(ReadStringError("Invalid string length".to_string()));
    }

    let length_hex = &clean_hex[64..128];
    let length =
        usize::from_str_radix(length_hex, 16).map_err(|_| ReadStringError("Invalid string length".to_string()))?;

    // Get the actual string data
    let string_start = 128;
    let string_end = string_start + length * 2;

    if clean_hex.len() < string_end {
        return Err(ReadStringError("Insufficient string data".to_string()));
    }

    let string_hex = &clean_hex[string_start..string_end];

    // Convert hex string to bytes
    let mut bytes = Vec::with_capacity(length);
    for i in (0..string_hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&string_hex[i..i + 2], 16)
            .map_err(|_| ReadStringError("Invalid hex string".to_string()))?;
        bytes.push(byte);
    }

    // Convert bytes to UTF-8 string
    String::from_utf8(bytes).map_err(|_| ReadStringError("Invalid UTF-8 data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_string() {
        let hex = "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000057465737400000000000000000000000000000000000000000000000000000000";
        let result = read_string(hex).unwrap();
        assert_eq!(result, "test");
    }
}
