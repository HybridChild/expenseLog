use crate::models::expense::Expense;
use chrono::NaiveDate;
use super::error::RepositoryError;

/// Defines the interface for expense storage operations
pub trait ExpenseRepository {
    /// Save a new expense or update an existing one
    /// If expense.id() is None, a new expense is created
    /// Otherwise, the expense with the given ID is updated
    fn save(&self, expense: &mut Expense) -> Result<(), RepositoryError>;
    
    /// Get an expense by its ID
    fn get_by_id(&self, id: i64) -> Result<Option<Expense>, RepositoryError>;
    
    /// Get all expenses
    fn get_all(&self) -> Result<Vec<Expense>, RepositoryError>;
    
    /// Get expenses by category name
    fn get_by_category(&self, category_name: &str) -> Result<Vec<Expense>, RepositoryError>;
    
    /// Get expenses within a date range (inclusive)
    fn get_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<Expense>, RepositoryError>;
    
    /// Delete an expense by ID
    /// Returns true if an expense was deleted, false if no expense with that ID was found
    fn delete(&self, id: i64) -> Result<bool, RepositoryError>;
    
    /// Get total expenses for a specific category within a date range
    fn get_category_total(&self, category_name: &str, start: NaiveDate, end: NaiveDate) -> Result<f64, RepositoryError>;
    
    /// Get monthly averages by category for a given date range
    fn get_monthly_category_averages(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<(String, f64)>, RepositoryError>;
}
