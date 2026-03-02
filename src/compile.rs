mod assignment;
mod environment;
mod error;
mod module;
mod process_family_builder;
mod process_family_collector;
mod register_allocator;

use std::rc::Rc;

use assert_matches::assert_matches;
use either::Either::{Left, Right};

use crate::builtin;
use crate::compile::assignment::{AssignmentCompilationResult, AssignmentContext};
use crate::compile::environment::{GlobalDefinition, LocalDefinition};
use crate::compile::error::CompilationError;
use crate::compile::module::Module;
use crate::compile::process_family_builder::ProcessFamilyBuilder;
use crate::compile::process_family_collector::ProcessFamilyCollector;
use crate::compile::register_allocator::{ChosenRegister, RegisterChoice};
use crate::parse::location::Location;
use crate::parse::tree::{
    Argument, ArgumentType, Assignment, AssignmentType, BinaryOperator, CodeFile, CodeVisitor,
    Condition, ConstantLiteral, Expr, ExprType, GlobalDeclaration, JumpType, RaiseType, Stmt,
    StmtType, UnaryOperator,
};
use crate::vm::builder::VmBuilder;
use crate::vm::macros::{code, instruction};
use crate::vm::{Value, Vm};

pub struct Compiler {
    code: Rc<CodeFile>,
    processes: ProcessFamilyCollector,
    // The root module contains all the modules in the program.
    root_module: Module,
    const_undefined_index: u32,
    output: VmBuilder,
}

impl Compiler {
    pub fn new(code: Rc<CodeFile>) -> Result<Self, CompilationError> {
        let mut this = Self {
            code: Rc::clone(&code),
            processes: ProcessFamilyCollector::new(),
            root_module: Module::new(),
            const_undefined_index: 0,
            output: VmBuilder::new(),
        };
        this.root_module.init_from_builtin(&builtin::ROOT_MODULE);
        for (i, constant) in builtin::CONSTANTS.iter().enumerate() {
            let index = constant(&mut this.output);
            assert_eq!(i as u64, index as u64);
        }
        this.const_undefined_index = this
            .output
            .constant(Value::UNDEFINED)
            .expect("failed to create constant Undefined");
        this.processes.visit(&code)?;
        Ok(this)
    }

    fn compile_move(&mut self, dst: u16, src: u16, builder: &mut ProcessFamilyBuilder) {
        if src != dst {
            builder.add_code(code! {
                MOVE dst, src, 0;
            });
        }
    }

    // lhs, rhs, and output are registers.
    // They are not necessary allocated,
    // but lhs and rhs contain valid values before the operation.
    fn compile_binary_op(
        &mut self,
        op: BinaryOperator,
        lhs: u16,
        rhs: u16,
        output: u16,
        builder: &mut ProcessFamilyBuilder,
    ) {
        use BinaryOperator::*;
        let code = match op {
            Power => code! {
                POWER output, lhs, rhs;
            },
            Multiply => code! {
                MULTIPLY output, lhs, rhs;
            },
            Divide => code! {
                DIVIDE output, lhs, rhs;
            },
            Remainder => code! {
                REMAINDER output, lhs, rhs;
            },
            Add => code! {
                ADD output, lhs, rhs;
            },
            Subtract => code! {
                SUBTRACT output, lhs, rhs;
            },
            Equals => code! {
                EQUALS output, lhs, rhs;
            },
            NotEquals => code! {
                NOT_EQUALS output, lhs, rhs;
            },
            Less => code! {
                LESS output, lhs, rhs;
            },
            Greater => code! {
                LESS output, rhs, lhs; // Reversed
            },
            LessOrEqual => code! {
                LESS_OR_EQUAL output, lhs, rhs;
            },
            GreaterOrEqual => code! {
                LESS_OR_EQUAL output, rhs, lhs; // Reversed
            },
            Xor => code! {
                XOR output, lhs, rhs;
            },
            And | Or => panic!("compile_binary_op does not support `and` and `or`"),
        };
        builder.add_code(code);
    }

