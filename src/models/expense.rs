use serde::{Serialize, Deserialize};
use chrono::NaiveDate;
use thiserror::Error;
use crate::models::category::{Category, CategoryError};

#[derive(Debug, Error)]
pub enum ExpenseError {
    #[error("Invalid expense amount: {0}")]
    InvalidAmount(String),
    
    #[error("Invalid expense category: {0}")]
    InvalidCategory(#[from] CategoryError),
    
    #[error("Invalid expense date: {0}")]
    InvalidDate(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expense {
    id: Option<i64>,
    amount: f64,
    category: Category,
    date: NaiveDate,
    description: String,
}

impl Expense {
    pub fn new(amount: f64, category: Category, date: NaiveDate, description: String) -> Self {
        Self {
            id: None,
            amount,
            category,
            date,
            description,
        }
    }

    pub fn new_validated(
        amount: f64, 
        category: Category, 
        date: NaiveDate, 
        description: String
    ) -> Result<Self, ExpenseError> {
        // Validate amount
        if amount < 0.0 {
            return Err(ExpenseError::InvalidAmount("amount cannot be negative".to_string()));
        }
        
        // Category is already validated by the Category::new method
        
        // Validate date (example: don't allow future dates)
        let today = chrono::Local::now().naive_local().date();
        if date > today {
            return Err(ExpenseError::InvalidDate("date cannot be in the future".to_string()));
        }
        
        Ok(Self {
            id: None,
            amount,
            category,
            date,
            description,
        })
    }
    
    // Helper method that creates a Category and then an Expense in one step
    pub fn with_category_name(
        amount: f64,
        category_name: &str,
        category_description: Option<&str>,
        date: NaiveDate,
        description: String
    ) -> Result<Self, ExpenseError> {
        let category = Category::new(category_name, category_description)?;
        
        Self::new_validated(
            amount,
            category,
            date,
            description
        )
    }
    
    pub fn id(&self) -> Option<i64> {
        self.id
    }
    
    pub fn amount(&self) -> f64 {
        self.amount
    }
    
    pub fn category(&self) -> &Category {
        &self.category
    }
    
    pub fn date(&self) -> &NaiveDate {
        &self.date
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use serde_json;

    #[test]
    fn create_expense() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();
        let category = Category::new("Groceries", None).unwrap();

        let expense = Expense::new(
            42.50, 
            category, 
            date, 
            "Weekly shopping trip".to_string()
        );
        
        assert_eq!(expense.amount(), 42.50);
        assert_eq!(expense.category().name(), "Groceries");
        assert_eq!(expense.date(), &date);
        assert_eq!(expense.description(), "Weekly shopping trip");
    }
    
    #[test]
    fn create_expense_with_category_name() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();

        let expense = Expense::with_category_name(
            42.50, 
            "Groceries",
            Some("Food and household items"),
            date, 
            "Weekly shopping trip".to_string()
        ).unwrap();
        
        assert_eq!(expense.amount(), 42.50);
        assert_eq!(expense.category().name(), "Groceries");
        assert_eq!(expense.category().description(), Some("Food and household items"));
        assert_eq!(expense.date(), &date);
        assert_eq!(expense.description(), "Weekly shopping trip");
    }
    
    #[test]
    fn reject_empty_category_name() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();

        let result = Expense::with_category_name(
            42.50, 
            "",
            None,
            date, 
            "Weekly shopping trip".to_string()
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Category name cannot be empty"));
    }
    
    #[test]
    fn expense_equality() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();
        let category1 = Category::new("Groceries", None).unwrap();
        let category2 = Category::new("Groceries", None).unwrap();

        let expense1 = Expense::new(
            42.50, 
            category1, 
            date, 
            "Weekly shopping trip".to_string()
        );
        
        let expense2 = Expense::new(
            42.50, 
            category2, 
            date, 
            "Weekly shopping trip".to_string()
        );
        
        assert_eq!(expense1, expense2);
    }

    #[test]
    fn validate_expense_amount() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();
        let category = Category::new("Groceries", None).unwrap();
        
        // Test that negative amounts are rejected
        let result = Expense::new_validated(
            -50.0,
            category.clone(),
            date,
            "Weekly shopping".to_string()
        );
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid expense amount: amount cannot be negative"
        );
        
        // Test that zero amount is allowed
        let result = Expense::new_validated(
            0.0,
            category,
            date,
            "Free item".to_string()
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn validate_expense_date() {
        // Test that future dates are rejected (if that's a business rule)
        let future_date = chrono::Local::now().naive_local().date() + chrono::Duration::days(10);
        let category = Category::new("Groceries", None).unwrap();
        
        let result = Expense::new_validated(
            50.0,
            category,
            future_date,
            "Future shopping".to_string()
        );
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid expense date: date cannot be in the future"
        );
    }
    
    #[test]
    fn serialize_expense() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();
        let category = Category::new("Groceries", Some("Food and household items")).unwrap();

        let expense = Expense::new(
            42.50,
            category,
            date,
            "Weekly shopping trip".to_string()
        );
        
        let serialized = serde_json::to_string(&expense).unwrap();
        
        // Verify the JSON contains expected fields
        assert!(serialized.contains("amount"));
        assert!(serialized.contains("42.5"));
        assert!(serialized.contains("Groceries"));
        assert!(serialized.contains("Food and household items"));
        assert!(serialized.contains("2025-04-11"));
        assert!(serialized.contains("Weekly shopping trip"));
    }
    
    #[test]
    fn deserialize_expense() {
        let json = r#"{
            "id": null,
            "amount": 42.50,
            "category": {
                "name": "Groceries",
                "description": "Food and household items"
            },
            "date": "2025-04-11",
            "description": "Weekly shopping trip"
        }"#;
        
        let expense: Expense = serde_json::from_str(json).unwrap();
        
        assert_eq!(expense.amount(), 42.50);
        assert_eq!(expense.category().name(), "Groceries");
        assert_eq!(expense.category().description(), Some("Food and household items"));
        assert_eq!(
            expense.date(), 
            &NaiveDate::from_ymd_opt(2025, 4, 11).unwrap()
        );
        assert_eq!(expense.description(), "Weekly shopping trip");
    }
    
    #[test]
    fn roundtrip_serialization() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();
        let category = Category::new("Groceries", Some("Food and household items")).unwrap();

        let original = Expense::new(
            42.50,
            category,
            date,
            "Weekly shopping trip".to_string()
        );
        
        // Serialize to JSON
        let serialized = serde_json::to_string(&original).unwrap();
        
        // Deserialize back to Expense
        let deserialized: Expense = serde_json::from_str(&serialized).unwrap();
        
        // Original and deserialized should be equal
        assert_eq!(original, deserialized);
    }
}
