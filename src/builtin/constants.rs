use std::f64::consts;

use crate::vm::builder::VmBuilder;

pub fn infinity(builder: &mut VmBuilder) -> u32 {
    builder
        .float_constant(f64::INFINITY)
        .expect("failed to create constant")
}

pub fn nan(builder: &mut VmBuilder) -> u32 {
    builder
        .float_constant(f64::NAN)
        .expect("failed to create constant")
}

pub fn pi(builder: &mut VmBuilder) -> u32 {
    builder
        .float_constant(consts::PI)
        .expect("failed to create constant")
}

pub fn e(builder: &mut VmBuilder) -> u32 {
    builder
        .float_constant(consts::E)
        .expect("failed to create constant")
}
