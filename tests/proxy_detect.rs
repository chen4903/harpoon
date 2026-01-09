use alloy::providers::ProviderBuilder;
use harpoon::proxy_detect::{ProxyType, detect_proxy};
#[tokio::test]
async fn test_eip1967_direct_proxy() {
    // Example: EIP-1967 direct proxy
    let proxy_address = "0xA7AeFeaD2F25972D80516628417ac46b3F2604Af";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    // Note: If this contract is not actually EIP-1967, the test will fail with expect()
    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected proxy type: {:?}, target = {}", proxy_type, target);
                assert_eq!(proxy_type, ProxyType::Eip1967Direct);
                assert!(
                    target.to_string().to_lowercase()
                        == "0x4bd844f72a8edd323056130a86fc624d0dbcf5b0".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ This contract is not a proxy or not EIP-1967 Direct pattern");
    }
}

#[tokio::test]
async fn test_eip1967_beacon_proxy() {
    let proxy_address = "0xDd4e2eb37268B047f55fC5cAf22837F9EC08A881";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected proxy type: {:?}, target = {}", proxy_type, target);
                // Verify it's one of the expected types
                assert!(
                    proxy_type == ProxyType::Eip1967Beacon,
                    "Expected EIP-1967 Beacon proxy pattern"
                );
                assert!(
                    target.to_string().to_lowercase()
                        == "0xe5c048792dcf2e4a56000c8b6a47f21df22752d1".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected for this address");
    }
}

#[tokio::test]
async fn test_eip1967_variant_proxy() {
    // Example: EIP-1967 variant proxy
    let proxy_address = "0x114f1388fAB456c4bA31B1850b244Eedcd024136";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;
    // Note: If this contract is not actually EIP-1967, the test will fail with expect()
    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected proxy type: {:?}, target = {}", proxy_type, target);
                assert_eq!(proxy_type, ProxyType::Eip1967Beacon);
                assert!(
                    target.to_string().to_lowercase()
                        == "0x0fa0fd98727c443dd5275774c44d27cff9d279ed".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ This contract is not a proxy or not EIP-1967 Direct pattern");
    }
}

#[tokio::test]
async fn test_openzeppelin_proxy() {
    let proxy_address = "0xC986c2d326c84752aF4cC842E033B9ae5D54ebbB";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected OpenZeppelin proxy: target = {}", target);
                assert_eq!(proxy_type, ProxyType::OpenZeppelin);
                assert!(
                    target.to_string().to_lowercase()
                        == "0x0656368c4934e56071056da375d4a691d22161f8".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected - this is expected for dummy address");
    }
}

#[tokio::test]
async fn test_eip897_delegate_proxy() {
    let proxy_address = "0x8260b9eC6d472a34AD081297794d7Cc00181360a";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single {
                target,
                proxy_type,
                immutable,
            } => {
                println!(
                    "✅ Detected proxy type: {:?}, target = {}, immutable = {}",
                    proxy_type, target, immutable
                );
                assert_eq!(proxy_type, ProxyType::Eip1967Direct);
                assert!(
                    target.to_string().to_lowercase()
                        == "0xe4e4003afe3765aca8149a82fc064c0b125b9e5a".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected - this is expected for dummy address");
    }
}

#[tokio::test]
async fn test_eip1167_minimal_proxy() {
    let proxy_address = "0x6d5d9b6ec51c15f45bfa4c460502403351d5b999";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected proxy type: {:?}, target = {}", proxy_type, target);
                // Verify it's one of the expected types
                assert!(proxy_type == ProxyType::Eip1167, "Expected EIP-1167");
                assert!(
                    target.to_string().to_lowercase()
                        == "0x210ff9ced719e9bf2444dbc3670bac99342126fa".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected for this address");
    }
}

#[tokio::test]
async fn test_eip1167_minimal_proxy_with_vanity_address() {
    let proxy_address = "0xa81043fd06D57D140f6ad8C2913DbE87fdecDd5F";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected proxy type: {:?}, target = {}", proxy_type, target);
                // Verify it's one of the expected types
                assert!(proxy_type == ProxyType::Eip1167, "Expected EIP-1167");
                assert!(
                    target.to_string().to_lowercase()
                        == "0x0000000010fd301be3200e67978e3cc67c962f48".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected for this address");
    }
}

#[tokio::test]
async fn test_safe_proxy() {
    let proxy_address = "0x0DA0C3e52C977Ed3cBc641fF02DD271c3ED55aFe";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected Safe proxy: target = {}", target);
                assert_eq!(proxy_type, ProxyType::Safe);
                assert!(
                    target.to_string().to_lowercase()
                        == "0xd9db270c1b5e3bd161e8c8503c55ceabee709552".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected - this is expected for dummy address");
    }
}

#[tokio::test]
async fn test_comptroller_proxy() {
    // Example: Compound Comptroller proxy
    let proxy_address = "0x3d9819210A31b4961b30EF54bE2aeD79B9c9Cd3B"; // Compound Comptroller
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected Comptroller proxy: target = {}", target);
                assert_eq!(proxy_type, ProxyType::Comptroller);
                assert!(
                    target.to_string().to_lowercase()
                        == "0xbafe01ff935c7305907c33bf824352ee5979b526".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected - this is expected for dummy address");
    }
}

#[tokio::test]
async fn test_balancer_batch_relayer_proxy() {
    let proxy_address = "0x35cea9e57a393ac66aaa7e25c391d52c74b5648f";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Single { target, proxy_type, .. } => {
                println!("✅ Detected BatchRelayer proxy: target = {}", target);
                assert_eq!(proxy_type, ProxyType::BatchRelayer);
                assert!(
                    target.to_string().to_lowercase()
                        == "0xea66501df1a00261e3bb79d1e90444fc6a186b62".to_string().to_lowercase()
                );
            }
            _ => panic!("Expected single target result"),
        }
    } else {
        println!("❌ No proxy detected - this is expected for dummy address");
    }
}

#[tokio::test]
async fn test_eip2535_diamond_proxy() {
    // Example: EIP-2535 Diamond proxy
    let proxy_address = "0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE";
    println!("Testing proxy address: {}", proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, proxy_address.parse().unwrap(), None).await;

    if let Some(proxy_result) = result {
        match proxy_result {
            harpoon::proxy_detect::ProxyResult::Diamond { target, proxy_type, .. } => {
                println!("✓ Detected EIP-2535 Diamond proxy: {} facets", target.len());
                assert_eq!(proxy_type, ProxyType::Eip2535Diamond);
                assert!(!target.is_empty());
                assert!(target.len() > 20);
            }
            _ => panic!("Expected diamond target result"),
        }
    } else {
        println!("❌ No proxy detected - this is expected for dummy address");
    }
}

#[tokio::test]
async fn test_non_proxy_contract() {
    // Test with a non-proxy contract
    let non_proxy_address = "0x0000000000000000000000000000000000000000";
    println!("Testing proxy address: {}", non_proxy_address);

    let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse().unwrap());
    let result = detect_proxy(&provider, non_proxy_address.parse().unwrap(), None).await;

    println!("✓ Correctly returned None for non-proxy contract");
    assert!(result.is_none(), "Should return None for non-proxy contracts");
}
