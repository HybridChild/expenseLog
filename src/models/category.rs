use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CategoryError {
    #[error("Invalid category: {0}")]
    InvalidCategory(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    name: String,
    description: Option<String>,
}

// Manual implementations for equality and hashing based only on name
impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Category {}

impl std::hash::Hash for Category {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Category {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            description: description.map(String::from),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    pub fn set_description(&mut self, description: &str) {
        self.description = if description.trim().is_empty() {
            None
        } else {
            Some(description.to_string())
        };
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Manages all available categories in the application
pub struct CategoryRegistry {
    categories: HashSet<Category>,
}

impl CategoryRegistry {
    pub fn new() -> Self {
        Self {
            categories: HashSet::new(),
        }
    }
    
    /// Load categories into the registry
    pub fn load_categories(&mut self, categories: Vec<Category>) {
        self.categories.clear();
        
        for category in categories {
            self.categories.insert(category);
        }
    }
    
    /// Get all available categories
    pub fn all_categories(&self) -> Vec<&Category> {
        self.categories.iter().collect()
    }
    
    /// Check if a category with the given name exists
    pub fn category_exists(&self, name: &str) -> bool {
        self.categories.iter().any(|c| c.name.eq_ignore_ascii_case(name))
    }
    
    /// Get a category by name
    pub fn get_category(&self, name: &str) -> Option<&Category> {
        self.categories.iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }
    
    /// Add a new category
    pub fn add_category(&mut self, name: &str, description: Option<&str>) -> Result<&Category, CategoryError> {
        // Check if it already exists
        if self.category_exists(name) {
            return Err(CategoryError::InvalidCategory(
                format!("Category '{}' already exists", name)
            ));
        }
        
        let category = Category::new(name, description);
        self.categories.insert(category);
        
        Ok(self.get_category(name).unwrap())
    }
    
    /// Remove a category
    pub fn remove_category(&mut self, name: &str) -> Result<(), CategoryError> {
        let category = self.get_category(name).ok_or_else(|| {
            CategoryError::InvalidCategory(format!("Category '{}' not found", name))
        })?;
        
        // Clone the name because we need to use it after consuming the reference
        let category_name = category.name().to_string();
        
        // Remove the category
        self.categories.remove(&Category::new(&category_name, None));
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_category() {
        let category = Category::new("Food", None);
        
        assert_eq!(category.name(), "Food");
        assert_eq!(category.description(), None);
        
        let category_with_desc = Category::new(
            "Household", 
            Some("Furniture, kitchen ware, office supplies, etc.")
        );
        
        assert_eq!(category_with_desc.name(), "Household");
        assert_eq!(category_with_desc.description(), Some("Furniture, kitchen ware, office supplies, etc."));
    }
    
    #[test]
    fn category_equality() {
        let cat1 = Category::new("Food", None);
        let cat2 = Category::new("Food", None);
        let cat3 = Category::new("Housing", None);
        
        assert_eq!(cat1, cat2);
        assert_ne!(cat1, cat3);
        
        // Description doesn't affect equality (only name does)
        let cat4 = Category::new("Food", Some("Description"));
        assert_eq!(cat1, cat4);
    }
    
    #[test]
    fn category_display() {
        let category = Category::new("Food", None);
        
        assert_eq!(format!("{}", category), "Food");
    }
    
    #[test]
    fn empty_registry() {
        let registry = CategoryRegistry::new();
        assert_eq!(registry.all_categories().len(), 0);
        
        assert!(!registry.category_exists("Food"));
    }
    
    #[test]
    fn load_categories() {
        let mut registry = CategoryRegistry::new();
        let categories = vec![
            Category::new("Books", None),
            Category::new("Hobbies", Some("Various hobby expenses")),
        ];
        
        registry.load_categories(categories);
        
        assert!(registry.category_exists("Books"));
        assert!(registry.category_exists("Hobbies"));
        assert_eq!(registry.all_categories().len(), 2);
    }
    
    #[test]
    fn add_category() {
        let mut registry = CategoryRegistry::new();
        
        // Add a new category
        let result = registry.add_category("Software", Some("Apps, subscriptions, tools"));
        assert!(result.is_ok());
        
        // Verify it exists in the registry
        assert!(registry.category_exists("Software"));
        
        let category = registry.get_category("Software").unwrap();
        assert_eq!(category.name(), "Software");
        assert_eq!(category.description(), Some("Apps, subscriptions, tools"));
        
        // Try adding a duplicate
        let result = registry.add_category("Software", None);
        assert!(result.is_err());
    }
    
    #[test]
    fn remove_category() {
        let mut registry = CategoryRegistry::new();
        registry.add_category("Food", None).unwrap();
        assert!(registry.category_exists("Food"));
        
        // Remove a category
        let result = registry.remove_category("Food");
        assert!(result.is_ok());
        
        // Verify it's gone
        assert!(!registry.category_exists("Food"));
        
        // Try removing a non-existent category
        let result = registry.remove_category("NonExistent");
        assert!(result.is_err());
    }
    
    #[test]
    fn update_category_description() {
        let mut category = Category::new("Household", None);
        assert_eq!(category.description(), None);
        
        category.set_description("Furniture, kitchen ware, office supplies, etc.");
        assert_eq!(category.description(), Some("Furniture, kitchen ware, office supplies, etc."));
        
        // Test clearing description
        category.set_description("");
        assert_eq!(category.description(), None);
    }
    
    #[test]
    fn serialize_category() {
        let category = Category::new(
            "Household", 
            Some("Furniture, kitchen ware, office supplies, etc.")
        );
        
        let serialized = serde_json::to_string(&category).unwrap();
        
        assert!(serialized.contains("Household"));
        assert!(serialized.contains("Furniture, kitchen ware"));
    }
    
    #[test]
    fn deserialize_category() {
        let json = r#"{"name":"Food","description":"Groceries and restaurants"}"#;
        
        let category: Category = serde_json::from_str(json).unwrap();
        
        assert_eq!(category.name(), "Food");
        assert_eq!(category.description(), Some("Groceries and restaurants"));
    }
}
