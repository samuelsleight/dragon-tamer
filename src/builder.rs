use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildRet, LLVMBuildSub,
        LLVMCreateBuilder, LLVMDisposeBuilder,
    },
    LLVMBuilder, LLVMIntPredicate,
};

use crate::{value::Integer, Block, Function, FunctionType, Value, ValueType};

pub struct Builder {
    builder: *mut LLVMBuilder,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        let builder = unsafe { LLVMCreateBuilder() };

        Self { builder }
    }

    pub fn set_block(&self, block: &Block) {
        block.set_to_builder(self.builder);
    }

    pub fn build_call<T: FunctionType>(
        &self,
        function: &Function<T>,
        params: T::Params,
    ) -> T::Return {
        function.build_call(self.builder, params)
    }

    pub fn build_conditional_jump<T: Integer>(&self, value: &Value<T>, t: &Block, f: &Block) {
        unsafe {
            let zero = T::zero();
            let name = CString::new("").unwrap();
            let cmp = LLVMBuildICmp(
                self.builder,
                LLVMIntPredicate::LLVMIntEQ,
                zero,
                value.value(),
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            );
            LLVMBuildCondBr(self.builder, cmp, t.value(), f.value());
        }
    }

    pub fn build_add<T: Integer>(&self, lhs: &Value<T>, rhs: &Value<T>) -> Value<T> {
        unsafe {
            let name = CString::new("").unwrap();
            Value::new(LLVMBuildAdd(
                self.builder,
                lhs.value(),
                rhs.value(),
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            ))
        }
    }

    pub fn build_sub<T: Integer>(&self, lhs: &Value<T>, rhs: &Value<T>) -> Value<T> {
        unsafe {
            let name = CString::new("").unwrap();
            Value::new(LLVMBuildSub(
                self.builder,
                lhs.value(),
                rhs.value(),
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            ))
        }
    }

    pub fn build_ret(&self) {
    pub fn build_ret<T: ValueType>(&self, value: &Value<T>) {
        unsafe {
            LLVMBuildRet(self.builder, value.value());
        }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}
