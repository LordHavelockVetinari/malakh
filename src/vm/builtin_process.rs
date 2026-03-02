use std::alloc::{self, Layout};
use std::fmt::{self, Debug};
use std::ptr::{self, NonNull};

use crate::vm::gc::{GarbageCollector, GcInfo};
use crate::vm::process::ProcessState;
use crate::vm::{Value, Vm};

pub struct BuiltinProcessFamily {
    pub layout: Layout,
    pub init: unsafe fn(process: BuiltinProcessRef, parent: Option<BuiltinProcessRef>, vm: &mut Vm),
    pub deinit: unsafe fn(process: BuiltinProcessRef),
    pub enter: unsafe fn(
        process: BuiltinProcessRef,
        vm: &mut Vm,
        input: Option<Value>,
    ) -> BuiltinProcessRef,
    pub gc_mark_content: unsafe fn(process: BuiltinProcessRef, gc: &mut GarbageCollector),
    pub name: &'static str,
}

#[repr(C, align(8))]
pub struct BuiltinProcessHeader {
    gc_info: GcInfo,
    state: ProcessState,
    family: &'static BuiltinProcessFamily,
    output_slot: Value,
}

#[derive(Clone, Copy)]
pub struct BuiltinProcessRef(pub NonNull<BuiltinProcessHeader>);

pub trait BuiltinProcessData: Sized {
    const NAME: &str;

    unsafe fn init(process: BuiltinProcessRef, parent: Option<BuiltinProcessRef>, vm: &mut Vm);

    unsafe fn deinit(mut process: BuiltinProcessRef) {
        unsafe {
            ptr::drop_in_place(process.data_ptr::<Self>().as_ptr());
        }
    }

    unsafe fn enter(
        process: BuiltinProcessRef,
        vm: &mut Vm,
        input: Option<Value>,
    ) -> BuiltinProcessRef;

    unsafe fn gc_mark_content(process: BuiltinProcessRef, gc: &mut GarbageCollector);
}

impl BuiltinProcessFamily {
    pub fn from_type<T: BuiltinProcessData>() -> Self {
        let layout = const {
            let Ok((layout, offset)) =
                Layout::new::<BuiltinProcessHeader>().extend(Layout::new::<T>())
            else {
                panic!("layout error");
            };
            assert!(
                offset == size_of::<BuiltinProcessHeader>(),
                "data alignment is too large"
            );
            layout
        };
        Self {
            layout,
            init: T::init,
            deinit: T::deinit,
            enter: T::enter,
            gc_mark_content: T::gc_mark_content,
            name: T::NAME,
        }
    }
}

impl Debug for BuiltinProcessFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<builtin process family {}>", self.name)
    }
}

impl BuiltinProcessRef {
    pub fn new(
        family: &'static BuiltinProcessFamily,
        parent: Option<BuiltinProcessRef>,
        vm: &mut Vm,
    ) -> Self {
        let this = Self(
            NonNull::new(unsafe { alloc::alloc(family.layout) })
                .unwrap()
                .cast::<BuiltinProcessHeader>(),
        );
        unsafe {
            this.0.write(BuiltinProcessHeader {
                gc_info: GcInfo::default(),
                state: ProcessState::Run,
                family,
                output_slot: Value::default(),
            });
            (family.init)(this, parent, vm);
        }
        vm.gc
            .start_tracking(Value::from_builtin_process_ref(this), family.layout.size());
        this
    }

    fn header(&self) -> &BuiltinProcessHeader {
        unsafe { self.0.as_ref() }
    }

    fn header_mut(&mut self) -> &mut BuiltinProcessHeader {
        unsafe { self.0.as_mut() }
    }

    pub fn family(&self) -> &'static BuiltinProcessFamily {
        self.header().family
    }

    pub fn type_name(&self) -> &'static str {
        self.family().name
    }

    pub fn output_slot(&self) -> Value {
        self.header().output_slot
    }

    pub fn output_slot_mut(&mut self) -> &mut Value {
        &mut self.header_mut().output_slot
    }

    pub fn state(&self) -> ProcessState {
        self.header().state
    }

    pub fn state_mut(&mut self) -> &mut ProcessState {
        &mut self.header_mut().state
    }

    pub fn data_ptr<T>(&mut self) -> NonNull<T> {
        unsafe { self.0.add(1).cast::<T>() }
    }

    pub unsafe fn data<T>(&self) -> &T {
        unsafe { self.0.add(1).cast::<T>().as_ref() }
    }

    pub unsafe fn data_mut<T>(&mut self) -> &mut T {
        unsafe { self.data_ptr().as_mut() }
    }

    pub fn gc_mark(&self, gc: &mut GarbageCollector) {
        if self.header().gc_info.mark() {
            return;
        }
        gc.mark(self.output_slot());
        unsafe {
            (self.family().gc_mark_content)(*self, gc);
        }
    }

    // True if the value is still alive.
    pub fn gc_sweep(&mut self) -> bool {
        if self.header().gc_info.unmark() {
            return true;
        }
        let family = self.family();
        unsafe {
            (family.deinit)(*self);
            alloc::dealloc(self.0.as_ptr().cast::<u8>(), family.layout);
        }
        false
    }
}

impl From<BuiltinProcessRef> for Value {
    fn from(process: BuiltinProcessRef) -> Self {
        Value::from_builtin_process_ref(process)
    }
}
