use std::marker::PhantomData;

use llvm_sys::{core::LLVMAppendBasicBlock, LLVMBuilder, LLVMValue};

use crate::{value::UntypedValue, Block, FunctionType, Value, ValueType, VariadicFunctionType};

use std::ffi::CString;

#[derive(Copy, Clone)]
pub struct Function<T: FunctionType> {
    value: *mut LLVMValue,
    phantom: PhantomData<T>,
}

impl<T: FunctionType> Function<T> {
    pub(crate) fn new(value: *mut LLVMValue) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }

    pub(crate) fn build_call(&self, builder: *mut LLVMBuilder, params: T::Params) -> T::Return {
        T::build_call(builder, self.value, params)
    }

    pub(crate) fn build_variadic_call(
        &self,
        builder: *mut LLVMBuilder,
        params: T::Params,
        variadic_params: &[UntypedValue],
    ) -> T::Return
    where
        T: VariadicFunctionType,
    {
        T::build_variadic_call(builder, self.value, params, variadic_params)
    }

    pub fn params(&self) -> T::Params {
        T::function_params(self.value)
    }

    pub fn add_block<S: AsRef<str>>(&self, name: S) -> Block {
        let name = CString::new(name.as_ref()).unwrap();

        let block = unsafe {
            LLVMAppendBasicBlock(self.value, name.to_bytes_with_nul().as_ptr().cast::<i8>())
        };

        Block::new(block)
    }

    pub fn as_value(&self) -> Value<T>
    where
        T: ValueType,
    {
        Value::new(self.value)
    }
}
