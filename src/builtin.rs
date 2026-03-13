use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::LazyLock;

use crate::builtin::helper::{AsBasicAggregator, AsFunction};
use crate::vm::builder::VmBuilder;
use crate::vm::builtin_process::{BuiltinProcessData, BuiltinProcessFamily};

mod assert;
mod assert_err;
mod assert_stop;
mod cat;
mod constants;
mod helper;
mod is_bool;
mod is_float;
mod is_int;
mod is_number;
mod is_process;
mod is_string;
mod is_symbol;
mod list;
mod map;
mod math;
mod max;
mod min;
mod process;
mod queue;
mod random;
mod range;
pub mod runtime_error;
mod stack;
mod string;
mod sum;
mod to_float;
mod to_int;
mod to_number;
mod user;

pub type ConstantConstructor = fn(&mut VmBuilder) -> u32;

pub struct BuiltinModule {
    pub constructor_definitions: HashMap<&'static str, u32>,
    pub constant_definitions: HashMap<&'static str, u32>,
    pub sub_modules: HashMap<&'static str, BuiltinModule>,
}

struct BuiltinCollector {
    builtin_constructors: Vec<&'static BuiltinProcessFamily>,
    builtin_constants: Vec<ConstantConstructor>,
    root_module: BuiltinModule,
}

const NO_PATH: &str = "?";
const ROOT_PATH: &str = "";

impl BuiltinModule {
    fn new() -> Self {
        Self {
            constructor_definitions: HashMap::new(),
            constant_definitions: HashMap::new(),
            sub_modules: HashMap::new(),
        }
    }

    fn add_sub_module(&mut self, name: &'static str) -> &mut Self {
        let Entry::Vacant(entry) = self.sub_modules.entry(name) else {
            panic!("sub-module {} already present", name);
        };
        entry.insert(BuiltinModule::new())
    }

    fn add_constructor(&mut self, name: &'static str, index: u32) {
        debug_assert!(!self.constant_definitions.contains_key(name));
        let Entry::Vacant(entry) = self.constructor_definitions.entry(name) else {
            panic!("definition {} already present", name);
        };
        entry.insert(index);
    }

    fn add_constant(&mut self, name: &'static str, offset: u32) {
        debug_assert!(!self.constructor_definitions.contains_key(name));
        let Entry::Vacant(entry) = self.constant_definitions.entry(name) else {
            panic!("definition {} already present", name);
        };
        entry.insert(offset);
    }

    fn walk_to_path_mut(&mut self, path: &str) -> Option<&mut Self> {
        if path == NO_PATH {
            return None;
        }
        if path.is_empty() {
            return Some(self);
        }
        let mut module = self;
        for part in path.split("::") {
            let Some(sub_module) = module.sub_modules.get_mut(part) else {
                panic!("could not find path {}", path);
            };
            module = sub_module;
        }
        Some(module)
    }
}

impl BuiltinCollector {
    fn new() -> Self {
        Self {
            builtin_constructors: Vec::new(),
            builtin_constants: Vec::new(),
            root_module: BuiltinModule::new(),
        }
    }

    fn add_module_to_root(mut self, name: &'static str) -> Self {
        self.root_module.add_sub_module(name);
        self
    }

    fn add_family(&mut self, family: BuiltinProcessFamily, path: &str) -> u32 {
        let index =
            u32::try_from(self.builtin_constructors.len()).expect("too many builtin processes");
        if let Some(module) = self.root_module.walk_to_path_mut(path) {
            module.add_constructor(family.name, index);
        }
        self.builtin_constructors.push(Box::leak(Box::new(family)));
        index
    }

    fn add_type<T: BuiltinProcessData>(mut self, path: &str) -> Self {
        self.add_family(BuiltinProcessFamily::from_type::<T>(), path);
        self
    }

    fn add_type_init<T: BuiltinProcessData>(
        mut self,
        path: &str,
        init: impl FnOnce(&mut Self, u32),
    ) -> Self {
        let index = self.add_family(BuiltinProcessFamily::from_type::<T>(), path);
        init(&mut self, index);
        self
    }

    fn add_constant(
        mut self,
        path: &str,
        name: &'static str,
        constructor: ConstantConstructor,
    ) -> Self {
        let offset =
            u32::try_from(self.builtin_constants.len()).expect("too many builtin constants");
        self.builtin_constants.push(constructor);
        if let Some(module) = self.root_module.walk_to_path_mut(path) {
            module.add_constant(name, offset);
        }
        self
    }
}

