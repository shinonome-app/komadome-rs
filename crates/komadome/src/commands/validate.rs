use anyhow::{Context, Result};
use natsuzora::{IncludeLoader, LoaderError, Natsuzora, Template};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::generator::contracts::ContractRegistry;

/// File-system backed include loader for template validation.
///
/// Resolves include names (e.g. `/top/body`) to partial files
/// (e.g. `templates/top/_body.ntzr`) under the templates directory.
struct FileIncludeLoader {
    root: PathBuf,
    cache: HashMap<String, Template>,
}

impl FileIncludeLoader {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            cache: HashMap::new(),
        }
    }

    fn resolve_path(&self, name: &str) -> Result<PathBuf> {
        let segments: Vec<&str> = name
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        if segments.is_empty() {
            anyhow::bail!("include name '{}' resolves to empty path", name);
        }

        let mut path = self.root.clone();
        if segments.len() > 1 {
            for seg in &segments[..segments.len() - 1] {
                path.push(seg);
            }
        }
        path.push(format!("_{}.ntzr", segments.last().unwrap()));
        Ok(path)
    }
}

impl IncludeLoader for FileIncludeLoader {
    fn load(&mut self, name: &str) -> Result<Template, LoaderError> {
        if let Some(template) = self.cache.get(name) {
            return Ok(template.clone());
        }

        let path = self.resolve_path(name)?;
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading include '{}'", path.display()))?;
        let parsed = Natsuzora::parse(&source)
            .map_err(|e| -> LoaderError { Box::new(e) })?;
        let template = parsed.template().clone();
        self.cache.insert(name.to_string(), template.clone());
        Ok(template)
    }
}

pub fn run(config: &Config) -> Result<()> {
    let templates_dir = &config.templates.directory;
    let contracts_dir = templates_dir
        .parent()
        .expect("templates directory must have a parent")
        .join("contracts");

    if !contracts_dir.exists() {
        println!("Contracts directory not found: {}", contracts_dir.display());
        return Ok(());
    }

    if !templates_dir.exists() {
        println!("Templates directory not found: {}", templates_dir.display());
        return Ok(());
    }

    println!("Loading contracts from {}...", contracts_dir.display());
    let registry = ContractRegistry::load(&contracts_dir)?;
    let mut contract_names: Vec<String> = registry.names().cloned().collect();
    contract_names.sort();

    println!("Found {} contract(s).\n", contract_names.len());

    let mut total_errors = 0usize;
    let mut checked = 0usize;
    let mut skipped = 0usize;

    for name in &contract_names {
        let template_path = templates_dir.join(format!("{}.ntzr", name));

        if !template_path.exists() {
            println!("  skip: {}.ntzc (no matching template)", name);
            skipped += 1;
            continue;
        }

        let contract = registry.get(name).unwrap();
        let template_source = fs::read_to_string(&template_path)
            .with_context(|| format!("reading template {}", template_path.display()))?;

        let parsed = match Natsuzora::parse(&template_source) {
            Ok(t) => t,
            Err(e) => {
                println!("  PARSE ERROR: {}.ntzr: {}", name, e);
                total_errors += 1;
                continue;
            }
        };

        let mut loader = FileIncludeLoader::new(templates_dir.to_path_buf());
        let errors = subaru::check_template(parsed.template(), contract, &mut loader);

        checked += 1;

        if errors.is_empty() {
            println!("  ok: {}", name);
        } else {
            for error in &errors {
                println!(
                    "  {}:{}:{}: error: {}",
                    template_path.display(),
                    error.location.line,
                    error.location.column,
                    error.message
                );
                if let Some(suggestion) = &error.suggestion {
                    println!("    hint: {}", suggestion);
                }
            }
            total_errors += errors.len();
        }
    }

    println!();
    println!(
        "Checked {} template(s), skipped {}, found {} violation(s).",
        checked, skipped, total_errors
    );

    if total_errors > 0 {
        anyhow::bail!("{} violation(s) found", total_errors);
    }

    Ok(())
}
