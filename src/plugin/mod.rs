#[derive(Debug)]
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

// ... rest of the implementation ... 