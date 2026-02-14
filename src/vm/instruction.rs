use core::fmt::{self, Debug};
use std::mem;

use crate::vm::opcode::opcode_arity;
use crate::vm::opcode::opcode_name;

use super::Vm;
use super::opcode;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C, align(8))]
pub struct Instruction([u16; 4]);

const fn split_u16s(n1_n2: u32) -> [u16; 2] {
    unsafe { mem::transmute::<u32, [u16; 2]>(n1_n2) }
}

fn join_u16s(n1: u16, n2: u16) -> u32 {
    unsafe { mem::transmute::<[u16; 2], u32>([n1, n2]) }
}

impl Instruction {
    pub const fn two_operand(opcode: u16, op1: u16, op23: u32) -> Self {
        let [op2, op3] = split_u16s(op23);
        Self([opcode, op1, op2, op3])
    }

    pub const fn three_operand(opcode: u16, op1: u16, op2: u16, op3: u16) -> Self {
        Self([opcode, op1, op2, op3])
    }

    pub fn opcode(self) -> u16 {
        self.0[0]
    }

    pub fn operand1(self) -> u16 {
        self.0[1]
    }

    pub fn operand2(self) -> u16 {
        self.0[2]
    }

    pub fn operand3(self) -> u16 {
        self.0[3]
    }

    pub fn operand23(self) -> u32 {
        join_u16s(self.operand2(), self.operand3())
    }

    pub fn as_two_operand(self) -> (u16, u32) {
        (self.operand1(), self.operand23())
    }

    pub fn as_three_operand(self) -> (u16, u16, u16) {
        (self.operand1(), self.operand2(), self.operand3())
    }

    pub fn run(self, vm: &mut Vm) {
        let instruction_fn = opcode::get_fn(self.opcode());
        instruction_fn(vm, self);
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let opcode = self.opcode();
        match opcode_arity(opcode) {
            Some(2) => {
                let (op1, op2) = self.as_two_operand();
                write!(f, "{} {}, {}", opcode_name(opcode), op1, op2 as i32)
            }
            Some(3) | None => {
                let (op1, op2, op3) = self.as_three_operand();
                write!(f, "{} {}, {}, {}", opcode_name(opcode), op1, op2, op3)
            }
            _ => write!(f, "{:?}", self.0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn constructor_test() {
        let inst1 = Instruction::two_operand(10, 20, 0xdeadbeef);
        assert_eq!(inst1.opcode(), 10);
        assert_eq!(inst1.operand1(), 20);
        assert_eq!(inst1.operand23(), 0xdeadbeef);
        let inst2 = Instruction::three_operand(100, 200, 300, 400);
        assert_eq!(inst2.opcode(), 100);
        assert_eq!(inst2.operand1(), 200);
        assert_eq!(inst2.operand2(), 300);
        assert_eq!(inst2.operand3(), 400);
    }
}
