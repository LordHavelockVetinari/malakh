use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{LazyLock, RwLock};
use std::{mem, ptr};

#[repr(align(8))]
pub struct Symbol {
    name: &'static str,
}

impl Symbol {
    pub fn get_global(name: &str) -> &'static Symbol {
        // We have to lie to the compiler that name is 'static so that
        // HashMap::get will work.
        let evil_name = unsafe { mem::transmute::<&str, &'static str>(name) };
        let copy = Symbol { name: evil_name };
        if let Some(symbol) = SYMBOL_TABLE.read().unwrap().get(&copy.name) {
            return symbol;
        }
        let copy = Box::leak(Box::new(Symbol {
            name: copy.name.to_string().leak(),
        }));
        SYMBOL_TABLE.write().unwrap().insert(copy.name, copy);
        copy
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

fn hash_map_from_symbols<'a>(symbols: &[&'a Symbol]) -> HashMap<&'a str, &'a Symbol> {
    let mut result = HashMap::with_capacity(symbols.len());
    for &sym in symbols {
        let Entry::Vacant(entry) = result.entry(sym.name) else {
            panic!("duplicate symbol definition: {}", sym.name)
        };
        entry.insert(sym);
    }
    result
}

macro_rules! builtin_symbols {
    ($($constant:ident = .$name:ident),* $(,)?) => {
        $(
            static $constant: Symbol = Symbol { name: stringify!($name) };

            impl Symbol {
                pub const $constant: &Symbol = &$constant;
            }
        )*

        static SYMBOL_TABLE: LazyLock<RwLock<HashMap<&str, &Symbol>>> = LazyLock::new(|| {
            RwLock::new(hash_map_from_symbols(&[$(
                Symbol::$constant,
            )*]))
        });
    }
}

builtin_symbols!(
    TRUE = .True,
    FALSE = .False,
    UNDEFINED = .Undefined,
    RUN = .Run,
    STOP = .Stop,
    OUT = .Out,
    IN = .In,
    OPT_IN = .OptIn,
    FORK_IN = .ForkIn,
    FROM = .From,
    TO = .To,
    THROUGH = .Through,
    STEP = .Step,
    LENGTH = .Length,
    PUSH = .Push,
    EACH = .Each,
    SET = .Set,
    INSERT = .Insert,
    REMOVE = .Remove,
    CLEAR = .Clear,
    IS = .Is,
    AT = .At,
    KEYS = .Keys,
    VALUES = .Values,
    HAS_KEY = .HasKey,
    UPDATE = .Update,
    COPY = .Copy,
);
