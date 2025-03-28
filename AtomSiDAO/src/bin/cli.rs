//! AtomSi DAO Command Line Interface
//!
//! This binary provides a command-line interface for interacting with the AtomSi DAO framework.

use atomsi_dao::{self, config::ConfigManager, DAOContext};
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "init" => {
            if args.len() < 3 {
                println!("Error: Missing configuration file path");
                print_usage();
                process::exit(1);
            }
            
            let config_path = &args[2];
            init_dao(config_path).await;
        }
        "run" => {
            if args.len() < 3 {
                println!("Error: Missing configuration file path");
                print_usage();
                process::exit(1);
            }
            
            let config_path = &args[2];
            run_dao(config_path).await;
        }
        "create-config" => {
            if args.len() < 3 {
                println!("Error: Missing output file path");
                print_usage();
                process::exit(1);
            }
            
            let output_path = &args[2];
            create_config(output_path);
        }
        "version" => {
            println!("{} version {}", atomsi_dao::NAME, atomsi_dao::VERSION);
        }
        "help" => {
            print_usage();
        }
        _ => {
            println!("Error: Unknown command '{}'", command);
            print_usage();
            process::exit(1);
        }
    }
}

/// Print usage information
fn print_usage() {
    println!("AtomSi DAO - A comprehensive framework for building decentralized autonomous organizations");
    println!();
    println!("USAGE:");
    println!("  atomsi_dao_cli [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  init [CONFIG_FILE]        Initialize the DAO using configuration from CONFIG_FILE");
    println!("  run [CONFIG_FILE]         Run the DAO using configuration from CONFIG_FILE");
    println!("  create-config [OUT_FILE]  Create a default configuration and save it to OUT_FILE");
    println!("  version                   Print version information");
    println!("  help                      Print this help message");
}

/// Initialize the DAO
async fn init_dao(config_path: &str) {
    println!("Initializing DAO from configuration file: {}", config_path);
    
    match atomsi_dao::init(config_path).await {
        Ok(context) => {
            println!("DAO initialized successfully");
            
            // Initialize database schema
            match context.db_manager.init_schema().await {
                Ok(_) => println!("Database schema initialized successfully"),
                Err(e) => {
                    println!("Error initializing database schema: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("Error initializing DAO: {}", e);
            process::exit(1);
        }
    }
}

/// Run the DAO
async fn run_dao(config_path: &str) {
    println!("Running DAO from configuration file: {}", config_path);
    
    match atomsi_dao::init(config_path).await {
        Ok(context) => {
            println!("DAO initialized successfully");
            
            // TODO: Implement API server or other runtime logic
            println!("DAO is running...");
            
            // Keep the application running
            tokio::signal::ctrl_c().await.unwrap();
            println!("Shutting down...");
            
            // Graceful shutdown
            match atomsi_dao::shutdown(context).await {
                Ok(_) => println!("DAO shutdown successfully"),
                Err(e) => {
                    println!("Error during shutdown: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("Error initializing DAO: {}", e);
            process::exit(1);
        }
    }
}

/// Create a default configuration
fn create_config(output_path: &str) {
    println!("Creating default configuration at: {}", output_path);
    
    let config_manager = ConfigManager::with_defaults(output_path);
    
    match config_manager.save_to_file(output_path) {
        Ok(_) => println!("Configuration file created successfully"),
        Err(e) => {
            println!("Error creating configuration file: {}", e);
            process::exit(1);
        }
    }
} 