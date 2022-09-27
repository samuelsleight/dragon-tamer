use crate::ValueType;

use std::marker::PhantomData;

use llvm_sys::{core::LLVMConstInt, LLVMValue};

pub trait Constant: ValueType {
    fn constant(self) -> *mut LLVMValue;
}

impl Constant for i32 {
    fn constant(self) -> *mut LLVMValue {
        unsafe { LLVMConstInt(Self::value_type(), self as u64, 0) }
    }
}

#[derive(Clone)]
pub struct Value<T: ValueType> {
    value: *mut LLVMValue,
    phantom: PhantomData<T>,
}

impl<T: ValueType> Value<T> {
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

    pub(crate) fn value(&self) -> *mut LLVMValue {
        self.value
    }
}
