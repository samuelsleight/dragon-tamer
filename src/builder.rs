use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildCondBr, LLVMBuildICmp, LLVMBuildRet, LLVMBuildSub,
        LLVMCreateBuilder, LLVMDisposeBuilder,
    },
    LLVMBuilder, LLVMIntPredicate,
};

use crate::{Block, Function, FunctionType, Value};

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

    pub fn build_conditional_jump(&self, value: &Value<i32>, t: &Block, f: &Block) {
        unsafe {
            let zero = Value::constant(0_i32);
            let name = CString::new("").unwrap();
            let cmp = LLVMBuildICmp(
                self.builder,
                LLVMIntPredicate::LLVMIntEQ,
                zero.value(),
                value.value(),
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            );
            LLVMBuildCondBr(self.builder, cmp, t.value(), f.value());
        }
    }

    pub fn build_add(&self, lhs: &Value<i32>, rhs: &Value<i32>) -> Value<i32> {
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

    pub fn build_sub(&self, lhs: &Value<i32>, rhs: &Value<i32>) -> Value<i32> {
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
        unsafe {
            LLVMBuildRet(self.builder, Value::<i32>::constant(0).value());
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
