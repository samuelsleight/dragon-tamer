use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildCondBr, LLVMBuildGEP2, LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildRet,
        LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub, LLVMCreateBuilder, LLVMDisposeBuilder,
    },
    LLVMBuilder, LLVMIntPredicate,
};

use crate::{
    jump_table::JumpTable, value::Integer, Block, Function, FunctionType, Value, ValueType,
};

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

    pub fn build_load<T: ValueType>(&self, ptr: &Value<*mut T>) -> Value<T> {
        unsafe {
            let name = CString::new("").unwrap();
            Value::new(LLVMBuildLoad2(
                self.builder,
                T::value_type(),
                ptr.value(),
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            ))
        }
    }

    pub fn build_store<T: ValueType>(&self, ptr: &Value<*mut T>, value: &Value<T>) {
        unsafe {
            LLVMBuildStore(self.builder, value.value(), ptr.value());
        }
    }

    pub fn build_index_load<T: ValueType, const N: usize, I: Integer>(
        &self,
        array: &Value<*mut [T; N]>,
        index: &Value<I>,
    ) -> Value<T> {
        let ep = unsafe {
            let name = CString::new("").unwrap();
            let mut indices = [I::zero(), index.value()];

            LLVMBuildGEP2(
                self.builder,
                <[T; N] as ValueType>::value_type(),
                array.value(),
                indices.as_mut_ptr(),
                2,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            )
        };

        unsafe {
            let name = CString::new("").unwrap();
            Value::new(LLVMBuildLoad2(
                self.builder,
                T::value_type(),
                ep,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            ))
        }
    }

    pub fn build_index_store<T: ValueType, const N: usize, I: Integer>(
        &self,
        array: &Value<*mut [T; N]>,
        index: &Value<I>,
        value: &Value<T>,
    ) {
        let ep = unsafe {
            let name = CString::new("").unwrap();
            let mut indices = [I::zero(), index.value()];

            LLVMBuildGEP2(
                self.builder,
                <[T; N] as ValueType>::value_type(),
                array.value(),
                indices.as_mut_ptr(),
                2,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            )
        };

        unsafe {
            LLVMBuildStore(self.builder, value.value(), ep);
        }
    }

    pub fn build_jump_table<'a, 'b, T: ValueType>(
    pub fn build_jump_table<T: ValueType>(
        &self,
        switch: &Value<T>,
        default: &Block,
    ) -> JumpTable<T> {
        JumpTable::new(self.builder, switch.value(), default)
    }

    pub fn build_ret<T: ValueType>(&self, value: &Value<T>) {
        unsafe {
            LLVMBuildRet(self.builder, value.value());
        }
    }

    pub fn build_void_ret(&self) {
        unsafe {
            LLVMBuildRetVoid(self.builder);
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
