use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use crate::builtin;
use crate::compile::environment::GlobalDefinition;
use crate::compile::error::CompilationError;
use crate::compile::process_family_builder::ProcessFamilyBuilder;
use crate::parse::location::Location;
use crate::parse::tree::{
    Assignment, AssignmentType, CodeVisitor, DefaultCodeVisitor, Expr, ExprType, GlobalDeclaration,
};
use crate::util::ptr_map::PtrMap;

pub struct ProcessFamilyCollector {
    // For each i, builder for the i'th process family.
    pub process_families: Vec<Rc<RefCell<ProcessFamilyBuilder>>>,
    // For each process literal, the index of the family it creates.
    pub process_literal_map: PtrMap<Expr, u32>,
    // For each lazily-initialized variable, the index of its initializer process family.
    pub lazy_initializer_map: PtrMap<Assignment, u32>,
    // For each constructor, the index of its generating process family.
    pub constructor_map: PtrMap<Assignment, u32>,
    // For each global variable or constructor, the index of the global variable storing
    // either its value or the generating process.
    pub global_var_map: PtrMap<Assignment, u32>,
    // All the names in the global scope.
    pub global_definitions: Rc<HashMap<String, GlobalDefinition>>,
}

impl ProcessFamilyCollector {
    pub fn new() -> Self {
        Self {
            process_families: Vec::new(),
            process_literal_map: PtrMap::new(),
            lazy_initializer_map: PtrMap::new(),
            constructor_map: PtrMap::new(),
            global_var_map: PtrMap::new(),
            global_definitions: Rc::new(HashMap::new()),
        }
    }

    fn new_family(&mut self, location: &Location) -> Result<u32, CompilationError> {
        let index = self.process_families.len();
        let Ok(index) = u32::try_from(index) else {
            return CompilationError::err(
                "too many process types (current limit is 4294967295)",
                location,
            );
        };
        self.process_families
            .push(Rc::new(RefCell::new(ProcessFamilyBuilder::new())));
        Ok(index)
    }

    fn new_global(&mut self, decl: Rc<Assignment>) -> Result<u32, CompilationError> {
        let index = self.global_var_map.len();
        let Ok(index) = u32::try_from(index) else {
            return CompilationError::err(
                "too many global variables (current limit is 4294967295)",
                &decl.location,
            );
        };
        self.global_var_map.insert(decl, index);
        Ok(index)
    }

    fn add_global_definition(&mut self, decl: &GlobalDeclaration) -> Result<(), CompilationError> {
        let global_definitions = Rc::get_mut(&mut self.global_definitions)
            .expect("global_definitions Rc should have a reference-count of 1");
        match decl {
            GlobalDeclaration::Assignment(decl) => {
                assert_eq!(decl.targets.len(), 1);
                let target = &decl.targets[0];
                let global_definition = match target.typ {
                    AssignmentType::Constructor => GlobalDefinition::Constructor {
                        generator_family: self.constructor_map[decl],
                        result_expr: Rc::clone(&decl.values[0]),
                    },
                    AssignmentType::Declaration => GlobalDefinition::Variable {
                        global_index: self.global_var_map[decl],
                    },
                    _ => panic!("expected a constructor or a variable"),
                };
                match global_definitions.entry(target.name.clone()) {
                    Entry::Occupied(_) => {
                        return CompilationError::err(
                            format!("duplicate definition of global `{}`", target.name.clone()),
                            &decl.location,
                        );
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(global_definition);
                    }
                }
            }
            GlobalDeclaration::Import(decl) => {
                for (i, (old_name, new_name)) in decl.items.iter().enumerate() {
                    let new_name = new_name.as_ref().unwrap_or(old_name);
                    match global_definitions.entry(new_name.clone()) {
                        Entry::Occupied(_) => {
                            return CompilationError::err(
                                format!("duplicate definition of global `{}`", new_name),
                                &decl.location,
                            );
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(GlobalDefinition::Import {
                                decl: Rc::clone(decl),
                                index: i,
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl DefaultCodeVisitor for ProcessFamilyCollector {
    type Error = CompilationError;

    fn visit_process_literal(&mut self, expr: &std::rc::Rc<Expr>) -> Result<(), Self::Error> {
        let index = self.new_family(&expr.1)?;
        self.process_literal_map.insert(Rc::clone(expr), index);
        let Expr(ExprType::ProcessLiteral(stmts), _) = &**expr else {
            panic!("visitor method got wrong variant");
        };
        for stmt in stmts {
            self.visit(stmt)?
        }
        Ok(())
    }

    fn visit_global_assignment(&mut self, decl: &Rc<Assignment>) -> Result<(), Self::Error> {
        // If multiple targets are ever allowed, make sure to handle duplicate definitions.
        if decl.targets.len() != 1 {
            return CompilationError::err(
                "multiple items defined in a global declaration (only one is allowed)",
                &decl.location,
            );
        }
        let target = &decl.targets[0];
        if decl.values.len() > 1 {
            return CompilationError::err(
                "wrong number of values assigned in a global declaration",
                &decl.values[1].1,
            );
        }
        match target.typ {
            AssignmentType::Assignment | AssignmentType::AugmentedAssignment(_) => {
                return CompilationError::err(
                    "assignment is not allowed outside of a process",
                    &decl.location,
                );
            }
            AssignmentType::Declaration => {
                let family_index = self.new_family(&decl.location)?;
                self.lazy_initializer_map
                    .insert(Rc::clone(decl), family_index);
                self.new_global(Rc::clone(decl))?;
            }
            AssignmentType::Constructor => {
                let index = self.new_family(&decl.location)?;
                self.constructor_map.insert(Rc::clone(decl), index);
            }
        }
        let value = &decl.values[0];
        self.visit(value)
    }

    fn visit_code_file(
        &mut self,
        file: &Rc<crate::parse::tree::CodeFile>,
    ) -> Result<(), Self::Error> {
        for decl in &file.declarations {
            self.visit(decl)?;
        }
        for decl in &file.declarations {
            self.add_global_definition(decl)?;
        }
        let global_definitions = Rc::get_mut(&mut self.global_definitions)
            .expect("global_definitions Rc should have a reference-count of 1");
        for (&name, &index) in &builtin::ROOT_MODULE.constructor_definitions {
            if !global_definitions.contains_key(name) {
                global_definitions.insert(
                    name.to_string(),
                    GlobalDefinition::BuiltinConstructor { index },
                );
            }
        }
        for (&name, &index) in &builtin::ROOT_MODULE.constant_definitions {
            if !global_definitions.contains_key(name) {
                global_definitions.insert(
                    name.to_string(),
                    GlobalDefinition::BuiltinConstant { index },
                );
            }
        }
        for family in &self.process_families {
            family
                .borrow_mut()
                .environment_mut()
                .init_global_names(Rc::clone(&self.global_definitions));
        }
        Ok(())
    }
}
