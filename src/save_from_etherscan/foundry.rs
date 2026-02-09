use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::etherscan::{ContractInfo, SourceFile};
use chrono::Local;

/// Foundry project structure manager
pub struct FoundryProject {
    root_path: PathBuf,
}

impl FoundryProject {
    /// Create a new Foundry project manager
    fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    // Save contract info to foundry project
    pub fn save_as_foundry(output_path: &Path, contract_info: &ContractInfo) -> Result<PathBuf> {
        // Create project directory
        let project_path = Self::create_project_dir(output_path, &contract_info.contract_name)?;

        // Create project instance
        let project = Self::new(&project_path);

        // Initialize Foundry project structure
        project.ensure_structure()?;

        // Save source files
        project.save_sources(&contract_info.sources)?;

        // Save ABI if available
        if !contract_info.abi.is_empty() && contract_info.abi != "Contract source code not verified" {
            project.save_abi(&contract_info.contract_name, &contract_info.abi)?;
        }

        // Save contract metadata
        project.save_metadata(contract_info)?;

        Ok(project_path)
    }

    /// Initialize Foundry project structure if not exists
    fn ensure_structure(&self) -> Result<()> {
        // Create main directories
        let src_dir = self.root_path.join("src");
        let test_dir = self.root_path.join("test");
        let script_dir = self.root_path.join("script");
        let lib_dir = self.root_path.join("lib");

        for dir in [&src_dir, &test_dir, &script_dir, &lib_dir] {
            if !dir.exists() {
                fs::create_dir_all(dir).with_context(|| format!("Failed to create directory: {:?}", dir))?;
            }
        }

        // Create foundry.toml if not exists
        let foundry_toml = self.root_path.join("foundry.toml");
        if !foundry_toml.exists() {
            let default_config = r#"[profile.default]
src = "src"
out = "out"
libs = ["lib"]

# See more config options https://github.com/foundry-rs/foundry/tree/master/config
            "#;
            fs::write(&foundry_toml, default_config).context("Failed to create foundry.toml")?;
        }

        // Create .gitignore if not exists
        let gitignore = self.root_path.join(".gitignore");
        if !gitignore.exists() {
            let default_gitignore = r#"# Compiler files
cache/
out/

# Ignores development broadcast logs
!/broadcast
/broadcast/*/31337/
/broadcast/**/dry-run/

# Docs
docs/

# Dotenv file
.env
            "#;
            fs::write(&gitignore, default_gitignore).context("Failed to create .gitignore")?;
        }

        // Create remappings.txt if not exists
        let remappings = self.root_path.join("remappings.txt");
        if !remappings.exists() {
            let default_remappings = r#"forge-std/=lib/forge-std/src/
ds-test/=lib/forge-std/lib/ds-test/src/

@chainlink/contracts/=lib/chainlink-brownie-contracts/contracts/
@morpho-utils/=lib/morpho-utils/src/
@morpho-dao/morpho-utils/=lib/morpho-utils/src/
@morpho-dao/morpho-data-structures/=lib/morpho-data-structures/contracts/
permit2/=lib/v4-periphery/lib/permit2/
solady/=lib/solady/src/
ERC721A/=lib/ERC721A/
@rari-capital/=lib
solmate/=lib/solmate/src/

@openzeppelin/contracts/=lib/openzeppelin-contracts/contracts/
@openzeppelin/contracts-upgradeable/=lib/openzeppelin-contracts-upgradeable/contracts/
openzeppelin/=lib/openzeppelin-contracts/contracts/
openzeppelin-upgradeable/=lib/openzeppelin-contracts-upgradeable/contracts/
openzeppelin-contracts/=lib/openzeppelin-contracts/

v2-periphery/=lib/v2-periphery/
v2-core/=lib/v2-core/
v3-core/=lib/v3-core/
v3-periphery/=lib/v3-periphery/
v3-view/=lib/view-quoter-v3/
v4-core/=lib/v4-core/
v4-periphery/=lib/v4-periphery/
@uniswap/v4-core/=lib/v4-core/
@uniswap/v3-core=lib/v3-core/
@uniswap/v2-periphery=lib/v2-periphery/
@uniswap/v2-core=lib/v2-core/
            "#;
            fs::write(&remappings, default_remappings).context("Failed to create remappings.txt")?;
        }

        Ok(())
    }

