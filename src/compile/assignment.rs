use std::cell::OnceCell;
use std::rc::Rc;

use either::Either::{Left, Right};

use crate::compile::Compiler;
use crate::compile::environment::{GlobalDefinition, LocalDefinition};
use crate::compile::error::CompilationError;
use crate::compile::process_family_builder::ProcessFamilyBuilder;
use crate::compile::register_allocator::{ChosenRegister, RegisterChoice};
use crate::parse::location::Location;
use crate::parse::tree::{Assignment, AssignmentTarget, AssignmentType, Expr, ExprType, InputType};
use crate::util::ptr_map::PtrMap;
use crate::vm::macros::{code, instruction};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentContext {
    Normal,
    AssignElse,
    If,
    While,
}

#[derive(Default)]
pub struct AssignmentCompilationResult {
    pub continue_target: usize,
    pub skip_jumps: Vec<usize>,
    pub temporary_indices: Vec<u16>,
}

#[derive(Clone)]
enum AssignedValueType {
    In,
    Receive {
        expr: Rc<Expr>,
        source_reg: Rc<OnceCell<ChosenRegister>>,
        is_last_source_use: bool,
    },
    Value(Rc<Expr>),
}

#[derive(Clone)]
struct AssignedValue {
    typ: AssignedValueType,
    is_last: bool,
    location: Location,
}

struct CopiedRegister {
    original: u16,
    copy: u16,
}

struct AssignmentCompiler<'compiler, 'builder> {
    compiler: &'compiler mut Compiler,
    builder: &'builder mut ProcessFamilyBuilder,
    assignment: Rc<Assignment>,
    context: AssignmentContext,
    result: AssignmentCompilationResult,
    fail_jumps: Vec<usize>,
    selected_registers: PtrMap<AssignmentTarget, u16>,
    copied_registers: PtrMap<AssignmentTarget, CopiedRegister>,
}

impl<'compiler, 'builder> AssignmentCompiler<'compiler, 'builder> {
    fn is_captured(&self, target: &Rc<AssignmentTarget>) -> bool {
        self.compiler
            .captures
            .capture_assignment_targets
            .contains(Rc::clone(target))
    }

    fn validate_no_duplicate_targets(&self) -> Result<(), CompilationError> {
        let targets = &self.assignment.targets;
        match &targets[..] {
            [] => panic!("zero assignment targets"),
            [_] => return Ok(()),
            [target1, target2] => {
                if target1.typ == AssignmentType::Discard
                    || target2.typ == AssignmentType::Discard
                    || target1.name != target2.name
                {
                    return Ok(());
                }
            }
            _ => {
                let mut names = targets
                    .iter()
                    .filter(|target| target.typ != AssignmentType::Discard)
                    .map(|target| &target.name[..])
                    .collect::<Vec<&str>>();
                names.sort();
                if !names
                    .array_windows::<2>()
                    .any(|[name1, name2]| name1 == name2)
                {
                    return Ok(());
                }
            }
        }
        CompilationError::err("duplicate targets in assignment", &self.assignment.location)
    }

    fn get_assignment_index(&self, target: &Rc<AssignmentTarget>) -> Result<u16, CompilationError> {
        let AssignmentTarget { name, location, .. } = &**target;
        match self.builder.environment().get_definition(name) {
            None => CompilationError::err(
                format!(
                    "name `{}` not found (use `:=` to define a new variable)",
                    name
                ),
                location,
            ),
            Some(Left(
                GlobalDefinition::Variable { .. } | GlobalDefinition::BuiltinConstant { .. },
            )) => CompilationError::err(
                format!("cannot assign to global variable `{}`", name),
                location,
            ),
            Some(
                Left(GlobalDefinition::Constructor { .. })
                | Left(GlobalDefinition::BuiltinConstructor { .. })
                | Right(LocalDefinition::Constructor { .. }),
            ) => {
                CompilationError::err(format!("cannot assign to constructor `{}`", name), location)
            }
            Some(Left(GlobalDefinition::Import { .. })) => CompilationError::err(
                format!("cannot assign to externally defined `{}`", name),
                location,
            ),
            Some(Right(&LocalDefinition::Variable { index })) => Ok(index),
            Some(Right(&LocalDefinition::CapturedVariable { index })) => Ok(index),
        }
    }

