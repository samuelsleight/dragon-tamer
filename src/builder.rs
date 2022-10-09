use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCondBr, LLVMBuildGEP2, LLVMBuildICmp,
        LLVMBuildIntCast, LLVMBuildLoad2, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildStore,
        LLVMBuildStructGEP2, LLVMBuildSub, LLVMBuildUnreachable, LLVMCreateBuilder,
        LLVMDisposeBuilder,
    },
    LLVMBuilder, LLVMIntPredicate,
};

use crate::{
    jump_table::JumpTable, value::Integer, Block, Function, FunctionType, Value, ValueType,
};

#[must_use]
pub struct Builder {
    pub(crate) builder: *mut LLVMBuilder,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub(crate) fn new() -> Self {
        let builder = unsafe { LLVMCreateBuilder() };

        Self { builder }
    }

    pub fn build_call<T: FunctionType>(
        &self,
        function: &Function<T>,
        params: T::Params,
    ) -> T::Return {
        function.build_call(self.builder, params)
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

    pub fn build_eq<T: Integer>(&self, lhs: &Value<T>, rhs: &Value<T>) -> Value<T> {
        unsafe {
            let name = CString::new("").unwrap();

            let result = LLVMBuildICmp(
                self.builder,
                LLVMIntPredicate::LLVMIntEQ,
                lhs.value(),
                rhs.value(),
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            );

            Value::new(LLVMBuildIntCast(
                self.builder,
                result,
                T::value_type(),
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

    pub fn build_struct<A: ValueType, B: ValueType>(
        &self,
        a: &Value<A>,
        b: &Value<B>,
    ) -> Value<*mut (A, B)> {
        let value = unsafe {
            let name = CString::new("").unwrap();

            LLVMBuildAlloca(
                self.builder,
                <(A, B) as ValueType>::value_type(),
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            )
        };

        let first_ep = unsafe {
            let name = CString::new("").unwrap();

            LLVMBuildStructGEP2(
                self.builder,
                <(A, B) as ValueType>::value_type(),
                value,
                0,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            )
        };

        let second_ep = unsafe {
            let name = CString::new("").unwrap();

            LLVMBuildStructGEP2(
                self.builder,
                <(A, B) as ValueType>::value_type(),
                value,
                1,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            )
        };

        unsafe {
            LLVMBuildStore(self.builder, a.value(), first_ep);
            LLVMBuildStore(self.builder, b.value(), second_ep);
        }

        Value::new(value)
    }

    pub fn build_jump_table<T: ValueType>(
        self,
        switch: &Value<T>,
        default: &Block,
    ) -> JumpTable<T> {
        JumpTable::new(self, switch.value(), default)
    }

    pub fn build_unreachable(self) {
        unsafe {
            LLVMBuildUnreachable(self.builder);
        }
    }

    pub fn build_jump(self, block: &Block) {
        unsafe {
            LLVMBuildBr(self.builder, block.value());
        }
    }

    pub fn build_conditional_jump<T: Integer>(self, value: &Value<T>, t: &Block, f: &Block) {
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

    pub fn build_ret<T: ValueType>(self, value: &Value<T>) {
        unsafe {
            LLVMBuildRet(self.builder, value.value());
        }
    }

    pub fn build_void_ret(self) {
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
