use std::borrow::Cow;
use std::cmp::Ordering;
use std::io::Write;
use std::mem;

use either::Either::{Left, Right};

use crate::vm::builtin_process::BuiltinProcessRef;
use crate::vm::capture::CaptureRef;
use crate::vm::error::ErrorRef;
use crate::vm::process::{ProcessRef, ProcessState};
use crate::vm::user_process::UserProcessRef;
use crate::vm::{Value, throw_from_current_process};

use super::{Instruction, Vm};

macro_rules! opcodes {
    ($($(#[$meta:meta])* $name:ident ($arity:literal) = $value:literal;)*) => {
        $(
            $(#[$meta])*
            pub const $name: u16 = $value;
        )*

        pub fn opcode_arity(opcode: u16) -> Option<u32> {
            match opcode {
                $(
                    $value => Some($arity),
                )*
                _ => None,
            }
        }

        pub fn opcode_name(opcode: u16) -> Cow<'static, str> {
            match opcode {
                $(
                    $value => Cow::Borrowed(stringify!($name)),
                )*
                _ => Cow::Owned(format!("OPCODE#{}", opcode)),
            }
        }

        const _: () = {
            let values = [$($value),*];
            let mut i = 0;
            while i < values.len() - 1 {
                assert!(values[i] < values[i + 1]);
                i += 1;
            }
        };
    };
}

opcodes! {
    MOVE (3) = 1;
    //MOVE_FROM (3) = 2;
    ADD (3) = 4;
    SUBTRACT (3) = 5;
    MULTIPLY (3) = 6;
    DIVIDE (3) = 7;
    REMAINDER (3) = 8;
    POWER (3) = 9;
    UNARY_PLUS (3) = 10;
    NEGATE (3) = 11;
    EQUALS (3) = 14;
    NOT_EQUALS (3) = 15;
    LESS (3) = 16;
    LESS_OR_EQUAL (3) = 17;
    NOT (3) = 19;
    XOR (3) = 20;
    MOVE_BOOL (3) = 21;
    JUMP (2) = 25;
    JUMP_IF (2) = 26;
    JUMP_UNLESS (2) = 27;
    INIT (2) = 30;
    LOAD (2) = 31;
    STORE (2) = 32;
    CONST (2) = 36;
    NEW (2) = 37;
    NEW_BUILTIN (2) = 38;
    CAPTURE (3) = 40;
    LOAD_CAPTURE (3) = 41;
    STORE_CAPTURE (3) = 42;
    STOP (3) = 50;
    OUT (3) = 70;
    IN (3) = 72;
    OPT_IN (3) = 73;
    RECEIVE (3) = 75;
    TRY_RECEIVE (3) = 76;
    PEEK (3) = 77;
    SEND (3) = 78;
    NO_IN (3) = 79;
    STATE (3) = 80;
    NEW_ERROR (2) = 90;
    EXTEND_ERROR (3) = 91;
    ERR (3) = 92;
    THROW (3) = 93;
    RECEIVE_ERR (3) = 94;
    PEEK_ERR (3) = 96;
    RETHROW (3) = 97;
    PROPAGATE (3) = 98;
    ERROR_MATCHES (3) = 99;
    DEBUG (3) = 100;
    EXIT (3) = 101;
    DISPLAY_ERROR (3) = 102;
    ERROR_NO_CASE (3) = 103;
    UNREACHABLE (3) = 255;
}

pub type InstructionFn = fn(&mut Vm, Instruction);

fn run_undefined(_vm: &mut Vm, inst: Instruction) {
    panic!("undefined instruction: {:?}", inst);
}

fn run_move(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), MOVE);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    *vm.register_mut(dst) = src;
}

/*fn run_move_from(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), MOVE_FROM);
    let (dst, src, index) = inst.as_three_operand();
    let src = vm.register(src);
    let Some(proc) = src.as_user_process_ref() else {
        todo!();
    };
    let result = proc.memory()[index as usize];
    *vm.register_mut(dst) = result;
}*/

fn run_add(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), ADD);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Ok(result) = Value::add(src1, src2, &mut vm.gc) else {
        throw_from_current_process!(
            vm,
            "type error: {} + {}",
            src1.type_name(),
            src2.type_name(),
        );
        return;
    };
    *vm.register_mut(dst) = result;
}

