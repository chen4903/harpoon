const EIP_1167_BYTECODE_PREFIX: &str = "0x363d3d373d3d3d363d";
const EIP_1167_BYTECODE_SUFFIX: &str = "57fd5bf3";

#[derive(Debug)]
pub struct Eip1167Error(String);

impl std::fmt::Display for Eip1167Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Eip1167Error {}

/// Parse EIP-1167 minimal proxy bytecode to extract implementation address
pub fn parse_1167_bytecode(bytecode: &str) -> Result<String, Eip1167Error> {
    if !bytecode.starts_with(EIP_1167_BYTECODE_PREFIX) {
        return Err(Eip1167Error("Not an EIP-1167 bytecode".to_string()));
    }

    let prefix_len = EIP_1167_BYTECODE_PREFIX.len();

    // Extract push opcode to determine address length
    if bytecode.len() < prefix_len + 2 {
        return Err(Eip1167Error("Not an EIP-1167 bytecode".to_string()));
    }

    let push_n_hex = &bytecode[prefix_len..prefix_len + 2];
    let push_opcode =
        u8::from_str_radix(push_n_hex, 16).map_err(|_| Eip1167Error("Invalid push opcode".to_string()))?;

    // push1 ... push20 use opcodes 0x60 ... 0x73
    let address_length = (push_opcode as i32) - 0x5f;

    if address_length < 1 || address_length > 20 {
        return Err(Eip1167Error("Not an EIP-1167 bytecode".to_string()));
    }

    let address_start = prefix_len + 2;
    let address_end = address_start + (address_length as usize) * 2;

    if bytecode.len() < address_end {
        return Err(Eip1167Error("Not an EIP-1167 bytecode".to_string()));
    }

    let address_from_bytecode = &bytecode[address_start..address_end];

    // Verify suffix
    const SUFFIX_OFFSET_FROM_ADDRESS_END: usize = 22;
    let suffix_start = address_end + SUFFIX_OFFSET_FROM_ADDRESS_END;

    if bytecode.len() < suffix_start + EIP_1167_BYTECODE_SUFFIX.len() {
        return Err(Eip1167Error("Not an EIP-1167 bytecode".to_string()));
    }

    if !bytecode[suffix_start..].starts_with(EIP_1167_BYTECODE_SUFFIX) {
        return Err(Eip1167Error("Not an EIP-1167 bytecode".to_string()));
    }

    // Pad address to 40 chars for vanity addresses
    let padded_address = format!("{:0>40}", address_from_bytecode);
    Ok(format!("0x{}", padded_address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_1167() {
        let bytecode = "0x363d3d373d3d3d363d73bebebebebebebebebebebebebebebebebebebebe5af43d82803e903d91602b57fd5bf3";
        let result = parse_1167_bytecode(bytecode).unwrap();
        assert_eq!(result, "0xbebebebebebebebebebebebebebebebebebebebe");
    }
}