    fn target_to_register_choice(
        &self,
        target: &Rc<AssignmentTarget>,
    ) -> Result<RegisterChoice, CompilationError> {
        if let Some(&reg) = self.selected_registers.get(Rc::clone(target)) {
            return Ok(RegisterChoice::Existing(reg));
        }
        if let Some(copy) = self.copied_registers.get(Rc::clone(target)) {
            return Ok(RegisterChoice::Existing(copy.copy));
        }
        match target.typ {
            AssignmentType::Declaration => Ok(RegisterChoice::AllocNew),
            AssignmentType::Assignment => {
                Ok(RegisterChoice::Existing(self.get_assignment_index(target)?))
            }
            AssignmentType::Discard => Ok(RegisterChoice::Any),
            AssignmentType::AugmentedAssignment(_) => CompilationError::err(
                "an augmented assignment is not allowed in this context",
                &target.location,
            ),
            AssignmentType::Constructor => CompilationError::err(
                "a constructor is not allowed in this context",
                &target.location,
            ),
        }
    }

    fn validate_augmented_assignment(
        &self,
    ) -> Result<(Rc<AssignmentTarget>, Rc<Expr>), CompilationError> {
        if self.assignment.targets.len() != 1 {
            return CompilationError::err(
                "cannot have more than one target for augmented assignment",
                &self.assignment.targets[1].location,
            );
        }
        if self.assignment.values.len() != 1 {
            return CompilationError::err(
                "cannot have more than one value in an augmented assignment",
                &self.assignment.values[1].1,
            );
        }
        match self.context {
            AssignmentContext::Normal => {}
            AssignmentContext::AssignElse => {
                return CompilationError::err(
                    "augmented assignment cannot have an `else` clause",
                    &self.assignment.location,
                );
            }
            AssignmentContext::If | AssignmentContext::While => {
                return CompilationError::err(
                    "augmented assignment may not appear in a condition",
                    &self.assignment.location,
                );
            }
        }
        Ok((
            Rc::clone(&self.assignment.targets[0]),
            Rc::clone(&self.assignment.values[0]),
        ))
    }

    fn compile_augmented_assignment(
        &mut self,
        target: &Rc<AssignmentTarget>,
        value: &Rc<Expr>,
    ) -> Result<(), CompilationError> {
        let AssignmentType::AugmentedAssignment(op) = target.typ else {
            panic!("expected an augmented assignment");
        };
        let var_reg = self.get_assignment_index(target)?;
        let rhs_reg = self
            .compiler
            .compile_expr(value, RegisterChoice::Any, self.builder)?;
        if self.is_captured(target) {
            let tmp_reg = self
                .builder
                .register_allocator_mut()
                .alloc_temporary(&target.location)?;
            rhs_reg.dealloc(self.builder.register_allocator_mut());
            self.builder.register_allocator_mut().dealloc(tmp_reg);
            self.builder.add_code(code! {
                LOAD_CAPTURE tmp_reg, var_reg, 0;
            });
            self.compiler
                .compile_binary_op(op, tmp_reg, rhs_reg.index, tmp_reg, self.builder);
            self.builder.add_code(code! {
                STORE_CAPTURE var_reg, tmp_reg, 0;
            });
        } else {
            rhs_reg.dealloc(self.builder.register_allocator_mut());
            self.compiler
                .compile_binary_op(op, var_reg, rhs_reg.index, var_reg, self.builder);
        }
        Ok(())
    }

    fn validate_local_constructor(
        &self,
    ) -> Result<(Rc<AssignmentTarget>, Rc<Expr>), CompilationError> {
        if self.assignment.targets.len() != 1 {
            return CompilationError::err(
                "cannot declare multiple constructor in one line",
                &self.assignment.targets[1].location,
            );
        }
        if self.assignment.values.len() != 1 {
            return CompilationError::err(
                "constructor may not have multiple values",
                &self.assignment.values[1].1,
            );
        }
        match self.context {
            AssignmentContext::Normal => {}
            AssignmentContext::AssignElse => {
                return CompilationError::err(
                    "a constructor declaration cannot have an `else` clause",
                    &self.assignment.location,
                );
            }
            AssignmentContext::If | AssignmentContext::While => {
                return CompilationError::err(
                    "a constructor declaration may not appear in a condition",
                    &self.assignment.location,
                );
            }
        }
        Ok((
            Rc::clone(&self.assignment.targets[0]),
            Rc::clone(&self.assignment.values[0]),
        ))
    }

    fn compile_local_constructor(
        &mut self,
        target: &AssignmentTarget,
        value: &Rc<Expr>,
    ) -> Result<(), CompilationError> {
        assert_eq!(target.typ, AssignmentType::Constructor);
        let _ = value;
        todo!("local constructors are unimplemented")
    }

