use std::process;
use clap::Parser;

use expense_log::app::App;
use expense_log::cli::{Cli, Commands};
use expense_log::config::Config;
use expense_log::repository::SqliteExpenseRepository;

fn main() {
    let cli = Cli::parse();
    
    // Load config
    let config = match Config::load(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            process::exit(1);
        }
    };
    
    // Initialize repository
    let repository = match SqliteExpenseRepository::new(&config.database_path) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            process::exit(1);
        }
    };
    
    // Create app instance
    let mut app = App::new(repository, config);
    
    // Process commands
    let result = match &cli.command {
        Some(Commands::Add(args)) => app.add_expense(args.clone()),
        Some(Commands::List(args)) => app.list_expenses(args.clone()),
        Some(Commands::Summary(args)) => app.generate_summary(args.clone()),
        Some(Commands::Category(args)) => app.manage_categories(args.clone()),
        None => {
            // No command specified, show usage
            println!("expense_log - A simple expense tracking CLI");
            println!("\nUsage examples:");
            println!("  expense_log add 42.50 Food --date 2025-04-15 --description \"Groceries\"");
            println!("  expense_log list --category Food");
            println!("  expense_log summary --from 2025-01-01 --to 2025-03-31 --by-category");
            println!("  expense_log category list");
            println!("\nFor more details, run: expense_log --help");
            Ok(())
        }
    };
    
    // Handle any errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
