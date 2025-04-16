use std::path::Path;
use rusqlite::{Connection, params, types::Type};
use chrono::{NaiveDate, Datelike};

use crate::models::expense::Expense;
use crate::models::category::Category;
use crate::repository::{ExpenseRepository, RepositoryError};
use super::schema;

pub struct SqliteExpenseRepository {
    conn: Connection,
}

impl SqliteExpenseRepository {
    /// Create a new SQLite repository with the given database file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, RepositoryError> {
        let conn = Connection::open(path)?;
        
        // Initialize schema
        schema::initialize_schema(&conn)?;
        
        Ok(Self { conn })
    }
    
    /// Create a new in-memory SQLite repository (useful for testing)
    pub fn new_in_memory() -> Result<Self, RepositoryError> {
        let conn = Connection::open_in_memory()?;
        
        // Initialize schema
        schema::initialize_schema(&conn)?;
        
        Ok(Self { conn })
    }
}

impl ExpenseRepository for SqliteExpenseRepository {
    fn save(&self, expense: &mut Expense) -> Result<(), RepositoryError> {
        if expense.id().is_none() {
            // Insert new expense
            let result = self.conn.execute(
                "INSERT INTO expenses (amount, category, category_description, date, description) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    expense.amount(),
                    expense.category().name(),
                    expense.category().description(),
                    expense.date().to_string(),
                    expense.description(),
                ],
            )?;
            
            if result > 0 {
                // Get the last inserted ID
                let id = self.conn.last_insert_rowid();
                expense.set_id(id);
            }
        } else {
            // Update existing expense
            self.conn.execute(
                "UPDATE expenses SET 
                 amount = ?1, 
                 category = ?2, 
                 category_description = ?3,
                 date = ?4, 
                 description = ?5 
                 WHERE id = ?6",
                params![
                    expense.amount(),
                    expense.category().name(),
                    expense.category().description(),
                    expense.date().to_string(),
                    expense.description(),
                    expense.id().unwrap(),
                ],
            )?;
        }
        