fn run_subtract(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), SUBTRACT);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Ok(result) = Value::subtract(src1, src2, &mut vm.gc) else {
        throw_from_current_process!(
            vm,
            "type error: {} - {}",
            src1.type_name(),
            src2.type_name(),
        );
        return;
    };
    *vm.register_mut(dst) = result;
}

fn run_multiply(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), MULTIPLY);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Ok(result) = Value::multiply(src1, src2, &mut vm.gc) else {
        throw_from_current_process!(
            vm,
            "type error: {} * {}",
            src1.type_name(),
            src2.type_name(),
        );
        return;
    };
    *vm.register_mut(dst) = result;
}

fn run_divide(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), DIVIDE);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Ok(result) = Value::divide(src1, src2, &mut vm.gc) else {
        throw_from_current_process!(
            vm,
            "type error: {} / {}",
            src1.type_name(),
            src2.type_name(),
        );
        return;
    };
    let Some(result) = result else {
        throw_from_current_process!(vm, "division by zero: {:?} / {:?}", src1, src2);
        return;
    };
    *vm.register_mut(dst) = result;
}

fn run_remainder(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), REMAINDER);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Ok(result) = Value::remainder(src1, src2, &mut vm.gc) else {
        throw_from_current_process!(
            vm,
            "type error: {} % {}",
            src1.type_name(),
            src2.type_name(),
        );
        return;
    };
    let Some(result) = result else {
        throw_from_current_process!(vm, "division by zero: {:?} % {:?}", src1, src2);
        return;
    };
    *vm.register_mut(dst) = result;
}

fn run_power(_vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), POWER);
    todo!();
}

fn run_unary_plus(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), UNARY_PLUS);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    if !src.is_number() {
        throw_from_current_process!(vm, "type error: +{}", src.type_name());
        return;
    }
    *vm.register_mut(dst) = src;
}

fn run_negate(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), NEGATE);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    let Ok(result) = src.negate(&mut vm.gc) else {
        throw_from_current_process!(vm, "type error: -{}", src.type_name());
        return;
    };
    *vm.register_mut(dst) = result;
}

fn run_equals(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), EQUALS);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let result = Value::from(src1 == src2);
    *vm.register_mut(dst) = result;
}

fn run_not_equals(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), NOT_EQUALS);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let result = Value::from(src1 != src2);
    *vm.register_mut(dst) = result;
}

fn run_less(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), LESS);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Ok(ord) = src1.compare(src2) else {
        throw_from_current_process!(
            vm,
            "incomparable types: {}, {}",
            src1.type_name(),
            src2.type_name(),
        );
        return;
    };
    *vm.register_mut(dst) = Value::from(ord == Some(Ordering::Less));
}

fn run_less_or_equal(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), LESS_OR_EQUAL);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Ok(ord) = src1.compare(src2) else {
        throw_from_current_process!(
            vm,
            "incomparable types: {}, {}",
            src1.type_name(),
            src2.type_name(),
        );
        return;
    };
    *vm.register_mut(dst) = Value::from(ord.is_some_and(Ordering::is_le));
}

fn run_not(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), NOT);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    let Some(src) = src.as_bool() else {
        if src.is_symbol() {
            throw_from_current_process!(vm, "expected a Boolean, got {:?}", src);
        } else {
            throw_from_current_process!(vm, "type error: not {}", src.type_name());
        }
        return;
    };
    let result = Value::from(!src);
    *vm.register_mut(dst) = result;
}

fn run_xor(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), XOR);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let (Some(src1), Some(src2)) = (src1.as_bool(), src2.as_bool()) else {
        if !src1.is_symbol() || !src2.is_symbol() {
            throw_from_current_process!(
                vm,
                "type error: {} xor {}",
                src1.type_name(),
                src2.type_name(),
            );
        } else {
            let non_bool = if src1.is_bool() { src2 } else { src1 };
            throw_from_current_process!(vm, "expected a Boolean, got {:?}", non_bool);
        }
        return;
    };
    let result = Value::from(src1 ^ src2);
    *vm.register_mut(dst) = result;
}

fn run_move_bool(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), MOVE_BOOL);
    let (dst, src, _) = inst.as_three_operand();
    let result = vm.register(src);
    if !result.is_bool() {
        throw_from_current_process!(vm, "expected a Boolean, got {:?}", result);
        return;
    }
    *vm.register_mut(dst) = result;
}

fn run_jump(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), JUMP);
    let (_, n) = inst.as_two_operand();
    let n = n as i32 as isize;
    vm.jump(n);
}

