use std::rc::Rc;

use hashbrown::HashMap;

use crate::compile::error::CompilationError;
use crate::parse::tree::{
    Assignment, AssignmentTarget, AssignmentType, CodeVisitor, DefaultCodeVisitor, Expr, ExprType,
    InputType, ResultCodeVisitor, Stmt, StmtType,
};
use crate::util::ptr_map::PtrMap;
use crate::util::ptr_set::PtrSet;

#[derive(Debug)]
struct VariableInfo {
    declaration: Rc<AssignmentTarget>,
    nesting_level: u64,
    is_written: bool,
    is_captured: bool,
}

enum CapturingContextType {
    ProcessLiteral(Rc<Expr>),
    Constructor(Rc<AssignmentTarget>),
}

struct CapturingContext {
    typ: CapturingContextType,
    nesting_level: u64,
}

pub struct CaptureAnalyzer {
    nesting_level: u64,
    variables: HashMap<String, VariableInfo>,
    capturing_contexts: Vec<CapturingContext>,
    assignment_to_declaration: PtrMap<AssignmentTarget, Rc<AssignmentTarget>>,
    pub process_literal_captures: PtrMap<Expr, PtrSet<AssignmentTarget>>,
    pub constructor_captures: PtrMap<AssignmentTarget, PtrSet<AssignmentTarget>>,
    pub capture_assignment_targets: PtrSet<AssignmentTarget>,
}

impl CaptureAnalyzer {
    pub fn new() -> Self {
        Self {
            nesting_level: 0,
            variables: HashMap::new(),
            capturing_contexts: Vec::new(),
            assignment_to_declaration: PtrMap::new(),
            process_literal_captures: PtrMap::new(),
            constructor_captures: PtrMap::new(),
            capture_assignment_targets: PtrSet::new(),
        }
    }

    fn add_reference_helper(
        &mut self,
        name: &str,
        assignment_target: Option<Rc<AssignmentTarget>>,
    ) -> Option<&mut VariableInfo> {
        let Some(var_info) = self.variables.get_mut(name) else {
            // Silently ignore unknown variables (which may be global variables or undefined).
            // Errors will be caught eventually.
            return None;
        };
        if let Some(assignment_target) = assignment_target {
            self.assignment_to_declaration
                .insert(assignment_target, Rc::clone(&var_info.declaration));
            var_info.is_written = true;
        }
        if self.nesting_level <= var_info.nesting_level {
            return Some(var_info);
        }
        var_info.is_captured = true;
        for ctx in self.capturing_contexts.iter().rev() {
            if ctx.nesting_level <= var_info.nesting_level {
                break;
            }
            match &ctx.typ {
                CapturingContextType::ProcessLiteral(expr) => {
                    let captures = self
                        .process_literal_captures
                        .get_or_insert_default(Rc::clone(expr));
                    captures.insert(Rc::clone(&var_info.declaration));
                }
                CapturingContextType::Constructor(cons) => {
                    let captures = self
                        .constructor_captures
                        .get_or_insert_default(Rc::clone(cons));
                    captures.insert(Rc::clone(&var_info.declaration));
                }
            }
        }
        Some(var_info)
    }

    fn add_reference(&mut self, name: &str, assignment_target: Option<Rc<AssignmentTarget>>) {
        if let Some(var_info) = self.add_reference_helper(name, assignment_target)
            && var_info.is_captured
            && var_info.is_written
        {
            let decl = Rc::clone(&var_info.declaration);
            self.capture_assignment_targets.insert(decl);
        }
    }
}

impl DefaultCodeVisitor for CaptureAnalyzer {
    type Error = CompilationError;

    fn visit_identifier(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::Identifier(ident), _) = &**expr else {
            unreachable!();
        };
        self.add_reference(ident, None);
        Ok(())
    }

    fn visit_in(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        if !matches!(**expr, Expr(ExprType::In(InputType::Fork), _)) {
            return Ok(());
        };
        self.nesting_level += 1;
        Ok(())
    }

    fn visit_process_literal(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::ProcessLiteral(code), _) = &**expr else {
            unreachable!();
        };
        let old_nesting_level = self.nesting_level;
        self.nesting_level += 1;
        self.capturing_contexts.push(CapturingContext {
            nesting_level: self.nesting_level,
            typ: CapturingContextType::ProcessLiteral(Rc::clone(expr)),
        });
        self.visit_many(code)?;
        self.nesting_level = old_nesting_level;
        self.capturing_contexts.pop().unwrap();
        Ok(())
    }

    fn visit_assignment_target(
        &mut self,
        target: &Rc<AssignmentTarget>,
    ) -> Result<(), Self::Error> {
        match target.typ {
            AssignmentType::Declaration => {
                self.variables.insert(
                    target.name.clone(),
                    VariableInfo {
                        declaration: Rc::clone(target),
                        nesting_level: self.nesting_level,
                        is_written: false,
                        is_captured: false,
                    },
                );
            }
            AssignmentType::Assignment | AssignmentType::AugmentedAssignment(_) => {
                self.add_reference(&target.name, Some(Rc::clone(target)));
            }
            AssignmentType::Constructor => {}
            AssignmentType::Discard => {}
        }
        Ok(())
    }

    fn visit_assignment_stmt(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Assignment(assignment), _) = &**stmt else {
            unreachable!();
        };
        if assignment
            .targets
            .iter()
            .any(|target| target.typ == AssignmentType::Constructor)
        {
            assert!(assignment.targets.len() == 1);
            assert!(assignment.values.len() == 1);
            let target = &assignment.targets[0];
            let value = &assignment.values[0];
            let old_nesting_level = self.nesting_level;
            self.nesting_level += 1;
            self.capturing_contexts.push(CapturingContext {
                nesting_level: self.nesting_level,
                typ: CapturingContextType::Constructor(Rc::clone(target)),
            });
            self.visit(value)?;
            self.visit(target)?;
            self.nesting_level = old_nesting_level;
            self.capturing_contexts.pop().unwrap();
            Ok(())
        } else {
            self.visit_many(&assignment.values)?;
            self.visit_many(&assignment.targets)
        }
    }

    fn visit_if(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::If(cond, then, else_), _) = &**stmt else {
            unreachable!();
        };
        self.visit(cond)?;
        let old_nesting_level = self.nesting_level;
        self.visit_many(then)?;
        let if_nesting_level = self.nesting_level;
        self.nesting_level = old_nesting_level;
        self.visit_many(else_)?;
        self.nesting_level = self.nesting_level.max(if_nesting_level);
        Ok(())
    }

    fn visit_switch(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Switch(expr, cases), _) = &**stmt else {
            unreachable!();
        };
        self.visit(expr)?;
        let mut final_nesting_level = self.nesting_level;
        for case in cases {
            if let Some(values) = &case.values {
                self.visit_many(values)?;
            }
            let old_nesting_level = self.nesting_level;
            self.visit_many(&case.body)?;
            final_nesting_level = final_nesting_level.max(self.nesting_level);
            self.nesting_level = old_nesting_level;
        }
        self.nesting_level = final_nesting_level;
        Ok(())
    }

    fn visit_global_assignment(&mut self, decl: &Rc<Assignment>) -> Result<(), Self::Error> {
        let old_nesting_level = self.nesting_level;
        self.visit_many(&decl.values)?;
        self.nesting_level = old_nesting_level;
        for (assignment, declaration) in self.assignment_to_declaration.drain() {
            if self.capture_assignment_targets.contains(declaration) {
                self.capture_assignment_targets.insert(assignment);
            }
        }
        Ok(())
    }
}
