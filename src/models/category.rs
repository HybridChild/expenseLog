use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CategoryError {
    #[error("Invalid category: {0}")]
    InvalidCategory(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CategoryType {
    /// Built-in categories that are always available
    System,
    /// User-defined categories from configuration
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Category {
    name: String,
    category_type: CategoryType,
}

impl Category {
    pub fn new_system(name: &str) -> Self {
        Self {
            name: name.to_string(),
            category_type: CategoryType::System,
        }
    }

    pub fn new_custom(name: &str) -> Self {
        Self {
            name: name.to_string(),
            category_type: CategoryType::Custom,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn category_type(&self) -> &CategoryType {
        &self.category_type
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromStr for Category {
    type Err = CategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Get system categories registry
        let system_cats = get_system_categories();
        
        // First check if it's a system category
        for cat in system_cats {
            if cat.name.eq_ignore_ascii_case(s) {
                return Ok(cat);
            }
        }
        
        // In a real implementation, we'd check custom categories here
        // but for now we'll just return an error
        
        Err(CategoryError::InvalidCategory(format!("Category '{}' not found", s)))
    }
}

/// Returns a list of all system categories
fn get_system_categories() -> Vec<Category> {
    vec![
        Category::new_system("Food"),
        Category::new_system("Housing"),
        Category::new_system("Transportation"),
        Category::new_system("Utilities"),
        Category::new_system("Healthcare"),
        Category::new_system("Entertainment"),
    ]
}

/// Manages all available categories in the application
pub struct CategoryRegistry {
    system_categories: HashSet<Category>,
    custom_categories: HashSet<Category>,
}

impl CategoryRegistry {
    pub fn new() -> Self {
        let system_categories = get_system_categories()
            .into_iter()
            .collect::<HashSet<_>>();
            
        Self {
            system_categories,
            custom_categories: HashSet::new(),
        }
    }
    
    /// Load custom categories from configuration
    pub fn load_custom_categories(&mut self, category_names: Vec<String>) {
        self.custom_categories.clear();
        for name in category_names {
            self.custom_categories.insert(Category::new_custom(&name));
        }
    }
    
    /// Get all available categories (both system and custom)
    pub fn all_categories(&self) -> Vec<&Category> {
        self.system_categories.iter()
            .chain(self.custom_categories.iter())
            .collect()
    }
    
    /// Check if a category with the given name exists
    pub fn category_exists(&self, name: &str) -> bool {
        self.system_categories.iter().any(|c| c.name.eq_ignore_ascii_case(name)) ||
        self.custom_categories.iter().any(|c| c.name.eq_ignore_ascii_case(name))
    }
    
    /// Get a category by name
    pub fn get_category(&self, name: &str) -> Option<&Category> {
        self.system_categories.iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
            .or_else(|| self.custom_categories.iter()
                .find(|c| c.name.eq_ignore_ascii_case(name)))
    }
    
    /// Add a new custom category
    pub fn add_custom_category(&mut self, name: &str) -> Result<&Category, CategoryError> {
        // Check if it already exists
        if self.category_exists(name) {
            return Err(CategoryError::InvalidCategory(
                format!("Category '{}' already exists", name)
            ));
        }
        
        let category = Category::new_custom(name);
        self.custom_categories.insert(category);
        
        Ok(self.get_category(name).unwrap())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn create_system_category() {
        let category = Category::new_system("Food");
        
        assert_eq!(category.name(), "Food");
        assert_eq!(category.category_type(), &CategoryType::System);
    }
    
    #[test]
    fn create_custom_category() {
        let category = Category::new_custom("Books");
        
        assert_eq!(category.name(), "Books");
        assert_eq!(category.category_type(), &CategoryType::Custom);
    }
    
    #[test]
    fn category_equality() {
        let cat1 = Category::new_system("Food");
        let cat2 = Category::new_system("Food");
        let cat3 = Category::new_system("Housing");
        
        assert_eq!(cat1, cat2);
        assert_ne!(cat1, cat3);
    }
    
    #[test]
    fn category_display() {
        let category = Category::new_system("Food");
        
        assert_eq!(format!("{}", category), "Food");
    }
    
    #[test]
    fn category_from_str() {
        // First try with system category
        let food = Category::from_str("Food").unwrap();
        assert_eq!(food.name(), "Food");
        assert_eq!(food.category_type(), &CategoryType::System);
        
        // Try with case insensitivity
        let housing = Category::from_str("housing").unwrap();
        assert_eq!(housing.name(), "Housing"); // Note: should return canonical name
        
        // Try with a non-existent category
        let result = Category::from_str("NonExistent");
        assert!(result.is_err());
    }
    
    #[test]
    fn category_registry_initialize() {
        let registry = CategoryRegistry::new();
        
        // Check that default system categories exist
        assert!(registry.category_exists("Food"));
        assert!(registry.category_exists("Housing"));
        assert!(registry.category_exists("Transportation"));
        assert!(registry.category_exists("Utilities"));
        
        // Check case insensitivity
        assert!(registry.category_exists("food"));
        
        // Check that a non-existent category doesn't exist
        assert!(!registry.category_exists("NonExistent"));
    }
    
    #[test]
    fn load_custom_categories() {
        let mut registry = CategoryRegistry::new();
        
        let custom_categories = vec![
            "Books".to_string(),
            "Hobbies".to_string(),
        ];
        
        registry.load_custom_categories(custom_categories);
        
        assert!(registry.category_exists("Books"));
        assert!(registry.category_exists("Hobbies"));
        
        // Original system categories should still exist
        assert!(registry.category_exists("Food"));
    }
    
    #[test]
    fn serialize_category() {
        let category = Category::new_system("Food");
        
        let serialized = serde_json::to_string(&category).unwrap();
        
        assert!(serialized.contains("Food"));
        assert!(serialized.contains("System"));
    }
    
    #[test]
    fn deserialize_category() {
        let json = r#"{"name":"Food","category_type":"System"}"#;
        
        let category: Category = serde_json::from_str(json).unwrap();
        
        assert_eq!(category.name(), "Food");
        assert_eq!(category.category_type(), &CategoryType::System);
    }
}
