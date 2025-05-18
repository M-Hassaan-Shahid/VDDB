use crate::types::{DbError, Value};
use std::any::Any;
use std::collections::HashMap;

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self) -> Result<(), DbError>;
    fn shutdown(&mut self) -> Result<(), DbError>;
    fn execute(&self, command: &str, args: &[Value]) -> Result<Value, DbError>;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<(), DbError> {
        let name = plugin.name().to_string();
        if self.plugins.contains_key(&name) {
            return Err(DbError::ConfigurationError(format!("Plugin {} already registered", name)));
        }
        plugin.initialize()?;
        self.plugins.insert(name, plugin);
        Ok(())
    }

    pub fn unregister_plugin(&mut self, name: &str) -> Result<(), DbError> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.shutdown()?;
            Ok(())
        } else {
            Err(DbError::ConfigurationError(format!("Plugin {} not found", name)))
        }
    }

    pub fn execute_plugin(&self, name: &str, command: &str, args: &[Value]) -> Result<Value, DbError> {
        self.plugins
            .get(name)
            .ok_or_else(|| DbError::ConfigurationError(format!("Plugin {} not found", name)))?
            .execute(command, args)
    }

    pub fn list_plugins(&self) -> Vec<(&str, &str)> {
        self.plugins
            .iter()
            .map(|(name, plugin)| (name.as_str(), plugin.version()))
            .collect()
    }
}

// Example plugin implementation
pub struct ExamplePlugin {
    name: String,
    version: String,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        ExamplePlugin {
            name: "example".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn initialize(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    fn execute(&self, command: &str, args: &[Value]) -> Result<Value, DbError> {
        match command {
            "echo" => Ok(args.get(0).cloned().unwrap_or(Value::String("".to_string()))),
            _ => Err(DbError::QueryError(format!("Unknown command: {}", command))),
        }
    }
} 