fn run_jump_unless(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), JUMP_UNLESS);
    let (src, n) = inst.as_two_operand();
    let src = vm.register(src);
    let n = n as i32 as isize;
    match src.as_bool() {
        None => throw_from_current_process!(vm, "expected a Boolean, got {:?}", src),
        Some(false) => vm.jump(n),
        Some(true) => {}
    }
}

fn run_jump_if(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), JUMP_IF);
    let (src, n) = inst.as_two_operand();
    let src = vm.register(src);
    let n = n as i32 as isize;
    match src.as_bool() {
        None => throw_from_current_process!(vm, "expected a Boolean, got {:?}", src),
        Some(false) => {}
        Some(true) => vm.jump(n),
    }
}

fn run_init(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), INIT);
    let (_, idx) = inst.as_two_operand();
    let var = &vm.global_variables[idx as usize];
    var.begin_init(vm);
}

fn run_load(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), LOAD);
    let (dst, idx) = inst.as_two_operand();
    let var = &vm.global_variables[idx as usize];
    let Some(&result) = var.value().get() else {
        throw_from_current_process!(vm, "recursively-defined global variable");
        return;
    };
    *vm.register_mut(dst) = result;
}

fn run_store(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), STORE);
    let (src, idx) = inst.as_two_operand();
    let src = vm.register(src);
    let var = &vm.global_variables[idx as usize];
    if var.value().set(src).is_err() {
        panic!("global variable initialized twice");
    }
}

fn run_const(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), CONST);
    let (dst, index) = inst.as_two_operand();
    let constant = vm.constants[index as usize];
    *vm.register_mut(dst) = constant;
}

fn run_capture(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), CAPTURE);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    let result = Value::from(CaptureRef::new(src, vm.gc_mut()));
    *vm.register_mut(dst) = result;
}

fn run_load_capture(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), LOAD_CAPTURE);
    let (dst, src, _) = inst.as_three_operand();
    let Some(src) = vm.register(src).as_capture_ref() else {
        panic!("LOAD_CAPTURE didn't get a capture")
    };
    *vm.register_mut(dst) = src.value();
}

fn run_store_capture(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), STORE_CAPTURE);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    let Some(mut dst) = vm.register(dst).as_capture_ref() else {
        panic!("STORE_CAPTURE didn't get a capture")
    };
    *dst.value_mut() = src;
}

fn run_new(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), NEW);
    let (dst, src) = inst.as_two_operand();
    let family = vm.user_process_families[src as usize];
    let proc = UserProcessRef::new(family, &mut vm.gc);
    *vm.register_mut(dst) = Value::from(proc);
    vm.enter_user_process(proc);
}

fn run_new_builtin(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), NEW_BUILTIN);
    let (dst, src) = inst.as_two_operand();
    let family = vm.get_builtin_family(src);
    let proc = BuiltinProcessRef::new(family, None, vm);
    *vm.register_mut(dst) = Value::from(proc);
}

fn run_stop(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), STOP);
    *vm.current_process().state_mut() = ProcessState::Stop;
    vm.pause_user_process();
}

fn run_out(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), OUT);
    let (src, _, _) = inst.as_three_operand();
    let src = vm.register(src);
    let mut proc = vm.current_process();
    *proc.output_slot_mut() = src;
    *proc.state_mut() = ProcessState::Out;
    vm.pause_user_process();
}

fn run_in(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), IN);
    let mut proc = vm.current_process();
    *proc.state_mut() = ProcessState::In;
    vm.pause_user_process();
}

fn run_opt_in(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), OPT_IN);
    let mut proc = vm.current_process();
    *proc.state_mut() = ProcessState::OptIn;
    vm.pause_user_process();
}

fn run_receive(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), RECEIVE);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    if let Some(mut proc) = src.as_builtin_process_ref() {
        match proc.state() {
            ProcessState::Out => {
                *vm.register_mut(dst) = mem::take(proc.output_slot_mut());
                vm.enter_builtin_process(proc, None);
            }
            ProcessState::Err => vm.propagate_error(proc),
            _ => throw_from_current_process!(
                vm,
                "failed to receive output: process was in {:?} state",
                proc.state(),
            ),
        }
        return;
    }
    let Some(mut proc) = src.as_user_process_ref() else {
        throw_from_current_process!(vm, "type error: [{}]", src.type_name());
        return;
    };
    match proc.state() {
        ProcessState::Out => {
            *vm.register_mut(dst) = mem::take(proc.output_slot_mut());
            *proc.state_mut() = ProcessState::Run;
            vm.enter_user_process(proc);
        }
        ProcessState::Err => vm.propagate_error(proc),
        _ => throw_from_current_process!(
            vm,
            "failed to receive output: process was in {:?} state",
            proc.state(),
        ),
    }
}