    fn get_source_reg(
        &mut self,
        value: &AssignedValue,
        reg_choice: RegisterChoice,
    ) -> Result<ChosenRegister, CompilationError> {
        let &AssignedValueType::Receive {
            ref expr,
            ref source_reg,
            is_last_source_use,
        } = &value.typ
        else {
            panic!("expected a receive expression");
        };
        if let Some(&source_reg) = source_reg.get() {
            Ok(source_reg)
        } else {
            let source_reg_choice =
                if is_last_source_use && self.context == AssignmentContext::Normal {
                    reg_choice
                } else {
                    // TODO: consider relaxing this.
                    RegisterChoice::AllocNew
                };
            assert!(
                source_reg
                    .set(
                        self.compiler
                            .compile_expr(expr, source_reg_choice, self.builder)?
                    )
                    .is_ok()
            );
            Ok(*source_reg.get().unwrap())
        }
    }

    fn compile_assigned_value(
        &mut self,
        value: &AssignedValue,
        reg_choice: RegisterChoice,
    ) -> Result<ChosenRegister, CompilationError> {
        match &value.typ {
            AssignedValueType::In => {
                let reg =
                    reg_choice.or_alloc(self.builder.register_allocator_mut(), &value.location)?;
                debug_assert!(reg_choice == RegisterChoice::Any || reg.index != 0);
                match self.context {
                    AssignmentContext::Normal => {
                        self.builder.add_code(code! {
                            IN reg.index, 0, 0;
                        });
                    }
                    AssignmentContext::AssignElse
                    | AssignmentContext::If
                    | AssignmentContext::While => {
                        self.builder.add_code(code! {
                            OPT_IN reg.index, 0, 0;
                        });
                        if !value.is_last {
                            self.fail_jumps.push(self.builder.add_jump(instruction! {
                                JUMP_UNLESS 0, 0;
                            }));
                        }
                    }
                }
                Ok(reg)
            }
            &AssignedValueType::Receive {
                is_last_source_use, ..
            } => {
                let source_reg = self.get_source_reg(value, reg_choice)?;
                if is_last_source_use {
                    source_reg.dealloc(self.builder.register_allocator_mut());
                }
                let reg =
                    reg_choice.or_alloc(self.builder.register_allocator_mut(), &value.location)?;
                match self.context {
                    AssignmentContext::Normal => {
                        self.builder.add_code(code! {
                            NO_IN source_reg.index, 0, 0;
                            RECEIVE reg.index, source_reg.index, 0;
                        });
                    }
                    AssignmentContext::AssignElse
                    | AssignmentContext::If
                    | AssignmentContext::While => {
                        self.builder.add_code(code! {
                            NO_IN source_reg.index, 0, 0;
                            TRY_RECEIVE reg.index, 0, source_reg.index;
                        });
                        if !value.is_last {
                            self.fail_jumps.push(self.builder.add_jump(instruction! {
                                JUMP_UNLESS 0, 0;
                            }));
                        }
                    }
                }
                Ok(reg)
            }
            AssignedValueType::Value(expr) => {
                if self.context != AssignmentContext::Normal {
                    return CompilationError::err("invalid value for optional assignment", &expr.1);
                }
                self.compiler.compile_expr(expr, reg_choice, self.builder)
            }
        }
    }

    fn expr_to_assigned_value(expr: &Rc<Expr>) -> AssignedValue {
        AssignedValue {
            typ: match &expr.0 {
                ExprType::In(InputType::Normal) => AssignedValueType::In,
                ExprType::Receive(process) => AssignedValueType::Receive {
                    expr: Rc::clone(process),
                    source_reg: Rc::new(OnceCell::new()),
                    is_last_source_use: false,
                },
                _ => AssignedValueType::Value(Rc::clone(expr)),
            },
            is_last: false,
            location: expr.1.clone(),
        }
    }

    fn get_assigned_values(&self) -> Result<Vec<AssignedValue>, CompilationError> {
        let mut result = Vec::with_capacity(self.assignment.targets.len());
        if self.assignment.targets.len() == self.assignment.values.len() {
            for value in self.assignment.values.iter() {
                let mut value = Self::expr_to_assigned_value(value);
                if self.context != AssignmentContext::While
                    && let AssignedValueType::Receive {
                        is_last_source_use, ..
                    } = &mut value.typ
                {
                    *is_last_source_use = true;
                }
                result.push(value);
            }
        } else if self.assignment.values.len() == 1 {
            let value = Self::expr_to_assigned_value(&self.assignment.values[0]);
            if matches!(value.typ, AssignedValueType::Value(_)) {
                return CompilationError::err(
                    "invalid expression for assignment to multiple variables",
                    &value.location,
                );
            }
            for _ in 0..self.assignment.targets.len() {
                result.push(value.clone());
            }
            if self.context != AssignmentContext::While
                && let AssignedValueType::Receive {
                    is_last_source_use, ..
                } = &mut result.last_mut().unwrap().typ
            {
                *is_last_source_use = true;
            }
        } else {
            return CompilationError::err(
                format!(
                    "wrong number of assigned values ({} value(s) assigned to {} variable(s))",
                    self.assignment.values.len(),
                    self.assignment.targets.len()
                ),
                &self.assignment.location,
            );
        }
        assert_eq!(result.len(), self.assignment.targets.len());
        result.last_mut().unwrap().is_last = true;
        Ok(result)
    }

