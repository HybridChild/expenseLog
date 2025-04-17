use chrono::{NaiveDate, Datelike};
use std::io::{self, Write};
use std::path::Path;
use thiserror::Error;

use crate::cli::{AddArgs, ListArgs, SummaryArgs, CategoryArgs, CategoryCommands};
use crate::cli::helpers::{parse_date, validate_category, validate_amount, default_description, parse_date_range};
use crate::models::category::CategoryRegistry;
use crate::models::expense::Expense;
use crate::repository::{ExpenseRepository, RepositoryError};
use crate::config::Config;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    
    #[error("CLI error: {0}")]
    CliError(#[from] crate::cli::helpers::CliError),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    
    #[error("{0}")]
    Other(String),
}

pub struct App<R: ExpenseRepository> {
    repository: R,
    category_registry: CategoryRegistry,
    config: Config,
}

impl<R: ExpenseRepository> App<R> {
    pub fn new(repository: R, config: Config) -> Self {
        let mut category_registry = CategoryRegistry::new();
        config.configure_category_registry(&mut category_registry);
        
        Self {
            repository,
            category_registry,
            config,
        }
    }
    
    pub fn add_expense(&self, args: AddArgs) -> Result<(), AppError> {
        // Validate inputs
        validate_amount(args.amount)?;
        validate_category(&args.category, &self.category_registry)?;
        let date = parse_date(args.date)?;
        let description = default_description(args.description, &args.category);
        
        // Get the category from registry
        let category = self.category_registry.get_category(&args.category)
            .ok_or_else(|| AppError::Other(format!("Category not found: {}", args.category)))?;
        
        // Create expense
        let mut expense = Expense::new(
            args.amount,
            category.clone(),
            date,
            description,
        );
        
        // Save to repository
        self.repository.save(&mut expense)?;
        
        println!("Expense added: {} {} for {} on {}", 
            self.config.currency_symbol, 
            expense.amount(), 
            expense.description(),
            expense.date());
        
        Ok(())
    }
    
    pub fn list_expenses(&self, args: ListArgs) -> Result<(), AppError> {
        let expenses = if let Some(category) = args.category {
            validate_category(&category, &self.category_registry)?;
            self.repository.get_by_category(&category)?
        } else if args.from.is_some() || args.to.is_some() {
            let (from_date, to_date) = parse_date_range(args.from, args.to)?;
            self.repository.get_by_date_range(from_date, to_date)?
        } else {
            self.repository.get_all()?
        };
        
        // Apply limit if provided
        let expenses = if let Some(limit) = args.limit {
            expenses.into_iter().take(limit).collect()
        } else {
            expenses
        };
        
        if expenses.is_empty() {
            println!("No expenses found matching the criteria.");
            return Ok(());
        }
        
        // Print header
        println!("{:<5} {:<10} {:<15} {:<10} {:<30}", "ID", "Date", "Category", "Amount", "Description");
        println!("{}", "-".repeat(75));
        
        // Print each expense
        let mut total = 0.0;
        for expense in &expenses {
            println!("{:<5} {:<10} {:<15} {:<10.2} {:<30}",
                expense.id().unwrap_or(0),
                expense.date(),
                expense.category().name(),
                expense.amount(),
                expense.description()
            );
            total += expense.amount();
        }
        
        // Print footer with total
        println!("{}", "-".repeat(75));
        println!("Total: {} {:.2} ({} items)", self.config.currency_symbol, total, expenses.len());
        
        Ok(())
    }
    
    pub fn generate_summary(&self, args: SummaryArgs) -> Result<(), AppError> {
        let (from_date, to_date) = parse_date_range(args.from, args.to)?;
        
        println!("Expense Summary ({} to {})", from_date, to_date);
        println!("{}", "-".repeat(50));
        
        if args.by_category {
            self.summary_by_category(from_date, to_date)?;
        } else if args.by_month {
            self.summary_by_month(from_date, to_date)?;
        } else {
            // Default summary shows both
            self.summary_by_category(from_date, to_date)?;
            println!();
            self.summary_by_month(from_date, to_date)?;
        }
        
        // Show monthly averages
        println!();
        println!("Monthly Averages by Category:");
        println!("{}", "-".repeat(50));
        
        let averages = self.repository.get_monthly_category_averages(from_date, to_date)?;
        
        if averages.is_empty() {
            println!("No data available for the selected period.");
            return Ok(());
        }
        
        // Sort averages by amount (descending)
        let mut sorted_averages = averages;
        sorted_averages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        for (category, avg) in sorted_averages {
            println!("{:<20} {} {:.2}/month", category, self.config.currency_symbol, avg);
        }
        
        Ok(())
    }
    