        Ok(())
    }
    
    fn get_by_id(&self, id: i64) -> Result<Option<Expense>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, category, category_description, date, description 
             FROM expenses 
             WHERE id = ?1"
        )?;
        
        let expense_result = stmt.query_row(
            params![id],
            |row| {
                let id = row.get(0)?;
                let amount = row.get(1)?;
                let category_name: String = row.get(2)?;
                let category_description: Option<String> = row.get(3)?;
                let date_str: String = row.get(4)?;
                let description: String = row.get(5)?;
                
                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|_| rusqlite::Error::InvalidColumnType(4, "Invalid date format".to_string(), Type::Text))?;
                
                let category = Category::new(
                    &category_name, 
                    category_description.as_deref()
                ).map_err(|_| rusqlite::Error::InvalidColumnType(2, "Invalid category".to_string(), Type::Text))?;
                
                let expense = Expense::new(amount, category, date, description).with_id(id);
                
                Ok(expense)
            },
        );
        
        match expense_result {
            Ok(expense) => Ok(Some(expense)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(RepositoryError::DatabaseError(e)),
        }
    }
    
    fn get_all(&self) -> Result<Vec<Expense>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, category, category_description, date, description 
             FROM expenses 
             ORDER BY date DESC"
        )?;
        
        let expense_iter = stmt.query_map([], |row| {
            let id = row.get(0)?;
            let amount = row.get(1)?;
            let category_name: String = row.get(2)?;
            let category_description: Option<String> = row.get(3)?;
            let date_str: String = row.get(4)?;
            let description: String = row.get(5)?;
            
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "Invalid date format".to_string(), Type::Text))?;
            
            let category = Category::new(
                &category_name, 
                category_description.as_deref()
            ).map_err(|_| rusqlite::Error::InvalidColumnType(2, "Invalid category".to_string(), Type::Text))?;
            
            let expense = Expense::new(amount, category, date, description).with_id(id);
            
            Ok(expense)
        })?;
        
        let mut expenses = Vec::new();
        for expense_result in expense_iter {
            expenses.push(expense_result?);
        }
        
        Ok(expenses)
    }
    
    fn get_by_category(&self, category_name: &str) -> Result<Vec<Expense>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, category, category_description, date, description 
             FROM expenses 
             WHERE category = ?1 
             ORDER BY date DESC"
        )?;
        
        let expense_iter = stmt.query_map(params![category_name], |row| {
            let id = row.get(0)?;
            let amount = row.get(1)?;
            let category_name: String = row.get(2)?;
            let category_description: Option<String> = row.get(3)?;
            let date_str: String = row.get(4)?;
            let description: String = row.get(5)?;
            
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "Invalid date format".to_string(), Type::Text))?;
            
            let category = Category::new(
                &category_name, 
                category_description.as_deref()
            ).map_err(|_| rusqlite::Error::InvalidColumnType(2, "Invalid category".to_string(), Type::Text))?;
            
            let expense = Expense::new(amount, category, date, description).with_id(id);
            
            Ok(expense)
        })?;
        
        let mut expenses = Vec::new();
        for expense_result in expense_iter {
            expenses.push(expense_result?);
        }
        
        Ok(expenses)
    }
    
    fn get_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<Expense>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, category, category_description, date, description 
             FROM expenses 
             WHERE date >= ?1 AND date <= ?2 
             ORDER BY date DESC"
        )?;
        
        let expense_iter = stmt.query_map(params![start.to_string(), end.to_string()], |row| {
            let id = row.get(0)?;
            let amount = row.get(1)?;
            let category_name: String = row.get(2)?;
            let category_description: Option<String> = row.get(3)?;
            let date_str: String = row.get(4)?;
            let description: String = row.get(5)?;
            
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "Invalid date format".to_string(), Type::Text))?;
            
            let category = Category::new(
                &category_name, 
                category_description.as_deref()
            ).map_err(|_| rusqlite::Error::InvalidColumnType(2, "Invalid category".to_string(), Type::Text))?;
            
            let expense = Expense::new(amount, category, date, description).with_id(id);
            
            Ok(expense)
        })?;
        
        let mut expenses = Vec::new();
        for expense_result in expense_iter {
            expenses.push(expense_result?);
        }
        
        Ok(expenses)
    }
    
    fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let affected = self.conn.execute("DELETE FROM expenses WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }
    
    fn get_category_total(&self, category_name: &str, start: NaiveDate, end: NaiveDate) -> Result<f64, RepositoryError> {
        let total: f64 = self.conn.query_row(
            "SELECT COALESCE(SUM(amount), 0.0) 
             FROM expenses 
             WHERE category = ?1 AND date >= ?2 AND date <= ?3",
            params![category_name, start.to_string(), end.to_string()],
            |row| row.get(0)
        )?;
        
        Ok(total)
    }
    
    fn get_monthly_category_averages(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<(String, f64)>, RepositoryError> {
        // Calculate number of months in the date range
        let months = (end.year() * 12 + end.month() as i32) - (start.year() * 12 + start.month() as i32) + 1;
        
        if months <= 0 {
            return Ok(Vec::new());
        }
        
        // Get total per category
        let mut stmt = self.conn.prepare(
            "SELECT category, SUM(amount) 
             FROM expenses 
             WHERE date >= ?1 AND date <= ?2 
             GROUP BY category"
        )?;
        
        let rows = stmt.query_map(
            params![start.to_string(), end.to_string()],
            |row| {
                let category: String = row.get(0)?;
                let total: f64 = row.get(1)?;
                Ok((category, total))
            },
        )?;
        
        let mut averages = Vec::new();
        for result in rows {
            let (category, total) = result?;
            let monthly_avg = total / (months as f64);
            averages.push((category, monthly_avg));
        }
        
        Ok(averages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    fn create_test_repository() -> SqliteExpenseRepository {
        SqliteExpenseRepository::new_in_memory().unwrap()
    }
    
    fn create_test_expense(amount: f64, category_name: &str, date_str: &str, description: &str) -> Expense {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let category = Category::new(category_name, None).unwrap();
        Expense::new(amount, category, date, description.to_string())
    }
    
    #[test]
    fn test_save_and_get_expense() {
        let repo = create_test_repository();
        let mut expense = create_test_expense(42.50, "Food", "2025-04-11", "Weekly shopping");
        
        // Save the expense - should assign an ID
        repo.save(&mut expense).unwrap();
        
        // Verify ID was assigned
        assert!(expense.id().is_some());
        
        // Fetch the expense by ID
        let fetched = repo.get_by_id(expense.id().unwrap()).unwrap().unwrap();
        
        // Verify fetched data matches original
        assert_eq!(fetched.id(), expense.id());
        assert_eq!(fetched.amount(), 42.50);
        assert_eq!(fetched.category().name(), "Food");
        assert_eq!(fetched.date().to_string(), "2025-04-11");
        assert_eq!(fetched.description(), "Weekly shopping");
    }
    
    #[test]
    fn test_update_expense() {
        let repo = create_test_repository();
        let mut expense = create_test_expense(42.50, "Food", "2025-04-11", "Weekly shopping");
        
        // Save the expense - should assign an ID
        repo.save(&mut expense).unwrap();
        let id = expense.id().unwrap();
        
        // Update the expense
        let category = Category::new("Groceries", Some("Supermarket")).unwrap();
        expense.set_category(category);
        expense.set_amount(55.75).unwrap();
        
        // Save the updated expense
        repo.save(&mut expense).unwrap();
        
        // Fetch the expense by ID
        let fetched = repo.get_by_id(id).unwrap().unwrap();
        
        // Verify updated data
        assert_eq!(fetched.amount(), 55.75);
        assert_eq!(fetched.category().name(), "Groceries");
        assert_eq!(fetched.category().description(), Some("Supermarket"));
    }
    
    #[test]
    fn test_get_by_category() {
        let repo = create_test_repository();
        
        // Create and save expenses with different categories
        let mut food_expense = create_test_expense(42.50, "Food", "2025-04-11", "Weekly shopping");
        let mut rent_expense = create_test_expense(1200.00, "Housing", "2025-04-01", "Monthly rent");
        let mut utility_expense = create_test_expense(85.75, "Utilities", "2025-04-05", "Electricity");
        
        repo.save(&mut food_expense).unwrap();
        repo.save(&mut rent_expense).unwrap();
        repo.save(&mut utility_expense).unwrap();
        
        // Get expenses by category
        let food_expenses = repo.get_by_category("Food").unwrap();
        let housing_expenses = repo.get_by_category("Housing").unwrap();
        
        // Verify category filtering
        assert_eq!(food_expenses.len(), 1);
        assert_eq!(food_expenses[0].amount(), 42.50);
        
        assert_eq!(housing_expenses.len(), 1);
        assert_eq!(housing_expenses[0].amount(), 1200.00);
    }
    
    #[test]
    fn test_get_by_date_range() {
        let repo = create_test_repository();
        
        // Create and save expenses with different dates
        let mut expense1 = create_test_expense(42.50, "Food", "2025-03-15", "March shopping");
        let mut expense2 = create_test_expense(55.75, "Food", "2025-04-05", "April shopping");
        let mut expense3 = create_test_expense(60.25, "Food", "2025-04-20", "Late April shopping");
        
        repo.save(&mut expense1).unwrap();
        repo.save(&mut expense2).unwrap();
        repo.save(&mut expense3).unwrap();
        
        // Date range for April only
        let start = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 4, 30).unwrap();
        
        let april_expenses = repo.get_by_date_range(start, end).unwrap();
        
        // Should include expense2 and expense3, but not expense1
        assert_eq!(april_expenses.len(), 2);
        
        // Check that dates are within range
        for expense in april_expenses {
            assert!(expense.date() >= &start);
            assert!(expense.date() <= &end);
        }
    }
    
    #[test]
    fn test_delete_expense() {
        let repo = create_test_repository();
        let mut expense = create_test_expense(42.50, "Food", "2025-04-11", "Weekly shopping");
        
        // Save the expense
        repo.save(&mut expense).unwrap();
        let id = expense.id().unwrap();
        
        // Delete the expense
        let deleted = repo.delete(id).unwrap();
        assert!(deleted);
        
        // Verify it's no longer in the repository
        let result = repo.get_by_id(id).unwrap();
        assert!(result.is_none());
        
        // Try deleting non-existent expense
        let deleted = repo.delete(999).unwrap();
        assert!(!deleted);
    }
    
    #[test]
    fn test_get_category_total() {
        let repo = create_test_repository();
        
        // Create and save multiple expenses in the same category
        let mut expense1 = create_test_expense(42.50, "Food", "2025-04-05", "Week 1");
        let mut expense2 = create_test_expense(38.25, "Food", "2025-04-12", "Week 2");
        let mut expense3 = create_test_expense(45.00, "Food", "2025-04-19", "Week 3");
        let mut expense4 = create_test_expense(39.75, "Food", "2025-04-26", "Week 4");
        
        repo.save(&mut expense1).unwrap();
        repo.save(&mut expense2).unwrap();
        repo.save(&mut expense3).unwrap();
        repo.save(&mut expense4).unwrap();
        
        // Calculate total for the month
        let start = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 4, 30).unwrap();
        
        let total = repo.get_category_total("Food", start, end).unwrap();
        
        // Should be the sum of all food expenses
        assert_eq!(total, 42.50 + 38.25 + 45.00 + 39.75);
    }
    
    #[test]
    fn test_get_monthly_category_averages() {
        let repo = create_test_repository();
        
        // Create expenses across different months and categories
        let mut expense1 = create_test_expense(100.00, "Food", "2025-03-15", "March food");
        let mut expense2 = create_test_expense(200.00, "Food", "2025-04-15", "April food");
        let mut expense3 = create_test_expense(300.00, "Housing", "2025-03-01", "March rent");
        let mut expense4 = create_test_expense(300.00, "Housing", "2025-04-01", "April rent");
        
        repo.save(&mut expense1).unwrap();
        repo.save(&mut expense2).unwrap();
        repo.save(&mut expense3).unwrap();
        repo.save(&mut expense4).unwrap();
        
        // Get monthly averages for the two-month period
        let start = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 4, 30).unwrap();
        
        let averages = repo.get_monthly_category_averages(start, end).unwrap();
        
        // Convert to a map for easier testing
        let mut avg_map = std::collections::HashMap::new();
        for (category, avg) in averages {
            avg_map.insert(category, avg);
        }
        
        // Check food average: (100 + 200) / 2 months = 150
        assert!(avg_map.contains_key("Food"));
        assert!((avg_map["Food"] - 150.0).abs() < 0.001);
        
        // Check housing average: (300 + 300) / 2 months = 300
        assert!(avg_map.contains_key("Housing"));
        assert!((avg_map["Housing"] - 300.0).abs() < 0.001);
    }
}