static GLOBAL_BUILTIN_COLLECTOR: LazyLock<BuiltinCollector> = LazyLock::new(|| {
    BuiltinCollector::new()
        .add_type_init::<runtime_error::RuntimeError>(NO_PATH, runtime_error::init)
        .add_constant(ROOT_PATH, "Infinity", constants::infinity)
        .add_constant(ROOT_PATH, "NaN", constants::nan)
        .add_type::<AsFunction<user::User>>(ROOT_PATH)
        .add_type::<AsBasicAggregator<cat::Cat>>(ROOT_PATH)
        .add_type::<AsFunction<sum::Sum>>(ROOT_PATH)
        .add_type::<AsFunction<min::Min>>(ROOT_PATH)
        .add_type::<AsFunction<max::Max>>(ROOT_PATH)
        .add_type::<AsBasicAggregator<stack::Stack>>(ROOT_PATH)
        .add_type::<AsBasicAggregator<queue::Queue>>(ROOT_PATH)
        .add_type::<AsFunction<is_int::IsInt>>(ROOT_PATH)
        .add_type::<AsFunction<is_float::IsFloat>>(ROOT_PATH)
        .add_type::<AsFunction<is_number::IsNumber>>(ROOT_PATH)
        .add_type::<AsFunction<is_string::IsString>>(ROOT_PATH)
        .add_type::<AsFunction<is_symbol::IsSymbol>>(ROOT_PATH)
        .add_type::<AsFunction<is_bool::IsBool>>(ROOT_PATH)
        .add_type::<AsFunction<is_process::IsProcess>>(ROOT_PATH)
        .add_type::<AsFunction<to_int::ToInt>>(ROOT_PATH)
        .add_type::<AsFunction<to_float::ToFloat>>(ROOT_PATH)
        .add_type::<AsFunction<to_number::ToNumber>>(ROOT_PATH)
        .add_type::<AsFunction<range::Range>>(ROOT_PATH)
        .add_type::<AsFunction<assert::Assert>>(ROOT_PATH)
        .add_type::<AsFunction<assert_err::AssertErr>>(ROOT_PATH)
        .add_type::<AsFunction<assert_stop::AssertStop>>(ROOT_PATH)
        .add_type_init::<list::List>(ROOT_PATH, list::init)
        .add_module_to_root("List")
        .add_type::<AsFunction<list::of::Of>>("List")
        .add_type_init::<map::Map>(ROOT_PATH, map::init)
        .add_module_to_root("Map")
        .add_type::<AsFunction<map::of::Of>>("Map")
        .add_type::<AsFunction<map::from_pairs::FromPairs>>("Map")
        .add_type_init::<random::Random>(ROOT_PATH, random::init)
        .add_module_to_root("Math")
        .add_constant("Math", "Pi", constants::pi)
        .add_constant("Math", "E", constants::e)
        .add_type::<AsFunction<math::sin::Sin>>("Math")
        .add_type::<AsFunction<math::cos::Cos>>("Math")
        .add_type::<AsFunction<math::tan::Tan>>("Math")
        .add_type::<AsFunction<math::cot::Cot>>("Math")
        .add_type::<AsFunction<math::asin::Asin>>("Math")
        .add_type::<AsFunction<math::acos::Acos>>("Math")
        .add_type::<AsFunction<math::atan::Atan>>("Math")
        .add_type::<AsFunction<math::acot::Acot>>("Math")
        .add_type::<AsFunction<math::atan2::Atan2>>("Math")
        .add_type::<AsFunction<math::sqrt::Sqrt>>("Math")
        .add_type::<AsFunction<math::cbrt::Cbrt>>("Math")
        .add_type::<AsFunction<math::exp::Exp>>("Math")
        .add_type::<AsFunction<math::log::Log>>("Math")
        .add_type::<AsFunction<math::is_finite::IsFinite>>("Math")
        .add_type::<AsFunction<math::is_infinite::IsInfinite>>("Math")
        .add_type::<AsFunction<math::is_nan::IsNaN>>("Math")
        .add_type::<AsFunction<math::product::Product>>("Math")
        .add_type::<AsFunction<math::mean::Mean>>("Math")
        .add_type::<AsFunction<math::norm::Norm>>("Math")
        .add_type::<AsFunction<math::abs::Abs>>("Math")
        .add_type::<AsFunction<math::round::Round>>("Math")
        .add_type::<AsFunction<math::floor::Floor>>("Math")
        .add_type::<AsFunction<math::ceil::Ceil>>("Math")
        .add_type::<AsFunction<math::mod_pow::ModPow>>("Math")
        .add_module_to_root("String")
        .add_type::<AsFunction<string::length::Length>>("String")
        .add_type::<AsFunction<string::bytes::Bytes>>("String")
        .add_type::<AsFunction<string::from_bytes::FromBytes>>("String")
        .add_type::<AsFunction<string::words::Words>>("String")
        .add_type::<AsBasicAggregator<string::from_words::FromWords>>("String")
        .add_type::<AsFunction<string::lines::Lines>>("String")
        .add_type::<AsBasicAggregator<string::from_lines::FromLines>>("String")
        .add_type::<AsFunction<string::slice::Slice>>("String")
        .add_module_to_root("Process")
        .add_type::<AsFunction<process::state::State>>("Process")
        .add_type::<AsFunction<process::peek::Peek>>("Process")
});

pub static BUILTINS: LazyLock<&[&'static BuiltinProcessFamily]> =
    LazyLock::new(|| &GLOBAL_BUILTIN_COLLECTOR.builtin_constructors);

pub static CONSTANTS: LazyLock<&[ConstantConstructor]> =
    LazyLock::new(|| &GLOBAL_BUILTIN_COLLECTOR.builtin_constants);

pub static ROOT_MODULE: LazyLock<&BuiltinModule> =
    LazyLock::new(|| &GLOBAL_BUILTIN_COLLECTOR.root_module);