fn run_try_receive(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), TRY_RECEIVE);
    let (dst1, dst2, src) = inst.as_three_operand();
    let src = vm.register(src);
    if let Some(mut proc) = src.as_builtin_process_ref() {
        match proc.state() {
            ProcessState::Out => {
                *vm.register_mut(dst1) = mem::take(proc.output_slot_mut());
                *vm.register_mut(dst2) = Value::TRUE;
                vm.enter_builtin_process(proc, None);
            }
            ProcessState::Err => {
                vm.propagate_error(proc);
            }
            _ => {
                *vm.register_mut(dst2) = Value::FALSE;
            }
        }
        return;
    }
    let Some(mut proc) = src.as_user_process_ref() else {
        throw_from_current_process!(vm, "type error: [{}]", src.type_name());
        return;
    };
    match proc.state() {
        ProcessState::Out => {
            *vm.register_mut(dst1) = mem::take(proc.output_slot_mut());
            *vm.register_mut(dst2) = Value::TRUE;
            *proc.state_mut() = ProcessState::Run;
            vm.enter_user_process(proc);
        }
        ProcessState::Err => {
            vm.propagate_error(proc);
        }
        _ => {
            *vm.register_mut(dst2) = Value::FALSE;
        }
    }
}

fn run_peek(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), PEEK);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    let Some(proc) = src.as_any_process_ref() else {
        throw_from_current_process!(vm, "type error: [Peek {}]", src.type_name());
        return;
    };
    match proc.state() {
        ProcessState::Out => {
            *vm.register_mut(dst) = proc.output_slot();
        }
        ProcessState::Err => vm.propagate_error(proc),
        _ => {
            throw_from_current_process!(
                vm,
                "failed to peek: process was in {:?} state",
                proc.state()
            )
        }
    }
}

fn run_send(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), SEND);
    let (dst, src1, src2) = inst.as_three_operand();
    let src1 = vm.register(src1);
    let src2 = vm.register(src2);
    let Some(proc) = src1.as_any_process_ref() else {
        throw_from_current_process!(vm, "type error: cannot send input to {}", src1.type_name());
        return;
    };
    match (proc.state(), proc.builtin_or_user_defined()) {
        (ProcessState::In | ProcessState::OptIn | ProcessState::ForkIn, Left(proc)) => {
            let result = vm.enter_builtin_process(proc, Some(src2));
            *vm.register_mut(dst) = Value::from(result);
        }
        (ProcessState::In, Right(mut proc)) => {
            *vm.register_mut(dst) = src1;
            let in_inst = unsafe { *proc.instruction_pointer().sub(1) };
            debug_assert_eq!(in_inst.opcode(), IN);
            let (value_dst, _, _) = in_inst.as_three_operand();
            proc.memory_mut()[value_dst as usize] = src2;
            *proc.state_mut() = ProcessState::Run;
            vm.enter_user_process(proc);
        }
        (ProcessState::OptIn, Right(mut proc)) => {
            *vm.register_mut(dst) = src1;
            let opt_in_inst = unsafe { *proc.instruction_pointer().sub(1) };
            debug_assert_eq!(opt_in_inst.opcode(), OPT_IN);
            let (value_dst, bool_dst, _) = opt_in_inst.as_three_operand();
            let mem = proc.memory_mut();
            mem[value_dst as usize] = src2;
            mem[bool_dst as usize] = Value::TRUE;
            *proc.state_mut() = ProcessState::Run;
            vm.enter_user_process(proc);
        }
        (ProcessState::ForkIn, Right(_)) => todo!("send to user process in ForkIn state"),
        (ProcessState::Err, Left(p)) => vm.propagate_error(p),
        (ProcessState::Err, Right(p)) => vm.propagate_error(p),
        _ => throw_from_current_process!(
            vm,
            "failed to send input: process was in {:?} state",
            proc.state(),
        ),
    }
}

