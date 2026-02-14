use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::builtin::BuiltinModule;
use crate::compile::environment::GlobalDefinition;

#[derive(Debug)]
pub struct Module {
    definitions: HashMap<String, GlobalDefinition>,
    sub_modules: HashMap<String, Self>,
}

#[derive(Debug, thiserror::Error)]
#[error("trying to create duplicate item in module")]
pub struct DuplicateModuleDefinition;

impl Module {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            sub_modules: HashMap::new(),
        }
    }

    pub fn add_definition(
        &mut self,
        name: String,
        definition: GlobalDefinition,
    ) -> Result<(), DuplicateModuleDefinition> {
        if self.definitions.insert(name, definition).is_some() {
            return Err(DuplicateModuleDefinition);
        }
        Ok(())
    }

    pub fn add_new_sub_module(
        &mut self,
        name: String,
    ) -> Result<&mut Self, DuplicateModuleDefinition> {
        let Entry::Vacant(entry) = self.sub_modules.entry(name) else {
            return Err(DuplicateModuleDefinition);
        };
        Ok(entry.insert(Self::new()))
    }

    pub fn get_definition(&self, name: &str) -> Option<&GlobalDefinition> {
        self.definitions.get(name)
    }

    pub fn get_sub_module(&self, name: &str) -> Option<&Self> {
        self.sub_modules.get(name)
    }

    pub fn init_from_builtin(&mut self, builtin: &BuiltinModule) {
        for (name, &index) in &builtin.constructor_definitions {
            self.add_definition(
                name.to_string(),
                GlobalDefinition::BuiltinConstructor { index },
            )
            .unwrap();
        }
        for (name, &index) in &builtin.constant_definitions {
            self.add_definition(
                name.to_string(),
                GlobalDefinition::BuiltinConstant { index },
            )
            .unwrap();
        }
        for (name, builtin_sub) in &builtin.sub_modules {
            let sub = self.add_new_sub_module(name.to_string()).unwrap();
            sub.init_from_builtin(builtin_sub);
        }
    }
}
