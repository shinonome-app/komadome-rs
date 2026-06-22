use anyhow::{Context, Result};
use natsuzora::Natsuzora;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Pre-compiled template registry
pub struct TemplateRegistry {
    templates: HashMap<String, Natsuzora>,
    include_root: PathBuf,
}

impl TemplateRegistry {
    /// Load and compile all templates from a directory
    pub fn load(template_dir: &Path) -> Result<Self> {
        let mut templates = HashMap::new();

        // Find all .ntzr template files
        let entries = super::find_files_with_ext(template_dir, "ntzr")?;

        for entry in entries {
            let rel_path = entry
                .strip_prefix(template_dir)
                .unwrap_or(&entry)
                .to_string_lossy()
                .replace('\\', "/");

            let name = rel_path.trim_end_matches(".ntzr").to_string();
            let source = fs::read_to_string(&entry)
                .with_context(|| format!("Failed to read template: {}", entry.display()))?;

            let tmpl = Natsuzora::parse_with_includes(&source, template_dir)
                .with_context(|| format!("Failed to parse template: {name}"))?;

            templates.insert(name, tmpl);
        }

        Ok(Self {
            templates,
            include_root: template_dir.to_path_buf(),
        })
    }

    /// Get a compiled template by name
    pub fn get(&self, name: &str) -> Option<&Natsuzora> {
        self.templates.get(name)
    }

    /// Render a template with data
    pub fn render(&self, name: &str, data: serde_json::Value) -> Result<String> {
        let tmpl = self
            .get(name)
            .with_context(|| format!("Template not found: {name}"))?;

        tmpl.render(data)
            .map_err(|e| anyhow::anyhow!("Render error in {name}: {e}"))
    }

    /// Get the include root path
    pub fn include_root(&self) -> &Path {
        &self.include_root
    }

    /// List all loaded template names
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.templates.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_template_registry() {
        let dir = TempDir::new().unwrap();

        // Create a simple template
        let tmpl_path = dir.path().join("test.ntzr");
        let mut file = fs::File::create(&tmpl_path).unwrap();
        writeln!(file, "Hello, {{[ name ]}}!").unwrap();

        let registry = TemplateRegistry::load(dir.path()).unwrap();
        assert!(registry.get("test").is_some());

        let result = registry
            .render("test", serde_json::json!({"name": "World"}))
            .unwrap();
        assert_eq!(result, "Hello, World!\n");
    }
}