    fn pre_assignment(
        &mut self,
        assigned_values: &[AssignedValue],
    ) -> Result<(), CompilationError> {
        if self.context == AssignmentContext::Normal {
            return Ok(());
        }
        if self.context == AssignmentContext::While {
            for value in assigned_values {
                let AssignedValueType::Receive {
                    expr, source_reg, ..
                } = &value.typ
                else {
                    continue;
                };
                if source_reg.get().is_some() {
                    continue;
                }
                let tmp_reg =
                    self.compiler
                        .compile_expr(expr, RegisterChoice::AllocNew, self.builder)?;
                self.result.temporary_indices.push(tmp_reg.index);
                if source_reg.set(tmp_reg).is_err() {
                    panic!("source_reg should not be initialized");
                }
            }
        }
        self.result.continue_target = self.builder.next_instruction_index();
        for (i, target) in self.assignment.targets.iter().enumerate() {
            match self.target_to_register_choice(target)? {
                RegisterChoice::AllocNew if self.context == AssignmentContext::AssignElse => {
                    let reg = self
                        .builder
                        .register_allocator_mut()
                        .alloc(&target.location)?;
                    self.selected_registers.insert_new(Rc::clone(target), reg);
                    self.builder.add_code(code! {
                        CONST reg, self.compiler.const_undefined_index;
                    });
                }
                RegisterChoice::Existing(reg) => {
                    if i == self.assignment.targets.len() - 1 {
                        continue;
                    }
                    let new_reg = self
                        .builder
                        .register_allocator_mut()
                        .alloc(&target.location)?;
                    self.copied_registers.insert_new(
                        Rc::clone(target),
                        CopiedRegister {
                            original: reg,
                            copy: new_reg,
                        },
                    );
                    if self.is_captured(target) {
                        self.builder.add_code(code! {
                            LOAD_CAPTURE new_reg, reg, 0;
                        });
                    } else {
                        self.compiler.compile_move(new_reg, reg, self.builder);
                    }
                }
                RegisterChoice::Any if target.typ != AssignmentType::Discard => {
                    debug_assert!(self.is_captured(target));
                    if self.context == AssignmentContext::AssignElse {
                        debug_assert!(target.typ != AssignmentType::Declaration);
                    }
                }
                RegisterChoice::AllocNew | RegisterChoice::Any => {}
            }
        }
        Ok(())
    }

    fn compile_assignment(&mut self, values: &[AssignedValue]) -> Result<(), CompilationError> {
        let targets = &Rc::clone(&self.assignment).targets;
        assert_eq!(targets.len(), values.len());
        for (target, value) in targets.iter().zip(values) {
            let reg_choice = self.target_to_register_choice(target)?;
            if target.typ == AssignmentType::Discard {
                assert_eq!(reg_choice, RegisterChoice::Any);
                let new_reg = self.compile_assigned_value(value, reg_choice)?;
                new_reg.dealloc(self.builder.register_allocator_mut());
            } else if self.context == AssignmentContext::Normal && !value.is_last {
                let new_reg = self.compile_assigned_value(value, RegisterChoice::AllocNew)?;
                match reg_choice {
                    RegisterChoice::Existing(reg) => {
                        self.copied_registers.insert_new(
                            Rc::clone(target),
                            CopiedRegister {
                                original: reg,
                                copy: new_reg.index,
                            },
                        );
                    }
                    RegisterChoice::AllocNew => {
                        self.selected_registers
                            .insert_new(Rc::clone(target), new_reg.index);
                    }
                    RegisterChoice::Any => unreachable!(),
                }
            } else if self.is_captured(target)
                && self.copied_registers.get(Rc::clone(target)).is_none()
                && self.selected_registers.get(Rc::clone(target)).is_none()
            {
                // TODO: consider changing this to RegisterChoice::Any when possible.
                let new_reg = self.compile_assigned_value(value, RegisterChoice::AllocNew)?;
                if let RegisterChoice::Existing(capture_reg) = reg_choice {
                    if value.is_last {
                        match self.context {
                            AssignmentContext::Normal => {}
                            AssignmentContext::If
                            | AssignmentContext::While
                            | AssignmentContext::AssignElse => {
                                self.builder.add_code(code! {
                                    JUMP_UNLESS 0, 1;
                                });
                            }
                        }
                    }
                    self.builder.add_code(code! {
                        STORE_CAPTURE capture_reg, new_reg.index, 0;
                    });
                    new_reg.dealloc(self.builder.register_allocator_mut());
                } else {
                    self.selected_registers
                        .insert_new(Rc::clone(target), new_reg.index);
                }
            } else {
                let new_reg = self.compile_assigned_value(value, reg_choice)?;
                if new_reg.is_owned {
                    self.selected_registers
                        .insert_new(Rc::clone(target), new_reg.index);
                }
            }
        }
        Ok(())
    }