fn run_no_in(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), NO_IN);
    let (src, _, _) = inst.as_three_operand();
    let src = vm.register(src);
    if let Some(proc) = src.as_builtin_process_ref() {
        if proc.state() == ProcessState::OptIn {
            vm.enter_builtin_process(proc, None);
        }
        return;
    }
    let Some(mut proc) = src.as_user_process_ref() else {
        throw_from_current_process!(vm, "type error: [{}]", src.type_name());
        return;
    };
    if proc.state() != ProcessState::OptIn {
        return;
    }
    let opt_in_inst = unsafe { *proc.instruction_pointer().sub(1) };
    debug_assert_eq!(opt_in_inst.opcode(), OPT_IN);
    let (_, bool_dst, _) = opt_in_inst.as_three_operand();
    proc.memory_mut()[bool_dst as usize] = Value::FALSE;
    *proc.state_mut() = ProcessState::Run;
    vm.enter_user_process(proc);
}

fn run_state(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), STATE);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    let Some(proc) = src.as_any_process_ref() else {
        throw_from_current_process!(vm, "type error: [State {}]", src.type_name());
        return;
    };
    *vm.register_mut(dst) = Value::from(proc.state());
}

fn run_new_error(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), NEW_ERROR);
    let (dst, size) = inst.as_two_operand();
    let size = usize::try_from(size).expect("failed to allocate error");
    let mut error = ErrorRef::new(vm);
    error.reserve(size);
    *vm.register_mut(dst) = Value::from(error);
}

fn run_extend_error(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), EXTEND_ERROR);
    let (dst, src, _) = inst.as_three_operand();
    let mut dst = unsafe { ErrorRef::from_value(vm.register(dst), vm) };
    let src = vm.register(src);
    dst.extend(src);
}

fn run_err(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), ERR);
    let (src, _, _) = inst.as_three_operand();
    let src = unsafe { ErrorRef::from_value(vm.register(src), vm) };
    let mut proc = vm.current_process();
    *proc.output_slot_mut() = Value::from(src);
    *proc.state_mut() = ProcessState::Err;
    proc.set_can_resume();
    vm.pause_user_process();
}

fn run_throw(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), THROW);
    let (src, _, _) = inst.as_three_operand();
    let src = unsafe { ErrorRef::from_value(vm.register(src), vm) };
    vm.throw_from_current_process(src);
}

fn run_receive_err(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), RECEIVE_ERR);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    if let Some(mut proc) = src.as_builtin_process_ref() {
        if proc.state() != ProcessState::Err {
            throw_from_current_process!(
                vm,
                "failed to receive error: process was in {} state",
                src.type_name(),
            );
            return;
        }
        *vm.register_mut(dst) = mem::take(proc.output_slot_mut());
        vm.enter_builtin_process(proc, None);
        return;
    }
    let Some(mut proc) = src.as_user_process_ref() else {
        throw_from_current_process!(
            vm,
            "type error: attempt to receive error of {}",
            src.type_name(),
        );
        return;
    };
    match proc.state() {
        ProcessState::Err => {
            *vm.register_mut(dst) = mem::take(proc.output_slot_mut());
            if proc.take_can_resume() {
                *proc.state_mut() = ProcessState::Run;
                vm.enter_user_process(proc);
            } else {
                *proc.state_mut() = ProcessState::Stop;
            }
        }
        _ => throw_from_current_process!(
            vm,
            "failed to receive error: process was in {} state",
            src.type_name(),
        ),
    }
}

fn run_peek_err(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), PEEK_ERR);
    let (dst, src, _) = inst.as_three_operand();
    let src = vm.register(src);
    let Some(proc) = src.as_any_process_ref() else {
        throw_from_current_process!(
            vm,
            "type error: attempt to peek at error of {}",
            src.type_name(),
        );
        return;
    };
    if proc.state() != ProcessState::Err {
        throw_from_current_process!(
            vm,
            "failed to peek at error: process was in {} state",
            src.type_name(),
        );
    }
    *vm.register_mut(dst) = proc.output_slot();
}

fn run_rethrow(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), RETHROW);
    let (src, _, _) = inst.as_three_operand();
    let src = vm.register(src);
    let cause = unsafe { ErrorRef::from_value(src, vm) };
    let error = ErrorRef::new_propagated(vm, cause);
    vm.throw_from_current_process(error);
}

fn run_propagate(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), PROPAGATE);
    let (src, _, _) = inst.as_three_operand();
    let src = vm.register(src);
    if let Some(proc) = src.as_any_process_ref()
        && proc.state() == ProcessState::Err
    {
        vm.propagate_error(proc);
    }
}

