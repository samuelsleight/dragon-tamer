use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMBuildCall2, LLVMFunctionType, LLVMInt32Type, LLVMInt8Type, LLVMPointerType,
        LLVMVoidType,
    },
    LLVMBuilder, LLVMType, LLVMValue,
};

use crate::Value;

pub trait FunctionType {
    type Params;
    type Return;

    fn function_type() -> *mut LLVMType;
    fn build_call(
        builder: *mut LLVMBuilder,
        function: *mut LLVMValue,
        params: Self::Params,
    ) -> Self::Return;
}

pub trait ValueType {
    type ReturnType;

    fn value_type() -> *mut LLVMType;
    fn as_return_value(value: *mut LLVMValue) -> Self::ReturnType;
}

macro_rules! value_type {
    ($t:ty => $e:expr) => {
        impl ValueType for $t {
            type ReturnType = Value<$t>;

            fn value_type() -> *mut LLVMType {
                unsafe { $e }
            }

            fn as_return_value(value: *mut LLVMValue) -> Self::ReturnType {
                Value::<Self>::new(value)
            }
        }
    };
}

value_type!(i32 => LLVMInt32Type());
value_type!(i64 => LLVMInt64Type());
value_type!(u32 => LLVMInt32Type());
value_type!(u64 => LLVMInt32Type());
value_type!(String => LLVMPointerType(LLVMInt8Type(), 0));

impl ValueType for () {
    type ReturnType = ();

    fn value_type() -> *mut LLVMType {
        unsafe { LLVMVoidType() }
    }

    fn as_return_value(_value: *mut LLVMValue) -> Self::ReturnType {}
}

fn function_type(ret: *mut LLVMType, params: &[*mut LLVMType]) -> *mut LLVMType {
    unsafe { LLVMFunctionType(ret, params.as_ptr() as *mut _, params.len() as u32, 0) }
}

fn build_call(
    builder: *mut LLVMBuilder,
    function: *mut LLVMValue,
    function_type: *mut LLVMType,
    params: &[*mut LLVMValue],
) -> *mut LLVMValue {
    unsafe {
        let name = CString::new("").unwrap();
        LLVMBuildCall2(
            builder,
            function_type,
            function,
            params.as_ptr() as *mut _,
            params.len() as u32,
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        )
    }
}

impl<R> FunctionType for fn() -> R
where
    R: ValueType,
{
    type Params = ();
    type Return = R::ReturnType;

    fn function_type() -> *mut LLVMType {
        function_type(R::value_type(), &[])
    }

    fn build_call(
        builder: *mut LLVMBuilder,
        function: *mut LLVMValue,
        _: Self::Params,
    ) -> Self::Return {
        R::as_return_value(build_call(builder, function, Self::function_type(), &[]))
    }
}

impl<T, R> FunctionType for fn(T) -> R
where
    R: ValueType,
    T: ValueType,
{
    type Params = (Value<T>,);
    type Return = R::ReturnType;

    fn function_type() -> *mut LLVMType {
        function_type(R::value_type(), &[T::value_type()])
    }

    fn build_call(
        builder: *mut LLVMBuilder,
        function: *mut LLVMValue,
        params: Self::Params,
    ) -> Self::Return {
        R::as_return_value(build_call(
            builder,
            function,
            Self::function_type(),
            &[params.0.value()],
        ))
    }
}
