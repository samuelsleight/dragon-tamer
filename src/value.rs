use crate::ValueType;

use std::marker::PhantomData;

use llvm_sys::{core::LLVMConstInt, LLVMValue};

pub trait Constant: ValueType + Sized {
    fn constant(self) -> *mut LLVMValue;
}

pub trait Integer: Constant {
    fn zero() -> *mut LLVMValue;
}

macro_rules! constant {
    ($t:ty) => {
        impl Constant for $t {
            fn constant(self) -> *mut LLVMValue {
                unsafe { LLVMConstInt(Self::value_type(), self as u64, 0) }
            }
        }

        impl Integer for $t {
            fn zero() -> *mut LLVMValue {
                (0 as Self).constant()
            }
        }
    };
}

constant!(i32);
constant!(i64);
constant!(u32);
constant!(u64);

#[derive(Copy, Clone)]
pub struct UntypedValue {
    value: *mut LLVMValue,
}

impl UntypedValue {
    pub(crate) fn value(&self) -> *mut LLVMValue {
        self.value
    }
}

#[derive(Copy, Clone)]
pub struct Value<T: ValueType + ?Sized> {
    value: *mut LLVMValue,
    phantom: PhantomData<T>,
}

impl<T: ValueType + ?Sized> Value<T> {
    pub fn new(value: *mut LLVMValue) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }

    pub fn constant(t: T) -> Self
    where
        T: Constant,
    {
        Self {
            value: t.constant(),
            phantom: PhantomData,
        }
    }

    pub fn untyped(&self) -> UntypedValue {
        UntypedValue { value: self.value }
    }

    pub(crate) fn value(&self) -> *mut LLVMValue {
        self.value
    }
}
