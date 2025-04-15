use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;
use std::io;
use thiserror::Error;

use crate::models::category::{Category, CategoryRegistry, CategoryError};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error("Category error: {0}")]
    CategoryError(#[from] CategoryError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: String,
    pub currency_symbol: String,
    pub categories: Vec<Category>,
}

impl Config {
    pub fn default() -> Result<Self, ConfigError> {
        let default_categories = vec![
            Category::new("Food", Some("Groceries, restaurants, takeout"))?,
            Category::new("Housing", Some("Rent, mortgage, repairs"))?,
            Category::new("Transportation", Some("Public transit, gas, car maintenance"))?,
            Category::new("Utilities", Some("Electricity, water, internet"))?,
            Category::new("Healthcare", Some("Doctor visits, medications"))?,
            Category::new("Entertainment", Some("Movies, games, hobbies"))?,
            Category::new("Personal", Some("Clothing, haircuts, gym"))?,
            Category::new("Education", Some("Tuition, books, courses"))?,
        ];
        
        Ok(Self {
            database_path: "expense_log.db".to_string(),
            currency_symbol: "$".to_string(),
            categories: default_categories,
        })
    }
    
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Self::default();
        }
        
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn configure_category_registry(&self, registry: &mut CategoryRegistry) {
        registry.load_categories(self.categories.clone());
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default().unwrap();
        
        // Verify default values
        assert_eq!(config.database_path, "expense_log.db");
        assert_eq!(config.currency_symbol, "$");
        assert!(!config.categories.is_empty());
    }
    
    #[test]
    fn test_load_config() {
        // Create a temporary config file
        let mut file = NamedTempFile::new().unwrap();
        
        write!(file, r#"
database_path: "test.db"
currency_symbol: "€"
categories:
  - name: "Test Category"
    description: "A test category"
  - name: "Another Category"
    description: null
"#).unwrap();
        
        // Load the config
        let config = Config::load(file.path()).unwrap();
        
        // Verify loaded values
        assert_eq!(config.database_path, "test.db");
        assert_eq!(config.currency_symbol, "€");
        assert_eq!(config.categories.len(), 2);
        
        let category_names: Vec<_> = config.categories.iter()
            .map(|c| c.name())
            .collect();
        assert!(category_names.contains(&"Test Category"));
        assert!(category_names.contains(&"Another Category"));
    }
    
    #[test]
    fn test_save_config() -> Result<(), ConfigError> {
        let mut config = Config::default()?;

        config.database_path = "custom.db".to_string();
        config.currency_symbol = "£".to_string();
        config.categories = vec![
            Category::new("Custom Category", Some("A custom category"))?,
        ];
        
        // Create a temporary file for saving
        let file = NamedTempFile::new().unwrap();
        
        // Save the config
        config.save(file.path())?;
        
        // Load it back to verify
        let loaded_config = Config::load(file.path())?;
        
        assert_eq!(loaded_config.database_path, "custom.db");
        assert_eq!(loaded_config.currency_symbol, "£");
        assert_eq!(loaded_config.categories.len(), 1);
        assert_eq!(loaded_config.categories[0].name(), "Custom Category");
        
        Ok(())
    }
    
    #[test]
    fn test_load_nonexistent_config() {
        // Try to load a non-existent file
        let result = Config::load(std::path::Path::new("nonexistent.yaml"));
        
        // It should return the default config
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.database_path, "expense_log.db");
    }
    
    #[test]
    fn test_configure_category_registry() -> Result<(), ConfigError> {
        let config = Config {
            database_path: "test.db".to_string(),
            currency_symbol: "$".to_string(),
            categories: vec![
                Category::new("Food", Some("Groceries"))?,
                Category::new("Housing", None)?,
            ],
        };
        
        let mut registry = crate::models::category::CategoryRegistry::new();
        config.configure_category_registry(&mut registry);
        
        assert!(registry.category_exists("Food"));
        assert!(registry.category_exists("Housing"));
        assert_eq!(registry.all_categories().len(), 2);
        
        Ok(())
    }
}
