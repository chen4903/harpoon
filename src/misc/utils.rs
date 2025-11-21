use alloy::primitives::{Address, keccak256};

/// Calculate UniswapV2 pair address using CREATE2
///
/// # Arguments
/// * `token0` - Address of the first token
/// * `token1` - Address of the second token
/// * `factory` - Address of the UniswapV2 factory contract
/// * `init_code_hash` - Hash of the pair contract creation code
///
/// # Returns
/// * The calculated pair address
pub fn calculate_pair_address(token0: Address, token1: Address, factory: Address, init_code_hash: [u8; 32]) -> Address {
    // Ensure token0 < token1
    let (token0, token1) = if token0 < token1 {
        (token0, token1)
    } else {
        (token1, token0)
    };

    // Pack tokens for hashing (equivalent to abi.encodePacked)
    let mut packed = Vec::with_capacity(40);
    packed.extend_from_slice(&token0.as_slice());
    packed.extend_from_slice(&token1.as_slice());

    // Calculate salt
    let salt = keccak256(&packed);

    // Pack data for final hash (prefix + factory + salt + init_code_hash)
    let mut final_data = Vec::with_capacity(85);
    final_data.push(0xff); // prefix for CREATE2
    final_data.extend_from_slice(&factory.as_slice());
    final_data.extend_from_slice(&salt.to_vec());
    final_data.extend_from_slice(&init_code_hash);

    // Calculate final hash and convert to address
    let hash = keccak256(&final_data);
    // Take the last 20 bytes of the hash to form the address
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[12..32]);
    Address::from_slice(&addr)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, address};

    use super::calculate_pair_address;

    #[test]
    fn test_calculate_pair_address() {
        // https://bscscan.com/address/0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73#readContract
        // https://app.blocksec.com/explorer/tx/bsc/0xee2ba194fc5dc8fe4688b78ceddad40e8d3f43ce7833e5ff43e61f9521447b7f

        const PANCAKESWAP_FACTORY: Address = address!("0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73");
        const PANCAKESWAP_INIT_CODE_HASH: [u8; 32] = [
            0x00, 0xfb, 0x7f, 0x63, 0x07, 0x66, 0xe6, 0xa7, 0x96, 0x04, 0x8e, 0xa8, 0x7d, 0x01, 0xac, 0xd3, 0x06, 0x8e,
            0x8f, 0xf6, 0x7d, 0x07, 0x81, 0x48, 0xa3, 0xfa, 0x3f, 0x4a, 0x84, 0xf6, 0x9b, 0xd5,
        ];

        // Test tokens
        let token0 = address!("0x77b21F1c9817f480C8A91b16af37671592664444");
        let token1 = address!("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c");

        let pair_address = calculate_pair_address(token0, token1, PANCAKESWAP_FACTORY, PANCAKESWAP_INIT_CODE_HASH);

        // Known pair address from BSCScan
        let expected_address = address!("0xa19b1cD93E1A32895916Aff3888d47308D7C0Ad7");

        assert_eq!(
            pair_address, expected_address,
            "Calculated pair address does not match expected address"
        );
    }
}