    fn post_assignment(&mut self) -> Result<(), CompilationError> {
        for jump in self.fail_jumps.drain(..) {
            self.builder
                .link_jump_here(jump, &self.assignment.location)?;
        }
        for target in &self.assignment.targets {
            let Some(&reg) = self.selected_registers.get(Rc::clone(target)) else {
                continue;
            };
            if self.is_captured(target) {
                match target.typ {
                    AssignmentType::Assignment => {}
                    AssignmentType::Declaration => {
                        self.builder.add_code(code! {
                            CAPTURE reg, reg, 0;
                        });
                        self.builder.environment_mut().add_local(
                            target.name.clone(),
                            LocalDefinition::CapturedVariable { index: reg },
                            &target.location,
                        )?;
                    }
                    _ => panic!("unexpected target type"),
                }
            } else {
                self.builder.environment_mut().add_local(
                    target.name.clone(),
                    LocalDefinition::Variable { index: reg },
                    &target.location,
                )?;
            }
        }
        for target in &self.assignment.targets {
            if let Some(copy) = self.copied_registers.get(Rc::clone(target)) {
                if self.is_captured(target) {
                    self.builder.add_code(code! {
                        STORE_CAPTURE copy.original, copy.copy, 0;
                    });
                } else {
                    self.compiler
                        .compile_move(copy.original, copy.copy, self.builder);
                }
                self.builder.register_allocator_mut().dealloc(copy.copy);
            }
        }
        match self.context {
            AssignmentContext::AssignElse => {
                self.result
                    .skip_jumps
                    .push(self.builder.add_jump(instruction! {
                        JUMP_IF 0, 0;
                    }));
            }
            AssignmentContext::If | AssignmentContext::While => {
                self.result
                    .skip_jumps
                    .push(self.builder.add_jump(instruction! {
                        JUMP_UNLESS 0, 0;
                    }))
            }
            AssignmentContext::Normal => {}
        }
        Ok(())
    }

    fn compile(&mut self) -> Result<(), CompilationError> {
        assert!(!self.assignment.targets.is_empty());
        assert!(!self.assignment.values.is_empty());
        self.validate_no_duplicate_targets()?;
        if matches!(
            self.assignment.targets[0].typ,
            AssignmentType::AugmentedAssignment(_)
        ) {
            let (target, value) = self.validate_augmented_assignment()?;
            self.compile_augmented_assignment(&target, &value)?;
            return Ok(());
        }
        if matches!(self.assignment.targets[0].typ, AssignmentType::Constructor) {
            let (target, value) = self.validate_local_constructor()?;
            self.compile_local_constructor(&target, &value)?;
            return Ok(());
        }
        let assigned = self.get_assigned_values()?;
        self.pre_assignment(&assigned)?;
        self.compile_assignment(&assigned)?;
        self.post_assignment()?;
        Ok(())
    }

    fn new(
        compiler: &'compiler mut Compiler,
        builder: &'builder mut ProcessFamilyBuilder,
        assignment: Rc<Assignment>,
        context: AssignmentContext,
    ) -> Self {
        Self {
            compiler,
            builder,
            assignment,
            context,
            result: AssignmentCompilationResult::default(),
            fail_jumps: Vec::new(),
            selected_registers: PtrMap::new(),
            copied_registers: PtrMap::new(),
        }
    }
}

pub fn compile(
    compiler: &mut Compiler,
    builder: &mut ProcessFamilyBuilder,
    assignment: Rc<Assignment>,
    context: AssignmentContext,
) -> Result<AssignmentCompilationResult, CompilationError> {
    let mut assignment_compiler = AssignmentCompiler::new(compiler, builder, assignment, context);
    assignment_compiler.compile()?;
    Ok(assignment_compiler.result)
}
