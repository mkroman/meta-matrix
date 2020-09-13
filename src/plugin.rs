use async_trait::async_trait;
use log::{debug, error};

use crate::Error;

#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

#[async_trait]
pub trait Plugin {
    /// Returns the full Rust module path of the plugin
    fn module_path(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Returns the type name fo the plugin
    fn name(&self) -> &'static str {
        self.module_path().split("::").last().unwrap()
    }

    fn new() -> Result<Self, Error>
    where
        Self: Sized;

    async fn handle_room_message(&self) -> Result<(), Error>;
}

impl PluginRegistry {
    /// Constructs and returns a new plugin registry
    pub fn new() -> PluginRegistry {
        PluginRegistry { plugins: vec![] }
    }

    /// Instantiates the given trait and adds it to the registry
    pub fn register<P: Plugin + 'static>(&mut self) -> Result<(), Error> {
        debug!("Registering plugin {}", std::any::type_name::<P>());

        let plugin = P::new();

        match plugin {
            Ok(plugin) => {
                debug!("Registered plugin {}", plugin.name());

                self.plugins.push(Box::new(plugin));
            }
            Err(err) => error!("Failed to register plugin: {}", err),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestPlugin {}

    #[async_trait]
    impl Plugin for TestPlugin {
        fn new() -> Result<TestPlugin, Error> {
            Ok(TestPlugin {})
        }
        async fn handle_room_message(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[test]
    fn test_register_plugin() {
        let mut registry = PluginRegistry::new();

        registry.register::<TestPlugin>().unwrap();
    }
}