    fn summary_by_category(&self, from_date: NaiveDate, to_date: NaiveDate) -> Result<(), AppError> {
        println!("Expenses by Category:");
        
        let mut total = 0.0;
        let mut category_totals = Vec::new();
        
        // Get totals for each category in registry
        for category in self.category_registry.all_categories() {
            let amount = self.repository.get_category_total(category.name(), from_date, to_date)?;
            
            if amount > 0.0 {
                category_totals.push((category.name().to_string(), amount));
                total += amount;
            }
        }
        
        // Sort by amount (descending)
        category_totals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Print results
        for (category, amount) in category_totals {
            let percentage = if total > 0.0 { (amount / total) * 100.0 } else { 0.0 };
            println!("{:<20} {} {:<10.2} ({:.1}%)", 
                category, 
                self.config.currency_symbol, 
                amount, 
                percentage
            );
        }
        
        println!("{}", "-".repeat(50));
        println!("Total: {} {:.2}", self.config.currency_symbol, total);
        
        Ok(())
    }
    
    fn summary_by_month(&self, from_date: NaiveDate, to_date: NaiveDate) -> Result<(), AppError> {
        println!("Expenses by Month:");
        
        // Get all expenses in date range
        let expenses = self.repository.get_by_date_range(from_date, to_date)?;
        
        if expenses.is_empty() {
            println!("No data available for the selected period.");
            return Ok(());
        }
        
        // Group by month
        let mut monthly_totals: std::collections::HashMap<(i32, u32), f64> = std::collections::HashMap::new();
        
        for expense in expenses {
            let key = (expense.date().year(), expense.date().month());
            *monthly_totals.entry(key).or_insert(0.0) += expense.amount();
        }
        
        // Convert to vector and sort by date
        let mut sorted_totals: Vec<_> = monthly_totals.into_iter().collect();
        sorted_totals.sort_by_key(|&((year, month), _)| (year, month));
        
        // Print results
        let mut total = 0.0;
        for ((year, month), amount) in sorted_totals {
            let month_name = match month {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => "Unknown",
            };
            
            println!("{} {:<10} {} {:.2}", year, month_name, self.config.currency_symbol, amount);
            total += amount;
        }
        
        println!("{}", "-".repeat(50));
        println!("Total: {} {:.2}", self.config.currency_symbol, total);
        
        Ok(())
    }
    
    pub fn manage_categories(&mut self, args: CategoryArgs) -> Result<(), AppError> {
        match args.command {
            CategoryCommands::List => {
                println!("Available Categories:");
                println!("{}", "-".repeat(50));
                
                let categories = self.category_registry.all_categories();
                
                if categories.is_empty() {
                    println!("No categories defined.");
                    return Ok(());
                }
                
                for category in categories {
                    if let Some(desc) = category.description() {
                        println!("{:<20} - {}", category.name(), desc);
                    } else {
                        println!("{}", category.name());
                    }
                }
            },
            CategoryCommands::Add { name, description } => {
                // Add the category
                match self.category_registry.add_category(&name, description.as_deref()) {
                    Ok(category) => {
                        println!("Added category: {}", category.name());
                        
                        // Update the config and save it
                        self.update_config_categories()?;
                    },
                    Err(e) => {
                        return Err(AppError::Other(format!("Failed to add category: {}", e)));
                    }
                }
            },
            CategoryCommands::Remove { name } => {
                // First check if there are any expenses with this category
                if let Ok(expenses) = self.repository.get_by_category(&name) {
                    if !expenses.is_empty() {
                        // Ask for confirmation
                        print!("There are {} expenses with category '{}'. Are you sure you want to remove it? (y/N): ", 
                            expenses.len(), name);
                        io::stdout().flush()?;
                        
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        
                        if !input.trim().eq_ignore_ascii_case("y") {
                            println!("Operation cancelled.");
                            return Ok(());
                        }
                    }
                }
                
                // Remove the category
                match self.category_registry.remove_category(&name) {
                    Ok(_) => {
                        println!("Removed category: {}", name);
                        
                        // Update the config and save it
                        self.update_config_categories()?;
                    },
                    Err(e) => {
                        return Err(AppError::Other(format!("Failed to remove category: {}", e)));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    // Update config with the current categories and save it
    fn update_config_categories(&mut self) -> Result<(), AppError> {
        // Update config with current categories
        self.config.categories = self.category_registry.all_categories()
            .into_iter()
            .cloned()
            .collect();
        
        // Save config
        let config_path = Path::new("expense_log.yaml");
        self.config.save(&config_path)?;
        
        Ok(())
    }
}