fn run_error_matches(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), ERROR_MATCHES);
    let (dst, error_src, value_src) = inst.as_three_operand();
    let error_src = vm.register(error_src);
    let value_src = vm.register(value_src);
    let error = unsafe { ErrorRef::from_value(error_src, vm) };
    *vm.register_mut(dst) = Value::from(error.matches(value_src));
}

fn run_debug(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), DEBUG);
    let src = inst.operand1();
    let src = vm.register(src);
    eprintln!("DEBUG: {:?}", src);
}

fn run_exit(_: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), EXIT);
    std::process::exit(0);
}

fn run_display_error(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), DISPLAY_ERROR);
    let (src, _, _) = inst.as_three_operand();
    let src = unsafe { ErrorRef::from_value(vm.register(src), vm) };
    let mut stderr = std::io::stderr();
    if vm.options().raw_errors {
        for value in src.values() {
            let _ = value.write_to(&mut stderr);
            let _ = writeln!(stderr);
        }
    } else {
        let _ = src.pretty_print(&mut stderr);
    }
}

fn run_error_no_case(vm: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), ERROR_NO_CASE);
    throw_from_current_process!(vm, "no case matched in 'switch' statement")
}

fn run_unreachable(_: &mut Vm, inst: Instruction) {
    debug_assert_eq!(inst.opcode(), UNREACHABLE);
    unreachable!("unreachable instruction reached");
}

static OPCODE_TABLE: [InstructionFn; 256] = {
    let mut table = [run_undefined as InstructionFn; 256];
    table[MOVE as usize] = run_move;
    //table[MOVE_FROM as usize] = run_move_from;
    table[ADD as usize] = run_add;
    table[SUBTRACT as usize] = run_subtract;
    table[MULTIPLY as usize] = run_multiply;
    table[DIVIDE as usize] = run_divide;
    table[REMAINDER as usize] = run_remainder;
    table[POWER as usize] = run_power;
    table[UNARY_PLUS as usize] = run_unary_plus;
    table[NEGATE as usize] = run_negate;
    table[EQUALS as usize] = run_equals;
    table[NOT_EQUALS as usize] = run_not_equals;
    table[LESS as usize] = run_less;
    table[LESS_OR_EQUAL as usize] = run_less_or_equal;
    table[NOT as usize] = run_not;
    table[XOR as usize] = run_xor;
    table[MOVE_BOOL as usize] = run_move_bool;
    table[JUMP as usize] = run_jump;
    table[JUMP_IF as usize] = run_jump_if;
    table[JUMP_UNLESS as usize] = run_jump_unless;
    table[INIT as usize] = run_init;
    table[LOAD as usize] = run_load;
    table[STORE as usize] = run_store;
    table[CONST as usize] = run_const;
    table[CAPTURE as usize] = run_capture;
    table[LOAD_CAPTURE as usize] = run_load_capture;
    table[STORE_CAPTURE as usize] = run_store_capture;
    table[NEW as usize] = run_new;
    table[NEW_BUILTIN as usize] = run_new_builtin;
    table[STOP as usize] = run_stop;
    table[OUT as usize] = run_out;
    table[IN as usize] = run_in;
    table[OPT_IN as usize] = run_opt_in;
    table[RECEIVE as usize] = run_receive;
    table[TRY_RECEIVE as usize] = run_try_receive;
    table[PEEK as usize] = run_peek;
    table[SEND as usize] = run_send;
    table[NO_IN as usize] = run_no_in;
    table[STATE as usize] = run_state;
    table[NEW_ERROR as usize] = run_new_error;
    table[EXTEND_ERROR as usize] = run_extend_error;
    table[ERR as usize] = run_err;
    table[THROW as usize] = run_throw;
    table[RECEIVE_ERR as usize] = run_receive_err;
    table[PEEK_ERR as usize] = run_peek_err;
    table[RETHROW as usize] = run_rethrow;
    table[PROPAGATE as usize] = run_propagate;
    table[ERROR_MATCHES as usize] = run_error_matches;
    table[DEBUG as usize] = run_debug;
    table[EXIT as usize] = run_exit;
    table[DISPLAY_ERROR as usize] = run_display_error;
    table[ERROR_NO_CASE as usize] = run_error_no_case;
    table[UNREACHABLE as usize] = run_unreachable;
    table
};

pub fn get_fn(opcode: u16) -> InstructionFn {
    OPCODE_TABLE
        .get(opcode as usize)
        .copied()
        .unwrap_or(run_undefined)
}