    fn compile_short_circuit_op(
        &mut self,
        expr: &Rc<Expr>,
        register_choice: RegisterChoice,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<ChosenRegister, CompilationError> {
        let ExprType::Binary(op, lhs, rhs) = &expr.0 else {
            panic!("expected an operator expression");
        };
        let reg = self.compile_expr(lhs, RegisterChoice::AllocNew, builder)?;
        let jump = builder.add_jump(match op {
            BinaryOperator::And => instruction! {
                JUMP_UNLESS reg.index, 0;
            },
            BinaryOperator::Or => instruction! {
                JUMP_IF reg.index, 0;
            },
            _ => panic!("expected a short-circuit operator"),
        });
        match register_choice {
            RegisterChoice::AllocNew | RegisterChoice::Any => {
                let rhs_reg = self.compile_expr(rhs, RegisterChoice::Any, builder)?;
                rhs_reg.dealloc(builder.register_allocator_mut());
                builder.add_code(code! {
                    MOVE_BOOL reg.index, rhs_reg.index, 0;
                });
                builder.link_jump_here(jump, &expr.1)?;
                Ok(reg)
            }
            RegisterChoice::Existing(out_reg) => {
                self.compile_expr(rhs, RegisterChoice::Existing(reg.index), builder)?;
                builder.link_jump_here(jump, &expr.1)?;
                builder.add_code(code! {
                    MOVE_BOOL out_reg, reg.index, 0;
                });
                reg.dealloc(builder.register_allocator_mut());
                Ok(ChosenRegister::shared(out_reg))
            }
        }
    }

    fn compile_global_definition(
        &mut self,
        definition: GlobalDefinition,
        location: &Location,
        register_choice: RegisterChoice,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<ChosenRegister, CompilationError> {
        match definition {
            GlobalDefinition::Constructor {
                generator_family,
                result_expr,
            } => {
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), location)?;
                match result_expr.0 {
                    ExprType::ProcessLiteral(..) => {
                        let family = self.processes.process_literal_map[&result_expr];
                        builder.add_code(code! {
                            NEW output_reg.index, family;
                        });
                    }
                    _ => {
                        builder.add_code(code! {
                            NEW output_reg.index, generator_family;
                            RECEIVE output_reg.index, output_reg.index, 0;
                        });
                    }
                }
                Ok(output_reg)
            }
            GlobalDefinition::BuiltinConstructor { index } => {
                let reg = register_choice.or_alloc(builder.register_allocator_mut(), location)?;
                builder.add_code(code! {
                    NEW_BUILTIN reg.index, index;
                });
                Ok(reg)
            }
            GlobalDefinition::Variable { global_index, .. } => {
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), location)?;
                builder.add_code(code! {
                    INIT 0, global_index;
                    LOAD output_reg.index, global_index;
                });
                Ok(output_reg)
            }
            GlobalDefinition::BuiltinConstant { index } => {
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), location)?;
                builder.add_code(code! {
                    CONST output_reg.index, index;
                });
                Ok(output_reg)
            }
            GlobalDefinition::Import { decl, index } => {
                let name = &decl.items[index].0;
                let definition = self.resolve_path(&decl.module, name, location)?;
                self.compile_global_definition(
                    definition.clone(),
                    location,
                    register_choice,
                    builder,
                )
            }
        }
    }

    fn resolve_path(
        &self,
        parts: &[String],
        name: &str,
        location: &Location,
    ) -> Result<&GlobalDefinition, CompilationError> {
        assert!(
            !parts.is_empty(),
            "module name should contain at least 1 part"
        );
        let mut module = &self.root_module;
        for (i, part) in parts.iter().enumerate() {
            let Some(sub_module) = module.get_sub_module(part) else {
                let full_name = parts[..=i].join("::");
                return CompilationError::err(format!("module {} not found", full_name), location);
            };
            module = sub_module;
        }
        module.get_definition(name).ok_or_else(|| {
            let module_name = parts.join("::");
            CompilationError::new(
                format!("name {}::{} not found", module_name, name),
                location,
            )
        })
    }

    fn compile_expr(
        &mut self,
        expr: &Rc<Expr>,
        register_choice: RegisterChoice,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<ChosenRegister, CompilationError> {
        match &expr.0 {
            ExprType::ConstantLiteral(constant) => {
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), &expr.1)?;
                let const_index = match constant {
                    ConstantLiteral::Integer(n) => self.output.int_constant(n.clone()),
                    &ConstantLiteral::Float(x) => self.output.float_constant(x),
                    ConstantLiteral::String(s) => self.output.string_symbol(s),
                    ConstantLiteral::Symbol(name) => self.output.symbol_constant(name),
                };
                let Some(const_index) = const_index else {
                    return CompilationError::err(
                        "program contains too many constants (limit is 4294967295)",
                        &expr.1,
                    );
                };
                builder.add_code(code! {
                    CONST output_reg.index, const_index;
                });
                Ok(output_reg)
            }
            ExprType::ProcessLiteral(_) => {
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), &expr.1)?;
                let family = self.processes.process_literal_map[expr];
                builder.add_code(code! {
                    NEW output_reg.index, family;
                });
                Ok(output_reg)
            }
            ExprType::Identifier(name) => match builder.environment().get_definition(name) {
                None => CompilationError::err(format!("name `{}` not found", name), &expr.1),
                Some(Left(global_def)) => self.compile_global_definition(
                    global_def.clone(),
                    &expr.1,
                    register_choice,
                    builder,
                ),
                Some(Right(&LocalDefinition::Variable { index })) => {
                    let output_reg = register_choice.use_existing_or_alloc(
                        index,
                        builder.register_allocator_mut(),
                        &expr.1,
                    )?;
                    self.compile_move(output_reg.index, index, builder);
                    Ok(output_reg)
                }
                Some(Right(LocalDefinition::Constructor { .. })) => todo!(),
            },
            ExprType::Path(parts) => {
                assert!(!parts.len() >= 2, "path should contain at least 2 parts");
                let Some((name, modules)) = parts.split_last() else {
                    unreachable!();
                };
                let definition = self.resolve_path(modules, name, &expr.1)?;
                self.compile_global_definition(
                    definition.clone(),
                    &expr.1,
                    register_choice,
                    builder,
                )
            }
            ExprType::In => {
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), &expr.1)?;
                builder.add_code(code! {
                    IN output_reg.index, 0, 0;
                });
                Ok(output_reg)
            }
            ExprType::Binary(BinaryOperator::And | BinaryOperator::Or, ..) => {
                self.compile_short_circuit_op(expr, register_choice, builder)
            }
            &ExprType::Binary(op, ref lhs, ref rhs) => {
                let lhs_reg = self.compile_expr(lhs, RegisterChoice::Any, builder)?;
                let rhs_reg = self.compile_expr(rhs, RegisterChoice::Any, builder)?;
                lhs_reg.dealloc(builder.register_allocator_mut());
                rhs_reg.dealloc(builder.register_allocator_mut());
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), &expr.1)?;
                self.compile_binary_op(op, lhs_reg.index, rhs_reg.index, output_reg.index, builder);
                Ok(output_reg)
            }
            ExprType::Unary(op, rhs) => {
                let rhs_reg = self.compile_expr(rhs, RegisterChoice::Any, builder)?;
                rhs_reg.dealloc(builder.register_allocator_mut());
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), &expr.1)?;
                match op {
                    UnaryOperator::Plus => builder.add_code(code! {
                        UNARY_PLUS output_reg.index, rhs_reg.index, 0;
                    }),
                    UnaryOperator::Minus => builder.add_code(code! {
                        NEGATE output_reg.index, rhs_reg.index, 0;
                    }),
                    UnaryOperator::Not => builder.add_code(code! {
                        NOT output_reg.index, rhs_reg.index, 0;
                    }),
                }
                Ok(output_reg)
            }
            ExprType::Parenthesized(inner) => {
                self.compile_expr(inner, RegisterChoice::Any, builder)
            }
            ExprType::Send(proc, arg) => match arg.typ {
                ArgumentType::Single => {
                    let proc_reg = self.compile_expr(proc, RegisterChoice::Any, builder)?;
                    let arg_reg = self.compile_expr(&arg.expr, RegisterChoice::Any, builder)?;
                    proc_reg.dealloc(builder.register_allocator_mut());
                    arg_reg.dealloc(builder.register_allocator_mut());
                    let output_reg =
                        register_choice.or_alloc(builder.register_allocator_mut(), &expr.1)?;
                    builder.add_code(code! {
                        SEND output_reg.index, proc_reg.index, arg_reg.index;
                    });
                    Ok(output_reg)
                }
                ArgumentType::InLoop => {
                    let register_choice = match register_choice {
                        RegisterChoice::AllocNew | RegisterChoice::Existing(_) => register_choice,
                        RegisterChoice::Any => RegisterChoice::AllocNew,
                    };
                    let out_reg = self.compile_expr(proc, register_choice, builder)?;
                    let in_reg = builder.register_allocator_mut().alloc_temporary(&expr.1)?;
                    let bool_reg = builder.register_allocator_mut().alloc_temporary(&expr.1)?;
                    builder.register_allocator_mut().dealloc(in_reg);
                    builder.register_allocator_mut().dealloc(bool_reg);
                    builder.add_code(code! {
                        OPT_IN in_reg, bool_reg, 0;
                        JUMP_UNLESS bool_reg, 3;
                        SEND out_reg.index, out_reg.index, in_reg;
                        OPT_IN in_reg, bool_reg, 0;
                        JUMP_IF bool_reg, !2;
                    });
                    Ok(out_reg)
                }
                ArgumentType::ReceiveLoop => {
                    let proc_reg = self.compile_expr(proc, RegisterChoice::AllocNew, builder)?;
                    let source_reg = self.compile_expr(&arg.expr, RegisterChoice::Any, builder)?;
                    let recv_reg = builder.register_allocator_mut().alloc_temporary(&expr.1)?;
                    let bool_reg = builder.register_allocator_mut().alloc_temporary(&expr.1)?;
                    source_reg.dealloc(builder.register_allocator_mut());
                    builder.register_allocator_mut().dealloc(recv_reg);
                    builder.register_allocator_mut().dealloc(bool_reg);
                    builder.add_code(code! {
                        NO_IN source_reg.index, 0, 0;
                        TRY_RECEIVE recv_reg, bool_reg, source_reg.index;
                        JUMP_UNLESS bool_reg, 4;
                        SEND proc_reg.index, proc_reg.index, recv_reg;
                        NO_IN source_reg.index, 0, 0;
                        TRY_RECEIVE recv_reg, bool_reg, source_reg.index;
                        JUMP_IF bool_reg, !3;
                    });
                    let out_reg = match register_choice {
                        RegisterChoice::Any | RegisterChoice::AllocNew => proc_reg,
                        RegisterChoice::Existing(reg) => {
                            proc_reg.dealloc(builder.register_allocator_mut());
                            self.compile_move(reg, proc_reg.index, builder);
                            ChosenRegister::shared(reg)
                        }
                    };
                    Ok(out_reg)
                }
            },
            ExprType::Receive(inner) => {
                let source_reg = self.compile_expr(inner, RegisterChoice::Any, builder)?;
                source_reg.dealloc(builder.register_allocator_mut());
                let output_reg =
                    register_choice.or_alloc(builder.register_allocator_mut(), &expr.1)?;
                builder.add_code(code! {
                    NO_IN source_reg.index, 0, 0;
                    RECEIVE output_reg.index, source_reg.index, 0;
                });
                Ok(output_reg)
            }
        }
    }

    fn expr_needs_error_propagation(expr: &Rc<Expr>) -> bool {
        match &expr.0 {
            ExprType::Identifier(..) | ExprType::Path(..) | ExprType::Send(..) => true,
            ExprType::Parenthesized(inner) => Self::expr_needs_error_propagation(inner),
            _ => false,
        }
    }

    fn compile_block(
        &mut self,
        stmts: &[Rc<Stmt>],
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<(), CompilationError> {
        builder.enter_new_scope();
        for stmt in stmts {
            self.compile_stmt(stmt, builder)?;
        }
        builder.exit_scope();
        Ok(())
    }

    fn compile_condition(
        &mut self,
        cond: &Condition,
        context: AssignmentContext,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<AssignmentCompilationResult, CompilationError> {
        assert_matches!(context, AssignmentContext::If | AssignmentContext::While);
        match cond {
            Condition::Boolean(bool_expr) => {
                let continue_target = builder.next_instruction_index();
                let reg = self.compile_expr(bool_expr, RegisterChoice::Any, builder)?;
                reg.dealloc(builder.register_allocator_mut());
                let jump = builder.add_jump(instruction! {
                    JUMP_UNLESS reg.index, 0;
                });
                Ok(AssignmentCompilationResult {
                    continue_target,
                    skip_jumps: vec![jump],
                    temporary_indices: Vec::new(),
                })
            }
            Condition::Assignment(stmt) => {
                builder.enter_new_scope();
                let StmtType::Assignment(assignment) = &stmt.0 else {
                    return CompilationError::err("expected an assignment", &stmt.1);
                };
                assignment::compile(self, builder, Rc::clone(assignment), context)
            }
        }
    }

    fn compile_switch(
        &mut self,
        stmt: &Rc<Stmt>,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<(), CompilationError> {
        let StmtType::Switch(expr, cases) = &stmt.0 else {
            panic!("expected a switch statement");
        };
        let reg = self.compile_expr(expr, RegisterChoice::Any, builder)?;
        let mut success_jumps = Vec::new();
        let mut fallthrough_jumps = Vec::new();
        let mut end_jumps: Vec<usize> = Vec::new();
        for (case_no, case) in cases.iter().enumerate() {
            let is_last_case = case_no == cases.len() - 1;
            if let Some(values) = &case.values {
                assert_ne!(values.len(), 0);
                for (i, value) in values.iter().enumerate() {
                    let value_reg = self.compile_expr(value, RegisterChoice::Any, builder)?;
                    value_reg.dealloc(builder.register_allocator_mut());
                    let bool_reg = builder.register_allocator_mut().alloc_temporary(&value.1)?;
                    builder.register_allocator_mut().dealloc(bool_reg);
                    builder.add_code(code! {
                        EQUALS bool_reg, value_reg.index, reg.index;
                    });
                    if i == values.len() - 1 {
                        fallthrough_jumps.push(builder.add_jump(instruction! {
                            JUMP_UNLESS bool_reg, 0;
                        }));
                    } else {
                        success_jumps.push(builder.add_jump(instruction! {
                            JUMP_IF bool_reg, 0;
                        }));
                    }
                }
            }
            for jump in success_jumps.drain(..) {
                builder.link_jump_here(jump, &case.location)?;
            }
            if is_last_case {
                reg.dealloc(builder.register_allocator_mut());
            }
            self.compile_block(&case.body, builder)?;
            if !is_last_case || case.values.is_some() {
                end_jumps.push(builder.add_jump(instruction! {
                    JUMP 0, 0;
                }));
            }
            for jump in fallthrough_jumps.drain(..) {
                builder.link_jump_here(jump, &case.location)?;
            }
        }
        if cases.last().unwrap().values.is_some() {
            reg.dealloc(builder.register_allocator_mut());
            builder.add_code(code! {
                ERROR_NO_CASE 0, 0, 0;
            });
        }
        for &jump in &end_jumps {
            builder.link_jump_here(jump, &stmt.1)?;
        }
        Ok(())
    }

    fn compile_args<F>(
        &mut self,
        args: &[Argument],
        builder: &mut ProcessFamilyBuilder,
        mut each_arg: F,
    ) -> Result<(), CompilationError>
    where
        F: FnMut(u16, &mut ProcessFamilyBuilder) -> Result<(), CompilationError>,
    {
        for arg in args {
            match arg.typ {
                ArgumentType::Single => {
                    let reg = self.compile_expr(&arg.expr, RegisterChoice::Any, builder)?;
                    each_arg(reg.index, builder)?;
                    reg.dealloc(builder.register_allocator_mut());
                }
                ArgumentType::InLoop => {
                    let in_reg = builder
                        .register_allocator_mut()
                        .alloc_temporary(&arg.location)?;
                    let bool_reg = builder
                        .register_allocator_mut()
                        .alloc_temporary(&arg.location)?;
                    builder.register_allocator_mut().dealloc(bool_reg);
                    builder.add_code(code! {
                        OPT_IN in_reg, bool_reg, 0;
                    });

                    let continue_jump_target = builder.next_instruction_index();
                    builder.enter_new_loop(continue_jump_target, arg.location.clone());
                    builder.add_break_jump(instruction! {
                        JUMP_UNLESS bool_reg, 0;
                    });

                    each_arg(in_reg, builder)?;
                    builder.register_allocator_mut().dealloc(in_reg);

                    builder.add_code(code! {
                        OPT_IN in_reg, bool_reg, 0;
                    });
                    builder.add_continue_jump(instruction! {
                        JUMP_IF bool_reg, 0;
                    })?;
                    builder.exit_loop()?;
                }
                ArgumentType::ReceiveLoop => {
                    // Register 0 might be overwritten, so use AllocNew.
                    let source_reg =
                        self.compile_expr(&arg.expr, RegisterChoice::AllocNew, builder)?;

                    let recv_reg = builder
                        .register_allocator_mut()
                        .alloc_temporary(&arg.location)?;
                    let bool_reg = builder
                        .register_allocator_mut()
                        .alloc_temporary(&arg.location)?;
                    builder.register_allocator_mut().dealloc(bool_reg);
                    builder.add_code(code! {
                        NO_IN source_reg.index, 0, 0;
                        TRY_RECEIVE recv_reg, bool_reg, source_reg.index;
                    });

                    let continue_jump_target = builder.next_instruction_index();
                    builder.enter_new_loop(continue_jump_target, arg.location.clone());
                    builder.add_break_jump(instruction! {
                        JUMP_UNLESS bool_reg, 0;
                    });

                    each_arg(recv_reg, builder)?;
                    builder.register_allocator_mut().dealloc(recv_reg);

                    builder.add_code(code! {
                        NO_IN source_reg.index, 0, 0;
                        TRY_RECEIVE recv_reg, bool_reg, source_reg.index;
                    });
                    builder.add_continue_jump(instruction! {
                        JUMP_IF bool_reg, 0;
                    })?;
                    builder.exit_loop()?;
                    source_reg.dealloc(builder.register_allocator_mut());
                }
            }
        }
        Ok(())
    }

    fn compile_stmt(
        &mut self,
        stmt: &Rc<Stmt>,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<(), CompilationError> {
        match &stmt.0 {
            StmtType::Expr(expr) => {
                let reg = self.compile_expr(expr, RegisterChoice::Any, builder)?;
                reg.dealloc(builder.register_allocator_mut());
                if Self::expr_needs_error_propagation(expr) {
                    builder.add_code(code! {
                        PROPAGATE reg.index, 0, 0;
                    });
                }
                Ok(())
            }
            StmtType::Jump(JumpType::Stop) => {
                builder.add_code(code! {
                    STOP 0, 0, 0;
                });
                Ok(())
            }
            StmtType::Jump(JumpType::Break) => {
                if !builder.is_in_loop() {
                    return CompilationError::err(
                        "a `break` statement must appear inside a loop",
                        &stmt.1,
                    );
                }
                builder.add_break_jump(instruction! {
                    JUMP 0, 0;
                });
                Ok(())
            }
            StmtType::Jump(JumpType::Continue) => {
                if !builder.is_in_loop() {
                    return CompilationError::err(
                        "a `continue` statement must appear inside a loop",
                        &stmt.1,
                    );
                }
                builder.add_continue_jump(instruction! {
                    JUMP 0, 0;
                })
            }
            StmtType::Debug(expr) => {
                let reg = self.compile_expr(expr, RegisterChoice::Any, builder)?;
                reg.dealloc(builder.register_allocator_mut());
                builder.add_code(code! {
                    DEBUG reg.index, 0, 0;
                });
                Ok(())
            }
            StmtType::Declaration(vars) => {
                for (name, location) in vars {
                    let index = builder.register_allocator_mut().alloc(location)?;
                    builder.environment_mut().add_local(
                        name.clone(),
                        LocalDefinition::Variable { index },
                        location,
                    )?;
                    builder.add_code(code! {
                        CONST index, self.const_undefined_index;
                    });
                }
                Ok(())
            }
            StmtType::Assignment(assignment) => {
                assignment::compile(
                    self,
                    builder,
                    Rc::clone(assignment),
                    AssignmentContext::Normal,
                )?;
                Ok(())
            }
            StmtType::AssignmentElse(assignment, else_) => {
                let assignment_result = assignment::compile(
                    self,
                    builder,
                    Rc::clone(assignment),
                    AssignmentContext::AssignElse,
                )?;
                self.compile_block(else_, builder)?;
                for jump in assignment_result.skip_jumps {
                    builder.link_jump_here(jump, &assignment.location)?;
                }
                Ok(())
            }
            StmtType::Out(outputs) => self.compile_args(outputs, builder, |output, builder| {
                builder.add_code(code! {
                    OUT output, 0, 0;
                });
                Ok(())
            }),
            StmtType::Raise(raise_type, args) => {
                let error_reg = builder.register_allocator_mut().alloc_temporary(&stmt.1)?;
                let approx_size = u32::try_from(args.len()).unwrap_or(u32::MAX);
                builder.add_code(code! {
                    NEW_ERROR error_reg, approx_size;
                });
                self.compile_args(args, builder, |value, builder| {
                    builder.add_code(code! {
                        EXTEND_ERROR error_reg, value, 0;
                    });
                    Ok(())
                })?;
                builder.register_allocator_mut().dealloc(error_reg);
                match raise_type {
                    RaiseType::Err => builder.add_code(code! {
                        ERR error_reg, 0, 0;
                    }),
                    RaiseType::Throw => builder.add_code(code! {
                        THROW error_reg, 0, 0;
                    }),
                }
                Ok(())
            }
            StmtType::If(cond, then, else_) => {
                let AssignmentCompilationResult {
                    skip_jumps,
                    temporary_indices,
                    ..
                } = self.compile_condition(cond, AssignmentContext::If, builder)?;
                debug_assert!(temporary_indices.is_empty());
                self.compile_block(then, builder)?;
                let true_jump = if else_.is_empty() {
                    None
                } else {
                    Some(builder.add_jump(instruction! {
                        JUMP 0, 0;
                    }))
                };
                for jump in skip_jumps {
                    builder.link_jump_here(jump, &stmt.1)?;
                }
                match cond {
                    Condition::Boolean(_) => {}
                    Condition::Assignment(_) => builder.exit_scope(),
                }
                if !else_.is_empty() {
                    self.compile_block(else_, builder)?;
                    builder.link_jump_here(true_jump.unwrap(), &stmt.1)?;
                }
                Ok(())
            }
            StmtType::Switch(..) => self.compile_switch(stmt, builder),
            StmtType::Loop(cond, stmts) => {
                let condition_result = if let Some(cond) = cond {
                    self.compile_condition(cond, AssignmentContext::While, builder)?
                } else {
                    AssignmentCompilationResult {
                        continue_target: builder.next_instruction_index(),
                        skip_jumps: Vec::new(),
                        temporary_indices: Vec::new(),
                    }
                };
                builder.enter_new_loop(condition_result.continue_target, stmt.1.clone());
                self.compile_block(stmts, builder)?;
                builder.add_continue_jump(instruction! {
                    JUMP 0, 0;
                })?;
                builder.exit_loop()?;
                for jump in condition_result.skip_jumps {
                    builder.link_jump_here(jump, &stmt.1)?;
                }
                for tmp in condition_result.temporary_indices {
                    builder.register_allocator_mut().dealloc(tmp);
                }
                if matches!(cond, Some(Condition::Assignment(_))) {
                    builder.exit_scope();
                }
                Ok(())
            }
            StmtType::Try(_, _) => todo!(),
        }
    }

    fn compile_process_literal(
        &mut self,
        literal: Rc<Expr>,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<(), CompilationError> {
        let Expr(ExprType::ProcessLiteral(stmts), _) = &*literal else {
            panic!("expected a process literal expression");
        };
        for stmt in stmts {
            self.compile_stmt(stmt, builder)?;
        }
        builder.add_code(code! {
            STOP 0, 0, 0;
        });
        Ok(())
    }

    fn compile_lazy_initializer(
        &mut self,
        lazy_init: Rc<Assignment>,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<(), CompilationError> {
        let family_index = self.processes.lazy_initializer_map[&lazy_init];
        let global_index = self.processes.global_var_map[&lazy_init];
        if self.output.global_variable(Some(family_index)).is_none() {
            return CompilationError::err(
                "too many globals and constructors (limit is 4294967295)",
                &lazy_init.location,
            );
        }
        let value = &lazy_init.values[0];
        let out_reg = self.compile_expr(value, RegisterChoice::Any, builder)?;
        out_reg.dealloc(builder.register_allocator_mut());
        builder.add_code(code! {
            STORE out_reg.index, global_index;
            STOP 0, 0, 0;
        });
        Ok(())
    }

    fn compile_constructor(
        &mut self,
        constructor: Rc<Assignment>,
        builder: &mut ProcessFamilyBuilder,
    ) -> Result<(), CompilationError> {
        let value = &constructor.values[0];
        let reg = self.compile_expr(value, RegisterChoice::Existing(0), builder)?;
        reg.dealloc(builder.register_allocator_mut());
        builder.add_code(code! {
            OUT 0, 0, 0;
            STOP 0, 0, 0;
        });
        Ok(())
    }

    // Outputs a generic error on failure.
    fn add_const_symbol(&mut self, name: &str) -> Result<u32, CompilationError> {
        let Some(index) = self.output.symbol_constant(name) else {
            return CompilationError::err(
                "program contains too many constants (limit is 4294967295)",
                &Location::default(),
            );
        };
        Ok(index)
    }

    fn add_initial_process(&mut self) -> Result<(), CompilationError> {
        let family = {
            let mut builder = ProcessFamilyBuilder::new();
            let alloc = builder.register_allocator_mut();
            let reg1 = alloc.alloc(&Location::default())?;
            let reg2 = alloc.alloc(&Location::default())?;
            let reg3 = alloc.alloc(&Location::default())?;
            alloc.dealloc(reg1);
            alloc.dealloc(reg2);
            alloc.dealloc(reg3);
            match self.processes.global_definitions.get("Main") {
                None => return CompilationError::err("no Main process", &Location::default()),
                Some(&GlobalDefinition::Constructor {
                    generator_family, ..
                }) => builder.add_code(code! {
                    NEW 1, generator_family;
                    RECEIVE 1, 1, 0;
                }),
                Some(&GlobalDefinition::Variable { global_index, .. }) => {
                    builder.add_code(code! {
                        INIT 0, global_index;
                        LOAD 1, global_index;
                    });
                }
                Some(
                    GlobalDefinition::BuiltinConstructor { .. }
                    | GlobalDefinition::BuiltinConstant { .. },
                ) => {
                    panic!("Main should not be builtin")
                }
                Some(GlobalDefinition::Import { .. }) => {
                    return CompilationError::err(
                        "Main cannot be imported from another module",
                        &Location::default(),
                    );
                }
            }
            let user_family_index = builtin::ROOT_MODULE.constructor_definitions["User"];
            let const_stop = self.add_const_symbol("Stop")?;
            let const_in = self.add_const_symbol("In")?;
            let const_opt_in = self.add_const_symbol("OptIn")?;
            let const_fork_in = self.add_const_symbol("ForkIn")?;
            let const_out = self.add_const_symbol("Out")?;
            let const_err = self.add_const_symbol("Err")?;
            builder.add_code(code! {
                NEW_BUILTIN 3, user_family_index;
                STATE 2, 1, 0;
                CONST 0, const_out;
                EQUALS 0, 0, 2;
                JUMP_UNLESS 0, 4;
                PEEK 0, 1, 0;
                SEND 3, 3, 0;
                RECEIVE 0, 1, 0;
                JUMP 0, !7;
                CONST 0, const_in;
                EQUALS 0, 0, 2;
                JUMP_IF 0, 3;
                CONST 0, const_fork_in;
                EQUALS 0, 0, 2;
                JUMP_UNLESS 0, 4;
                NO_IN 3, 0, 0;
                RECEIVE 0, 3, 0;
                SEND 1, 1, 0;
                JUMP 0, !17;
                CONST 0, const_opt_in;
                EQUALS 0, 0, 2;
                JUMP_UNLESS 0, 7;
                NO_IN 3, 0, 0;
                TRY_RECEIVE 0, 2, 3;
                JUMP_UNLESS 2, 2;
                SEND 1, 1, 0;
                JUMP 0, !25;
                NO_IN 1, 0, 0;
                JUMP 0, !27;
                CONST 0, const_stop;
                EQUALS 0, 0, 2;
                JUMP_UNLESS 0, 1;
                EXIT 0, 0, 0;
                CONST 0, const_err;
                EQUALS 0, 0, 2;
                JUMP_UNLESS 0, 4;
                PEEK_ERR 0, 1, 0;
                DISPLAY_ERROR 0, 0, 0;
                RECEIVE_ERR 0, 1, 0;
                JUMP 0, !38;
                UNREACHABLE 0, 0, 0;
            });
            builder.build()
        };
        self.output.initial_process_family(family);
        Ok(())
    }

    pub fn build(mut self) -> Result<Vm, CompilationError> {
        for (proc, family) in self.processes.process_literal_map.clone() {
            let family = Rc::clone(&self.processes.process_families[family as usize]);
            let mut family = family.borrow_mut();
            self.compile_process_literal(proc, &mut family)?;
        }
        for decl in self.code.declarations.clone() {
            let GlobalDeclaration::Assignment(assignment) = decl else {
                continue;
            };
            assert_eq!(assignment.targets.len(), 1);
            assert_eq!(assignment.values.len(), 1);
            match assignment.targets[0].typ {
                AssignmentType::Declaration => {
                    let family = self.processes.lazy_initializer_map[&assignment];
                    let builder = Rc::clone(&self.processes.process_families[family as usize]);
                    self.compile_lazy_initializer(assignment, &mut builder.borrow_mut())?;
                }
                AssignmentType::Constructor => {
                    let family = self.processes.constructor_map[&assignment];
                    let builder = Rc::clone(&self.processes.process_families[family as usize]);
                    self.compile_constructor(assignment, &mut builder.borrow_mut())?;
                }
                _ => unreachable!(),
            }
        }
        for builder in self.processes.process_families.clone() {
            self.output.process_family(builder.borrow_mut().build());
        }
        // The initial process must be added last, after everything is compiled and built.
        self.add_initial_process()?;
        Ok(self.output.build())
    }
}
