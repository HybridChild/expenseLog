use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expense {
    id: Option<i64>,
    amount: f64,
    category: String, // We'll refine this to a Category type later
    date: NaiveDate,
    description: String,
}

impl Expense {
    pub fn new(amount: f64, category: String, date: NaiveDate, description: String) -> Self {
        Self {
            id: None,
            amount,
            category,
            date,
            description,
        }
    }
    
    pub fn id(&self) -> Option<i64> {
        self.id
    }
    
    pub fn amount(&self) -> f64 {
        self.amount
    }
    
    pub fn category(&self) -> &str {
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
    use chrono::NaiveDate;

    #[test]
    fn create_expense() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();

        let expense = super::Expense::new(
            42.50, 
            "Groceries".to_string(), 
            date, 
            "Weekly shopping trip".to_string()
        );
        
        assert_eq!(expense.amount(), 42.50);
        assert_eq!(expense.category(), "Groceries");
        assert_eq!(expense.date(), &date);
        assert_eq!(expense.description(), "Weekly shopping trip");
    }
    
    #[test]
    fn expense_equality() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 11).unwrap();

        let expense1 = super::Expense::new(
            42.50, 
            "Groceries".to_string(), 
            date, 
            "Weekly shopping trip".to_string()
        );
        
        let expense2 = super::Expense::new(
            42.50, 
            "Groceries".to_string(), 
            date, 
            "Weekly shopping trip".to_string()
        );
        
        assert_eq!(expense1, expense2);
    }
}
