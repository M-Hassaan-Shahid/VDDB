use crate::{
    query::parser::parse_query,
    query::Query,
    query::planner::QueryEngine,
    schema::Schema,
    storage::StorageManager,
    transaction::{Transaction, TransactionManager},
    types::{DbError, Value},
    plugins::PluginManager,
};
use rustyline::{Editor, Config, CompletionType, error::ReadlineError};
use std::sync::{Arc, Mutex};
use std::fmt;
use ordered_float::OrderedFloat;

pub struct QueryResult(pub Vec<Vec<Value>>);

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.0 {
            for (i, value) in row.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", value)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int32(i) => write!(f, "{}", i),
            Value::Float32(f32) => write!(f, "{}", f32.0),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

pub struct Repl {
    editor: Editor<ReplHelper, rustyline::history::FileHistory>,
    schema: Schema,
    storage: Arc<Mutex<StorageManager>>,
    tx_manager: TransactionManager,
    plugin_manager: PluginManager,
    query_engine: QueryEngine,
}

impl Repl {
    pub fn new(
        schema: Schema,
        storage: Arc<Mutex<StorageManager>>,
        tx_manager: TransactionManager,
        plugin_manager: PluginManager,
    ) -> Result<Self, DbError> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .build();
        
        let mut editor = Editor::with_config(config).map_err(|e| DbError::QueryError(e.to_string()))?;
        editor.set_helper(Some(ReplHelper::new()));
        
        let query_engine = QueryEngine::new(storage.clone());
        
        Ok(Self {
            editor,
            schema,
            storage,
            tx_manager,
            plugin_manager,
            query_engine,
        })
    }

    pub fn run(&mut self) -> Result<(), DbError> {
        println!("VDDB Interactive Shell");
        println!("Type 'HELP' for help, 'EXIT' to quit");

        loop {
            match self.editor.readline("vddb> ") {
                Ok(line) => {
                    self.editor.add_history_entry(line.as_str());
                    
                    match line.trim().to_uppercase().as_str() {
                        "EXIT" | "QUIT" => break,
                        "HELP" => self.show_help(),
                        cmd if cmd.starts_with("PLUGIN ") => {
                            if let Err(e) = self.handle_plugin_command(&line[7..]) {
                                eprintln!("Plugin error: {}", e);
                            }
                        }
                        _ => {
                            if let Err(e) = self.execute_query(&line) {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn execute_query(&mut self, query: &str) -> Result<(), DbError> {
        let parsed = parse_query(query)?;
        match self.query_engine.execute(parsed) {
            Ok(results) => {
                if !results.is_empty() {
                    let query_result = QueryResult(results);
                    println!("{}", query_result);
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn handle_plugin_command(&mut self, cmd: &str) -> Result<(), DbError> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            println!("Available plugins:");
            for plugin in self.plugin_manager.list_plugins() {
                println!("- {}", plugin.0);
            }
            return Ok(());
        }

        let plugin_name = parts[0];
        let command = parts.get(1).unwrap_or(&"");
        let args = &parts[2..];
        
        match self.plugin_manager.execute_plugin(plugin_name, command, &args.iter().map(|&s| Value::String(s.to_string())).collect::<Vec<_>>()) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Plugin error: {}", e),
        }
        
        Ok(())
    }

    fn show_help(&self) {
        println!("VDDB Commands:");
        println!("  SELECT - Query data from tables");
        println!("  INSERT - Add new data to tables");
        println!("  UPDATE - Modify existing data");
        println!("  DELETE - Remove data from tables");
        println!("  CREATE TABLE - Create a new table");
        println!("  DROP TABLE - Remove a table");
        println!("  PLUGIN - Manage plugins");
        println!("  HELP - Show this help message");
        println!("  EXIT/QUIT - Exit the shell");
        println!("\nSQL Syntax Examples:");
        println!("  SELECT * FROM users WHERE age > 18");
        println!("  INSERT INTO users (name, age) VALUES ('John', 25)");
        println!("  UPDATE users SET age = 26 WHERE name = 'John'");
        println!("  DELETE FROM users WHERE age < 18");
        println!("  CREATE TABLE users (id INT, name TEXT, age INT)");
        println!("  DROP TABLE users");
        println!("\nPlugin Commands:");
        println!("  PLUGIN - List available plugins");
        println!("  PLUGIN <name> <args> - Execute a plugin");
    }
}

struct ReplHelper;

impl ReplHelper {
    fn new() -> Self {
        Self
    }
}

impl rustyline::Helper for ReplHelper {}

impl rustyline::hint::Hinter for ReplHelper {
    type Hint = String;
}

impl rustyline::validate::Validator for ReplHelper {}

impl rustyline::highlight::Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        false
    }
}

impl rustyline::completion::Completer for ReplHelper {
    type Candidate = String;
}