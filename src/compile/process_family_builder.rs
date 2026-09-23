use std::cell::OnceCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use crate::compile::capture_analysis::CaptureAnalysis;
use crate::compile::environment::{LocalDefinition, ProcessEnvironment};
use crate::compile::error::CompilationError;
use crate::compile::register_allocator::RegisterAllocator;
use crate::parse::location::Location;
use crate::parse::tree::AssignmentTarget;
use crate::vm::Instruction;
use crate::vm::user_process::{TryBody, UserProcessFamily};

struct LoopContext {
    location: Location,
    break_jump_addresses: Vec<usize>,
    continue_jump_target: usize,
}

pub struct TryBuilder {
    pub start: usize,
    pub end: usize,
}

pub struct ProcessFamilyBuilder {
    location: Location,
    code: Vec<Instruction>,
    register_allocator: RegisterAllocator,
    environment: ProcessEnvironment,
    num_unlinked_jumps: usize,
    loops: Vec<LoopContext>,
    try_starts: Vec<usize>,
    try_builders: Vec<TryBuilder>,
    capture_order: OnceCell<Vec<String>>,
    capture_indices: OnceCell<HashMap<String, u16>>,
}

impl ProcessFamilyBuilder {
    pub fn new(location: Location) -> Self {
        let mut this = Self {
            location,
            code: vec![],
            register_allocator: RegisterAllocator::new(),
            environment: ProcessEnvironment::new(),
            num_unlinked_jumps: 0,
            loops: Vec::new(),
            try_starts: Vec::new(),
            try_builders: Vec::new(),
            capture_order: OnceCell::new(),
            capture_indices: OnceCell::new(),
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

    pub fn is_in_try(&self) -> bool {
        !self.try_starts.is_empty()
    }

    pub fn begin_try_body(&mut self) {
        self.try_starts.push(self.next_instruction_index());
    }

    pub fn end_try_body(&mut self) {
        let start = self.try_starts.pop().expect("should be in try statement");
        self.try_builders.push(TryBuilder {
            start,
            end: self.next_instruction_index(),
        });
    }

    pub fn init_capture_indices(&mut self, indices: HashMap<String, u16>) {
        self.capture_indices
            .set(indices)
            .expect("capture_indices should be uninitialized");
    }

    pub fn init_capture_order(
        &mut self,
        capture_order: Vec<Rc<AssignmentTarget>>,
        capture_analysis: &CaptureAnalysis,
    ) -> Result<(), CompilationError> {
        let names: Vec<String> = capture_order.iter().map(|var| var.name.clone()).collect();
        self.capture_order
            .set(names)
            .expect("capture_order should be uninitialized");
        for var in capture_order {
            let index = self.register_allocator.alloc(&self.location)?;
            let definition = if capture_analysis.is_mutably_captured(Rc::clone(&var)) {
                LocalDefinition::MutablyCapturedVariable { index }
            } else {
                LocalDefinition::Variable { index }
            };
            self.environment
                .add_local(var.name.clone(), definition, &var.location)?;
        }
        Ok(())
    }

    pub fn set_non_capturing(&mut self) -> Result<(), CompilationError> {
        self.init_capture_indices(HashMap::new());
        self.init_capture_order(Vec::new(), &CaptureAnalysis::new())?;
        Ok(())
    }

    pub fn build(&mut self) -> UserProcessFamily {
        self.exit_scope();
        assert!(
            self.register_allocator.are_all_freed(),
            "forgot to deallocate some registers"
        );
        assert_eq!(self.num_unlinked_jumps, 0, "forgot to link all jumps");
        assert!(self.loops.is_empty(), "forgot to exit some loops");
        assert!(self.try_starts.is_empty(), "forgot to exit some try bodies");
        let code = mem::take(&mut self.code).leak();
        let try_bodies = self
            .try_builders
            .iter()
            .map(|builder| TryBody {
                start: &code[builder.start],
                end: &code[builder.end],
            })
            .collect::<Vec<_>>()
            .leak();
        try_bodies.sort_unstable_by_key(|body| body.end.addr() - body.start.addr());
        let capture_indices = self
            .capture_indices
            .get()
            .expect("capture_indices should be initialized");
        let capture_order = self
            .capture_order
            .get()
            .expect("capture_order should be uninitialized");
        assert!(capture_indices.len() == capture_order.len());
        let capture_indices = capture_order
            .iter()
            .map(|name| capture_indices[name])
            .collect::<Vec<u16>>()
            .leak();
        UserProcessFamily {
            code,
            memory_len: self.register_allocator.required_num_registers(),
            try_bodies,
            capture_indices,
        }
    }
}
