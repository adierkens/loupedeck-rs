use crate::{PluginDeclaration, PluginRegistrar, ScreenPluginFactory, ScreenPluginOptions};
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::io::Result;
use std::sync::Arc;
use std::{alloc::System, collections::HashMap, env, ffi::OsStr, io, sync::Mutex};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginIdentifier {
    pub plugin_id: String,
    pub plugin_ref: String,
}

#[derive(Debug, Clone)]
pub struct LocalLoadedPlugin {
    lib: Arc<Library>,
    pub plugin_id: String,
    pub screens: HashMap<String, ScreenPluginFactory>,
}

pub struct TempPluginRegistrar {
    screens: HashMap<String, ScreenPluginFactory>,
    lib: Arc<Library>,
}

impl TempPluginRegistrar {
    pub fn new(lib: Arc<Library>) -> TempPluginRegistrar {
        TempPluginRegistrar {
            screens: HashMap::default(),
            lib,
        }
    }

    pub fn to_local_plugin(self, plugin_id: String) -> LocalLoadedPlugin {
        LocalLoadedPlugin {
            lib: self.lib,
            plugin_id,
            screens: self.screens,
        }
    }
}

impl PluginRegistrar for TempPluginRegistrar {
    fn register_screen(
        &mut self,
        name: &str,
        options: ScreenPluginOptions,
        create: ScreenPluginFactory,
    ) -> Result<()> {
        self.screens.insert(name.to_string(), create);

        return Ok(());
    }
}

pub struct PluginRegistry {
    pub plugins: HashMap<String, LocalLoadedPlugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::default(),
        }
    }

    pub unsafe fn load_from_path<P: AsRef<OsStr>>(&mut self, path: P) -> Result<()> {
        let library = Arc::new(Library::new(path)?);
        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
            .read();

        if decl.rustc_version != crate::RUSTC_VERSION || decl.core_version != crate::CORE_VERSION {
            return Err(io::Error::new(io::ErrorKind::Other, "Version mismatch"));
        }

        let mut registrar = TempPluginRegistrar::new(Arc::clone(&library));
        (decl.register)(&mut registrar);

        let local_plugin = registrar.to_local_plugin(decl.plugin_id.to_string());

        println!(
            "Loaded plugin_id: {:?} with handlers: {:?}",
            local_plugin.plugin_id,
            local_plugin.screens.keys()
        );

        self.plugins
            .insert(decl.plugin_id.to_string(), local_plugin);

        Ok(())
    }
}
