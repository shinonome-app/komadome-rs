use anyhow::{Context, Result};
use natsuzora_contract::{Contract, parse};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Pre-loaded contract registry for template validation
pub struct ContractRegistry {
    contracts: HashMap<String, Contract>,
}

impl ContractRegistry {
    /// Load all contracts from a directory
    pub fn load(contracts_dir: &Path) -> Result<Self> {
        let mut contracts = HashMap::new();
        let entries = Self::find_contracts(contracts_dir)?;

        for entry in entries {
            let rel_path = entry
                .strip_prefix(contracts_dir)
                .unwrap_or(&entry)
                .to_string_lossy()
                .replace('\\', "/");

            let name = rel_path.trim_end_matches(".ntzc").to_string();
            let source = fs::read_to_string(&entry)
                .with_context(|| format!("Failed to read contract: {}", entry.display()))?;

            let contract = parse(&source)
                .map_err(|e| anyhow::anyhow!("Failed to parse contract {name}: {e}"))?;

            contracts.insert(name, contract);
        }

        Ok(Self { contracts })
    }

    /// Get a contract by name
    pub fn get(&self, name: &str) -> Option<&Contract> {
        self.contracts.get(name)
    }

    /// List all loaded contract names
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.contracts.keys()
    }

    fn find_contracts(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut results = Vec::new();
        if !dir.exists() {
            return Ok(results);
        }
        Self::find_contracts_recursive(dir, &mut results)?;
        Ok(results)
    }

    fn find_contracts_recursive(dir: &Path, results: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::find_contracts_recursive(&path, results)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("ntzc") {
                results.push(path);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_contract_registry() {
        let dir = TempDir::new().unwrap();

        let contract_path = dir.path().join("test.ntzc");
        let mut file = fs::File::create(&contract_path).unwrap();
        writeln!(file, "name: scalar").unwrap();

        let registry = ContractRegistry::load(dir.path()).unwrap();
        assert!(registry.get("test").is_some());
    }
}
