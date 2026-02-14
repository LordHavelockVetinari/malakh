use std::mem;

use crate::compile::environment::ProcessEnvironment;
use crate::compile::error::CompilationError;
use crate::compile::register_allocator::RegisterAllocator;
use crate::parse::location::Location;
use crate::vm::Instruction;
use crate::vm::user_process::UserProcessFamily;

struct LoopContext {
    location: Location,
    break_jump_addresses: Vec<usize>,
    continue_jump_target: usize,
}

pub struct ProcessFamilyBuilder {
    code: Vec<Instruction>,
    register_allocator: RegisterAllocator,
    environment: ProcessEnvironment,
    num_unlinked_jumps: usize,
    loops: Vec<LoopContext>,
}

impl ProcessFamilyBuilder {
    pub fn new() -> Self {
        let mut this = Self {
            code: vec![],
            register_allocator: RegisterAllocator::new(),
            environment: ProcessEnvironment::new(),
            num_unlinked_jumps: 0,
            loops: Vec::new(),
        };
        this.enter_new_scope();
        this
    }

    pub fn add_code(&mut self, code: &[Instruction]) {
        self.code.extend(code);
    }

    pub fn register_allocator_mut(&mut self) -> &mut RegisterAllocator {
        &mut self.register_allocator
    }

    pub fn environment(&self) -> &ProcessEnvironment {
        &self.environment
    }

    pub fn environment_mut(&mut self) -> &mut ProcessEnvironment {
        &mut self.environment
    }

    pub fn enter_new_scope(&mut self) {
        self.environment.enter_new_scope();
    }

    pub fn exit_scope(&mut self) {
        self.environment.exit_scope(&mut self.register_allocator);
    }

    pub fn enter_new_loop(&mut self, continue_jump_target: usize, location: Location) {
        self.loops.push(LoopContext {
            break_jump_addresses: Vec::new(),
            continue_jump_target,
            location,
        });
    }

    pub fn exit_loop(&mut self) -> Result<(), CompilationError> {
        let ctx = self.loops.pop().expect("loops should not be empty");
        for addr in ctx.break_jump_addresses {
            self.link_jump_here(addr, &ctx.location)?;
        }
        Ok(())
    }

    pub fn is_in_loop(&mut self) -> bool {
        !self.loops.is_empty()
    }

    pub fn add_jump(&mut self, inst: Instruction) -> usize {
        let index = self.code.len();
        self.code.push(inst);
        self.num_unlinked_jumps += 1;
        index
    }

    pub fn add_break_jump(&mut self, inst: Instruction) {
        let addr = self.add_jump(inst);
        let ctx = self.loops.last_mut().expect("loops should not be empty");
        ctx.break_jump_addresses.push(addr);
    }

    pub fn add_continue_jump(&mut self, inst: Instruction) -> Result<(), CompilationError> {
        let addr = self.add_jump(inst);
        let &LoopContext {
            continue_jump_target,
            ref location,
            ..
        } = self.loops.last().expect("loops should not be empty");
        let location = location.clone();
        self.link_jump_absolute(addr, continue_jump_target, &location)
    }

    pub fn link_jump_relative(&mut self, index: usize, relative_target: i32) {
        let inst = &mut self.code[index];
        let opcode = inst.opcode();
        let op1 = inst.operand1();
        *inst = Instruction::two_operand(opcode, op1, relative_target as u32);
        self.num_unlinked_jumps = self
            .num_unlinked_jumps
            .checked_sub(1)
            .expect("link_jump called without add_jump");
    }

    pub fn link_jump_absolute(
        &mut self,
        index: usize,
        target: usize,
        location: &Location,
    ) -> Result<(), CompilationError> {
        let relative_target = target as i64 - index as i64 - 1;
        let Ok(relative_target) = i32::try_from(relative_target) else {
            return CompilationError::err("process is too long to be compiled", location);
        };
        self.link_jump_relative(index, relative_target);
        Ok(())
    }

    pub fn next_instruction_index(&self) -> usize {
        self.code.len()
    }

    pub fn link_jump_here(
        &mut self,
        index: usize,
        location: &Location,
    ) -> Result<(), CompilationError> {
        let here = self.next_instruction_index();
        self.link_jump_absolute(index, here, location)
    }

    pub fn build(&mut self) -> UserProcessFamily {
        self.exit_scope();
        assert!(
            self.register_allocator.are_all_freed(),
            "forgot to deallocate some registers"
        );
        assert_eq!(self.num_unlinked_jumps, 0, "forgot to link all jumps");
        assert!(self.loops.is_empty(), "forgot to exit some loops");
        UserProcessFamily {
            code: mem::take(&mut self.code).leak(),
            memory_len: self.register_allocator.required_num_registers(),
        }
    }
}
