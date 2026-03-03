#![deprecated]
#![allow(unused)]
#![allow(deprecated)]

use malachite::Integer;

use super::*;
use crate::vm::macros::const_code;

pub fn example1() -> Vm {
    let mut builder = Vm::builder();
    builder.int_constant(Integer::from(0));
    builder.int_constant(Integer::from(1));
    builder.initial_process_family(UserProcessFamily {
        code: const_code! {
            CONST 0, 1;
            CONST 1, 1;
            ADD 0, 0, 1;
            DEBUG 0, 0, 0;
            CONST 2, 1;
            ADD 1, 0, 2;
            DEBUG 1, 0, 0;
            EXIT 0, 0, 0;
        },
        memory_len: 3,
        try_bodies: &[],
    });
    builder.build()
}

pub fn example2() -> Vm {
    let mut builder = Vm::builder();
    builder.int_constant(Integer::from(0));
    builder.int_constant(Integer::from(1));
    builder.int_constant(Integer::from(2));
    builder.int_constant(Integer::from(3));
    builder.process_family(UserProcessFamily {
        code: const_code! {
            NEW 0, 1;
            CONST 1, 2;
            SEND 0, 0, 1;
            CONST 1, 3;
            SEND 0, 0, 1;
            RECEIVE 1, 0, 0;
            DEBUG 0, 0, 0;
            DEBUG 1, 0, 0;
            STOP 0, 0, 0;
        },
        memory_len: 2,
        try_bodies: &[],
    });
    builder.process_family(UserProcessFamily {
        code: const_code! {
            IN 0, 0, 0;
            IN 1, 0, 0;
            ADD 2, 0, 1;
            OUT 2, 0, 0;
            STOP 0, 0, 0;
        },
        memory_len: 3,
        try_bodies: &[],
    });
    builder.initial_process_family(UserProcessFamily {
        code: const_code! {
            NEW 0, 0, 0;
            EXIT 0, 0, 0;
        },
        memory_len: 1,
        try_bodies: &[],
    });
    builder.build()
}

pub fn example_fibonacci() -> Vm {
    let mut builder = Vm::builder();
    builder.int_constant(Integer::from(0));
    builder.int_constant(Integer::from(1));
    builder.initial_process_family(UserProcessFamily {
        code: const_code! {
            NEW 0, 1;
            CONST 1, 0;
            SEND 0, 0, 1;
            CONST 1, 1;
            SEND 0, 0, 1;
            EXIT 0, 0, 0;
        },
        memory_len: 2,
        try_bodies: &[],
    });
    builder.process_family(UserProcessFamily {
        code: const_code! {
            IN 0, 0, 0;
            IN 1, 0, 0;
            DEBUG 0, 0, 0;
            ADD 2, 0, 1;
            NEW 0, 1;
            SEND 0, 0, 1;
            SEND 0, 0, 2;
            EXIT 0, 0, 0;
        },
        memory_len: 3,
        try_bodies: &[],
    });
    builder.build()
}

pub fn example_countdown() -> Vm {
    let mut builder = Vm::builder();
    builder.int_constant(Integer::from(0));
    builder.int_constant(Integer::from(-1));
    builder.int_constant(Integer::from(10));
    builder.initial_process_family(UserProcessFamily {
        code: const_code! {
            CONST 0, 2;
            DEBUG 0, 0, 0;
            CONST 1, 1;
            ADD 0, 0, 1;
            CONST 1, 0;
            EQUALS 1, 0, 1;
            JUMP_UNLESS 1, -6i32 as u32;
            EXIT 0, 0, 0;
        },
        memory_len: 2,
        try_bodies: &[],
    });
    builder.build()
}

pub fn example_sum() -> Vm {
    let mut builder = Vm::builder();
    builder.int_constant(Integer::from(0));
    builder.int_constant(Integer::from(1));
    builder.int_constant(Integer::from(2));
    builder.initial_process_family(UserProcessFamily {
        code: const_code! {
            NEW 0, 1;
            CONST 1, 2;
            SEND 0, 0, 1;
            SEND 0, 0, 1;
            SEND 0, 0, 1;
            CONST 1, 1;
            SEND 0, 0, 1;
            NO_IN 0, 0, 0;
            RECEIVE 0, 0, 0;
            DEBUG 0, 0, 0;
            EXIT 0, 0, 0;
        },
        memory_len: 2,
        try_bodies: &[],
    });
    builder.process_family(UserProcessFamily {
        code: const_code! {
            CONST 0, 0, 0;
            OPT_IN 1, 2, 0;
            JUMP_UNLESS 2, 2;
            ADD 0, 0, 1;
            JUMP 0, -4i32 as u32;
            OUT 0, 0, 0;
            STOP 0, 0, 0;
        },
        memory_len: 10,
        try_bodies: &[],
    });
    builder.build()
}

pub fn example_stack() -> Vm {
    let mut builder = Vm::builder();
    builder.int_constant(Integer::from(0));
    builder.int_constant(Integer::from(1));
    builder.int_constant(Integer::from(2));
    builder.int_constant(Integer::from(3));
    builder.process_family(UserProcessFamily {
        // Stack process.
        code: const_code! {
            // Initialize.
            IN 0, 0, 0;
            NEW 1, 0;
            // Begin loop, non-empty.
            OPT_IN 2, 3, 0;
            JUMP_UNLESS 3, 3;
            // Got input.
            SEND 1, 1, 0;
            MOVE 0, 2, 0;
            JUMP 0, !4;
            // Caller wants output.
            OUT 0, 0, 0;
            NO_IN 1, 0, 0;
            TRY_RECEIVE 0, 2, 1;
            JUMP_IF 2, !7;
            // Need input.
            IN 0, 0, 0;
            JUMP 0, !9;
        },
        memory_len: 4,
        try_bodies: &[],
    });
    builder.initial_process_family(UserProcessFamily {
        code: const_code! {
            NEW 0, 0;
            CONST 1, 1;
            SEND 0, 0, 1;
            CONST 1, 2;
            SEND 0, 0, 1;
            CONST 1, 3;
            SEND 0, 0, 1;
            NO_IN 0, 0, 0;
            RECEIVE 1, 0, 0;
            DEBUG 1, 0, 0;
            NO_IN 0, 0, 0;
            RECEIVE 1, 0, 0;
            DEBUG 1, 0, 0;
            NO_IN 0, 0, 0;
            RECEIVE 1, 0, 0;
            DEBUG 1, 0, 0;
            NO_IN 0, 0, 0;
            TRY_RECEIVE 0, 1, 0;
            DEBUG 1, 0, 0;
            CONST 1, 0;
            SEND 0, 0, 1;
            NO_IN 0, 0, 0;
            TRY_RECEIVE 0, 1, 0;
            DEBUG 0, 0, 0;
            DEBUG 1, 0, 0;
            EXIT 0, 0, 0;
        },
        memory_len: 2,
        try_bodies: &[],
    });
    builder.build()
}
