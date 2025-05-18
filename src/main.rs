use std::env;
use vddb::{create_database, Repl, DbError};
use log::{info, error};

fn main() -> Result<(), DbError> {
    // Initialize logging
    env_logger::init();
    info!("Starting VDDB application");

    // Get data directory from environment or use default
    let data_dir = env::var("VDDB_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    
    // Create database with all components
    let (schema, storage, tx_manager, plugin_manager) = create_database(&data_dir)?;
    
    // Create and run REPL
    let mut repl = Repl::new(schema, storage, tx_manager, plugin_manager)?;
    
    match repl.run() {
        Ok(_) => {
            info!("REPL shutdown successfully");
            Ok(())
        }
        Err(e) => {
            error!("REPL error: {}", e);
            Err(e)
        }
    }
}