    /// Create a new project directory with "harpoon-ContractName-YYYYMMDD-UnixTimestamp" format
    fn create_project_dir(base_path: &Path, contract_name: &str) -> Result<PathBuf> {
        let now = Local::now();
        let date = now.format("%Y%m%d").to_string();
        let timestamp = now.timestamp();
        let project_name = format!("harpoon-{}-{}-{}", contract_name, date, timestamp);
        let project_path = base_path.join(project_name);

        if !project_path.exists() {
            fs::create_dir_all(&project_path)
                .with_context(|| format!("Failed to create project directory: {:?}", project_path))?;
        }

        Ok(project_path)
    }

    /// Save contract sources to the appropriate directory (src/ or lib/)
    fn save_sources(&self, sources: &HashMap<String, SourceFile>) -> Result<()> {
        // Save each source file
        for (file_path, source_file) in sources {
            let target_path = self.determine_file_location(file_path);

            // Create parent directories if needed
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
                }
            }

            // Write the file
            fs::write(&target_path, &source_file.content)
                .with_context(|| format!("Failed to write file: {:?}", target_path))?;
        }

        Ok(())
    }

    /// Save contract ABI to abi/ directory
    fn save_abi(&self, contract_name: &str, abi_json: &str) -> Result<()> {
        let abi_dir = self.root_path.join("abi");

        if !abi_dir.exists() {
            fs::create_dir_all(&abi_dir).with_context(|| format!("Failed to create abi directory: {:?}", abi_dir))?;
        }

        let abi_file = abi_dir.join(format!("{}.json", contract_name));
        fs::write(&abi_file, abi_json).with_context(|| format!("Failed to write ABI file: {:?}", abi_file))?;
        Ok(())
    }

    /// Save contract metadata to metadata.json
    fn save_metadata(&self, contract_info: &ContractInfo) -> Result<()> {
        let metadata = serde_json::json!({
            "contract_name": contract_info.contract_name,
            "compiler_version": contract_info.compiler_version,
            "optimization_used": contract_info.optimization_used,
            "runs": contract_info.runs,
            "evm_version": contract_info.evm_version,
            "license_type": contract_info.license_type,
            "proxy": contract_info.proxy,
            "implementation": contract_info.implementation,
            "constructor_arguments": contract_info.constructor_arguments,
            "library": contract_info.library,
            "swarm_source": contract_info.swarm_source,
        });

        let metadata_file = self.root_path.join("metadata.json");
        let metadata_str = serde_json::to_string_pretty(&metadata).context("Failed to serialize metadata")?;

        fs::write(&metadata_file, metadata_str)
            .with_context(|| format!("Failed to write metadata file: {:?}", metadata_file))?;

        Ok(())
    }

    /// Determine whether file should go to src/ or lib/ based on path
    fn determine_file_location(&self, file_path: &str) -> PathBuf {
        let normalized = file_path.trim_start_matches('/');

        // Check if this is a library dependency
        // Case 1: starts with @ (npm-style import)
        // Case 2: starts with lib/@ or lib/ prefix
        // Case 3: contains @ anywhere in the path

        if normalized.starts_with('@') {
            // e.g., @openzeppelin/contracts/token/ERC20/IERC20.sol
            let lib_path = self.convert_npm_path_to_lib(normalized);
            self.root_path.join("lib").join(lib_path)
        } else if let Some(after_lib) = normalized.strip_prefix("lib/") {
            // e.g., lib/@openzeppelin/contracts/token/ERC20/IERC20.sol
            // or lib/openzeppelin-contracts/contracts/...
            if after_lib.starts_with('@') {
                let lib_path = self.convert_npm_path_to_lib(after_lib);
                self.root_path.join("lib").join(lib_path)
            } else {
                // Recursively handle nested lib/ paths using convert_npm_path_to_lib
                // which will strip additional lib/ prefixes
                let lib_path = self.convert_npm_path_to_lib(after_lib);
                self.root_path.join("lib").join(lib_path)
            }
        } else if normalized.contains("/@") {
            // e.g., some/path/@openzeppelin/contracts/...
            // Extract the part after /@
            if let Some(idx) = normalized.find("/@") {
                let npm_part = &normalized[idx + 1..];
                let lib_path = self.convert_npm_path_to_lib(npm_part);
                self.root_path.join("lib").join(lib_path)
            } else {
                // Fallback to src
                let src_dir = self.root_path.join("src");
                self.resolve_file_path(&src_dir, file_path)
            }
        } else {
            // Regular contract files go to src/
            let src_dir = self.root_path.join("src");
            self.resolve_file_path(&src_dir, file_path)
        }
    }

    /// Convert npm-style import path to lib directory structure
    fn convert_npm_path_to_lib(&self, npm_path: &str) -> PathBuf {
        // Handle different library formats
        // @openzeppelin/contracts/... -> openzeppelin-contracts/contracts/...
        // @openzeppelin/contracts-upgradeable/... -> openzeppelin-contracts-upgradeable/contracts/...
        // @openzeppelin/contracts-upgradeable/lib/openzeppelin-contracts/... -> openzeppelin-contracts/...
        // @aave/core-v3/... -> aave-v3-core/...

        if let Some(rest) = npm_path.strip_prefix("@openzeppelin/contracts-upgradeable/") {
            // Check if this is a nested lib dependency
            if let Some(nested) = rest.strip_prefix("lib/") {
                // Recursively process the nested path
                self.convert_npm_path_to_lib(nested)
            } else {
                PathBuf::from("openzeppelin-contracts-upgradeable/contracts").join(rest)
            }
        } else if let Some(rest) = npm_path.strip_prefix("@openzeppelin/contracts/") {
            if let Some(nested) = rest.strip_prefix("lib/") {
                self.convert_npm_path_to_lib(nested)
            } else {
                PathBuf::from("openzeppelin-contracts/contracts").join(rest)
            }
        } else if let Some(rest) = npm_path.strip_prefix("@openzeppelin/") {
            if let Some(nested) = rest.strip_prefix("lib/") {
                self.convert_npm_path_to_lib(nested)
            } else {
                PathBuf::from("openzeppelin-contracts").join(rest)
            }
        } else if let Some(rest) = npm_path.strip_prefix("@aave/core-v3/") {
            if let Some(nested) = rest.strip_prefix("lib/") {
                self.convert_npm_path_to_lib(nested)
            } else {
                PathBuf::from("aave-v3-core").join(rest)
            }
        } else if let Some(rest) = npm_path.strip_prefix("@aave/") {
            if let Some(nested) = rest.strip_prefix("lib/") {
                self.convert_npm_path_to_lib(nested)
            } else {
                PathBuf::from("aave").join(rest)
            }
        } else if npm_path.starts_with('@') {
            // Generic handling: @scope/package/path -> scope-package/path
            let without_at = npm_path.trim_start_matches('@');
            if let Some((scope, rest)) = without_at.split_once('/') {
                if let Some((package, path)) = rest.split_once('/') {
                    // Check for nested lib
                    if let Some(nested) = path.strip_prefix("lib/") {
                        self.convert_npm_path_to_lib(nested)
                    } else {
                        PathBuf::from(format!("{}-{}", scope, package)).join(path)
                    }
                } else {
                    PathBuf::from(format!("{}-{}", scope, rest))
                }
            } else {
                PathBuf::from(npm_path)
            }
        } else if let Some(nested) = npm_path.strip_prefix("lib/") {
            // Handle direct lib/ prefix (for non-npm style paths)
            self.convert_npm_path_to_lib(nested)
        } else if npm_path.contains("/lib/") {
            // Handle lib/ in the middle of path (e.g., openzeppelin-contracts-upgradeable/lib/openzeppelin-contracts/...)
            if let Some(idx) = npm_path.find("/lib/") {
                let after_lib = &npm_path[idx + 5..]; // +5 to skip "/lib/"
                self.convert_npm_path_to_lib(after_lib)
            } else {
                PathBuf::from(npm_path)
            }
        } else {
            PathBuf::from(npm_path)
        }
    }

    /// Resolve file path, handling absolute and relative paths
    fn resolve_file_path(&self, base_dir: &Path, file_path: &str) -> PathBuf {
        // Remove leading slashes and normalize path
        let normalized = file_path.trim_start_matches('/');

        // Handle common path prefixes
        let cleaned = if let Some(stripped) = normalized.strip_prefix("contracts/") {
            stripped
        } else if let Some(stripped) = normalized.strip_prefix("src/") {
            stripped
        } else {
            normalized
        };

        base_dir.join(cleaned)
    }
}
