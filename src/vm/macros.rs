macro_rules! instruction {
    ($opcode:ident $op1:expr, $op2:expr;) => {
        $crate::vm::Instruction::two_operand($crate::vm::opcode::$opcode, $op1, $op2)
    };
    ($opcode:ident $op1:expr, $op2:expr, $op3:expr;) => {
        $crate::vm::Instruction::three_operand($crate::vm::opcode::$opcode, $op1, $op2, $op3)
    };
}

macro_rules! code {
    ($($opcode:ident $($params:expr),*;)*) => {
        &[
            $(
                $crate::vm::macros::instruction!($opcode $($params),*;)
            ),*
        ]
    };
}

macro_rules! const_code {
    ($($opcode:ident $($params:expr),*;)*) => {
        const {
            $crate::vm::macros::code!($($opcode $($params),*;)*)
        }
    };
}

macro_rules! vec_code {
    ($($opcode:ident $($params:expr),*;)*) => {
        vec![
            $(
                $crate::vm::macros::instruction!($opcode $($params),*;)
            ),*
        ]
    };
}

macro_rules! leak_code {
    ($($t:tt)*) => {
        $crate::vm::macros::vec_code!($($t)*).leak()
    };
}

pub(crate) use {code, const_code, instruction, leak_code, vec_code};
