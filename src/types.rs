#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMArrayType, LLVMBuildCall2, LLVMBuildExtractValue, LLVMFunctionType, LLVMGetParam,
        LLVMInt32Type, LLVMInt64Type, LLVMInt8Type, LLVMPointerType, LLVMStructType, LLVMVoidType,
    },
    LLVMBuilder, LLVMType, LLVMValue,
};

use crate::{Function, Value};

pub trait FunctionType {
    type Params;
    type Return;

    fn function_type() -> *mut LLVMType;

    fn function_params(function: *mut LLVMValue) -> Self::Params;

    fn build_call(
        builder: *mut LLVMBuilder,
        function: *mut LLVMValue,
        params: Self::Params,
    ) -> Self::Return;
}

pub trait ValueType {
    type ReturnType;

    fn value_type() -> *mut LLVMType;
    fn as_return_value(builder: *mut LLVMBuilder, value: *mut LLVMValue) -> Self::ReturnType;
}

macro_rules! value_type {
    ($t:ty => $e:expr) => {
        impl ValueType for $t {
            type ReturnType = Value<$t>;

            fn value_type() -> *mut LLVMType {
                unsafe { $e }
            }

            fn as_return_value(_: *mut LLVMBuilder, value: *mut LLVMValue) -> Self::ReturnType {
                Value::new(value)
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

    fn as_return_value(_: *mut LLVMBuilder, _: *mut LLVMValue) -> Self::ReturnType {}
}

impl<T: ValueType> ValueType for *mut T {
    type ReturnType = Value<*mut T>;

    fn value_type() -> *mut LLVMType {
        unsafe { LLVMPointerType(T::value_type(), 0) }
    }

    fn as_return_value(_: *mut LLVMBuilder, value: *mut LLVMValue) -> Self::ReturnType {
        Value::new(value)
    }
}

impl<T: ValueType, const N: usize> ValueType for [T; N] {
    type ReturnType = Value<[T; N]>;

    fn value_type() -> *mut LLVMType {
        unsafe { LLVMArrayType(T::value_type(), N as u32) }
    }

    fn as_return_value(_: *mut LLVMBuilder, value: *mut LLVMValue) -> Self::ReturnType {
        Value::new(value)
    }
}

impl<R: ValueType> ValueType for fn() -> R {
    type ReturnType = Function<fn() -> R>;

    fn value_type() -> *mut LLVMType {
        unsafe { LLVMPointerType(<fn() -> R as FunctionType>::function_type(), 0) }
    }

    fn as_return_value(_: *mut LLVMBuilder, value: *mut LLVMValue) -> Self::ReturnType {
        Function::new(value)
    }
}

impl<A: ValueType, B: ValueType> ValueType for (A, B) {
    type ReturnType = (A::ReturnType, B::ReturnType);

    fn value_type() -> *mut LLVMType {
        unsafe {
            let mut types = [A::value_type(), B::value_type()];
            LLVMStructType(types.as_mut_ptr(), 2, 0)
        }
    }

    fn as_return_value(builder: *mut LLVMBuilder, value: *mut LLVMValue) -> Self::ReturnType {
        let first = unsafe {
            let name = CString::new("").unwrap();

            LLVMBuildExtractValue(
                builder,
                value,
                0,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            )
        };

        let second = unsafe {
            let name = CString::new("").unwrap();

            LLVMBuildExtractValue(
                builder,
                value,
                1,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
            )
        };

        (
            A::as_return_value(builder, first),
            B::as_return_value(builder, second),
        )
    }
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

    fn function_params(_: *mut LLVMValue) -> Self::Params {}

    fn build_call(
        builder: *mut LLVMBuilder,
        function: *mut LLVMValue,
        _: Self::Params,
    ) -> Self::Return {
        R::as_return_value(
            builder,
            build_call(builder, function, Self::function_type(), &[]),
        )
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

    fn function_params(function: *mut LLVMValue) -> Self::Params {
        unsafe { (Value::new(LLVMGetParam(function, 0)),) }
    }

    fn build_call(
        builder: *mut LLVMBuilder,
        function: *mut LLVMValue,
        params: Self::Params,
    ) -> Self::Return {
        R::as_return_value(
            builder,
            build_call(
                builder,
                function,
                Self::function_type(),
                &[params.0.value()],
            ),
        )
    }
}
