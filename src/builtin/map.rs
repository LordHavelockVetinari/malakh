mod at;
mod clear;
mod copy;
mod each;
pub mod from_pairs;
mod has_key;
mod keys;
mod length;
pub mod of;
mod update;
mod values;

use hashbrown::HashTable;

use crate::builtin::helper;
use crate::vm::builtin_process::{BuiltinProcessData, BuiltinProcessRef};
use crate::vm::process::{ProcessRef, ProcessState};
use crate::vm::value::hashable::HashableValue;
use crate::vm::{Value, Vm};

pub struct Map {
    data: HashTable<(HashableValue, Value)>,
}

struct Iter {
    index: usize,
}

helper::define_class!(
    CLEAR => self::clear::Clear,
    EACH => self::each::Each,
    LENGTH => self::length::Length,
    AT => self::at::At,
    KEYS => self::keys::Keys,
    VALUES => self::values::Values,
    HAS_KEY => self::has_key::HasKey,
    UPDATE => self::update::Update,
    COPY => self::copy::Copy,
);

type Entry<'a> = hashbrown::hash_table::Entry<'a, (HashableValue, Value)>;
type OccupiedEntry<'a> = hashbrown::hash_table::OccupiedEntry<'a, (HashableValue, Value)>;
type AbsentEntry<'a> = hashbrown::hash_table::AbsentEntry<'a, (HashableValue, Value)>;

impl Map {
    fn find_entry(&mut self, key: HashableValue) -> Result<OccupiedEntry<'_>, AbsentEntry<'_>> {
        self.data.find_entry(key.hash(), |&(k, _)| k == key)
    }

    fn entry(&mut self, key: HashableValue) -> Entry<'_> {
        self.data
            .entry(key.hash(), |&(k, _)| k == key, |&(k, _)| k.hash())
    }

    fn iter(&self) -> Iter {
        Iter { index: 0 }
    }
}

impl Iter {
    fn next(&mut self, map: &Map) -> Option<(HashableValue, Value)> {
        while self.index < map.data.num_buckets() {
            let i = self.index;
            self.index += 1;
            if let Some(&(key, value)) = map.data.get_bucket(i) {
                return Some((key, value));
            }
        }
        None
    }
}

impl BuiltinProcessData for Map {
    const NAME: &str = "Map";

    unsafe fn init(
        mut process: BuiltinProcessRef,
        parent: Option<BuiltinProcessRef>,
        _vm: &mut Vm,
    ) {
        debug_assert!(parent.is_none());
        unsafe {
            process.data_ptr::<Self>().write(Self {
                data: HashTable::new(),
            });
        }
        *process.state_mut() = ProcessState::ForkIn;
    }

    unsafe fn enter(
        process: BuiltinProcessRef,
        vm: &mut Vm,
        input: Option<Value>,
    ) -> BuiltinProcessRef {
        let input = input.expect("List process didn't get input");
        let Some(cmd) = input.as_symbol() else {
            let index = *methods::AT.get().expect("method .At uninitialized");
            let family = vm.get_builtin_family(index);
            vm.put_temporary1(input);
            return BuiltinProcessRef::new(family, Some(process), vm);
        };
        let Some(index) = symbol_to_method_index(cmd) else {
            todo!("invalid map method");
        };
        let family = vm.get_builtin_family(index);
        #[cfg(debug_assertions)]
        vm.assert_temporary1_none();
        BuiltinProcessRef::new(family, Some(process), vm)
    }

    unsafe fn gc_mark_content(
        process: BuiltinProcessRef,
        gc: &mut crate::vm::gc::GarbageCollector,
    ) {
        let this = unsafe { process.data::<Self>() };
        for &(key, value) in &this.data {
            gc.mark(key.get());
            gc.mark(value);
        }
    }
}
