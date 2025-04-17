use clap::{Parser, Subcommand, Args};
use std::path::PathBuf;
use crate::models::category::CategoryRegistry;

#[derive(Parser)]
#[command(name = "expense_log")]
#[command(about = "A simple CLI expense tracker")]
#[command(version)]
pub struct Cli {
    /// Path to the config file
    #[arg(short, long, default_value = "expense_log.yaml")]
    pub config: PathBuf,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new expense
    Add(AddArgs),
    
    /// List expenses with optional filtering
    List(ListArgs),
    
    /// Show summary and statistics
    Summary(SummaryArgs),
    
    /// Manage expense categories
    Category(CategoryArgs),
}

#[derive(Args, Clone)]
pub struct AddArgs {
    /// Amount spent
    pub amount: f64,
    
    /// Expense category
    pub category: String,
    
    /// Date of expense (YYYY-MM-DD format)
    #[arg(short = 't', long)]
    pub date: Option<String>,
    
    /// Description of the expense
    #[arg(short, long)]
    pub description: Option<String>,
}

#[derive(Args, Clone)]
pub struct ListArgs {
    /// Filter by category
    #[arg(short, long)]
    pub category: Option<String>,
    
    /// Start date (YYYY-MM-DD format)
    #[arg(long)]
    pub from: Option<String>,
    
    /// End date (YYYY-MM-DD format)
    #[arg(long)]
    pub to: Option<String>,
    
    /// Limit number of results
    #[arg(short, long)]
    pub limit: Option<usize>,
}

#[derive(Args, Clone)]
pub struct SummaryArgs {
    /// Start date (YYYY-MM-DD format)
    #[arg(long)]
    pub from: Option<String>,
    
    /// End date (YYYY-MM-DD format)
    #[arg(long)]
    pub to: Option<String>,
    
    /// Group by category
    #[arg(long)]
    pub by_category: bool,
    
    /// Group by month
    #[arg(long)]
    pub by_month: bool,
}

#[derive(Args, Clone)]
pub struct CategoryArgs {
    #[command(subcommand)]
    pub command: CategoryCommands,
}

#[derive(Subcommand, Clone)]
pub enum CategoryCommands {
    /// List all available categories
    List,
    
    /// Add a new category
    Add {
        /// Category name
        name: String,
        
        /// Category description
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// Remove an existing category
    Remove {
        /// Category name
        name: String,
    },
}

/// Helper functions for parsing and validating CLI arguments
pub mod helpers {
    use super::*;
    use chrono::{Local, NaiveDate};
    use thiserror::Error;
    
    #[derive(Debug, Error)]
    pub enum CliError {
        #[error("Invalid date format: {0}")]
        InvalidDate(String),
        
        #[error("Category not found: {0}")]
        CategoryNotFound(String),
        
        #[error("Invalid amount: {0}")]
        InvalidAmount(String),
    }
    
    /// Parse a date string or use today's date
    pub fn parse_date(date_str: Option<String>) -> Result<NaiveDate, CliError> {
        match date_str {
            Some(date_str) => {
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|_| CliError::InvalidDate(format!("Could not parse date: {}", date_str)))
            },
            None => Ok(Local::now().naive_local().date()),
        }
    }
    
    /// Validate that a category exists
    pub fn validate_category(category_name: &str, registry: &CategoryRegistry) -> Result<(), CliError> {
        if !registry.category_exists(category_name) {
            return Err(CliError::CategoryNotFound(category_name.to_string()));
        }
        
        Ok(())
    }
    
    /// Validate amount is positive
    pub fn validate_amount(amount: f64) -> Result<(), CliError> {
        if amount < 0.0 {
            return Err(CliError::InvalidAmount("Amount cannot be negative".to_string()));
        }
        
        Ok(())
    }
    
    /// Get default description if none provided
    pub fn default_description(description: Option<String>, category: &str) -> String {
        description.unwrap_or_else(|| format!("Expense in {}", category))
    }
    
    /// Parse a date range or use reasonable defaults
    pub fn parse_date_range(from: Option<String>, to: Option<String>) -> Result<(NaiveDate, NaiveDate), CliError> {
        let today = Local::now().naive_local().date();
        
        // Default "from" is 30 days ago
        let from_date = match from {
            Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|_| CliError::InvalidDate(format!("Could not parse 'from' date: {}", date_str)))?,
            None => today - chrono::Duration::days(30),
        };
        
        // Default "to" is today
        let to_date = match to {
            Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|_| CliError::InvalidDate(format!("Could not parse 'to' date: {}", date_str)))?,
            None => today,
        };
        
        // Ensure "from" is not after "to"
        if from_date > to_date {
            return Err(CliError::InvalidDate("'from' date must be before 'to' date".to_string()));
        }
        
        Ok((from_date, to_date))
    }
}
