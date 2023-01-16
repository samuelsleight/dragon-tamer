use std::ffi::CString;

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCondBr, LLVMBuildGEP2, LLVMBuildICmp,
        LLVMBuildIntCast, LLVMBuildLoad2, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildSelect,
        LLVMBuildStore, LLVMBuildStructGEP2, LLVMBuildSub, LLVMBuildUnreachable, LLVMCreateBuilder,
        LLVMDisposeBuilder, LLVMInt1Type,
    },
    LLVMBuilder, LLVMIntPredicate,
};

use crate::{
    jump_table::JumpTable,
    value::{Integer, UntypedValue},
    Block, Function, FunctionType, Value, ValueType, VariadicFunctionType,
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
        self,
        function: &Function<T>,
        params: T::Params,
    ) -> (T::Return, Self) {
        (build_call(self.builder, function, params), self)
    }

    pub fn build_variadic_call<T: VariadicFunctionType>(
        self,
        function: &Function<T>,
        params: T::Params,
        variadic_params: &[UntypedValue],
    ) -> (T::Return, Self) {
        (
            build_variadic_call(self.builder, function, params, variadic_params),
            self,
        )
    }

    pub fn build_int_cast<T: Integer, U: Integer>(self, from: &Value<T>) -> (Value<U>, Self) {
        (build_int_cast(self.builder, from), self)
    }

    pub fn build_add<T: Integer>(self, lhs: &Value<T>, rhs: &Value<T>) -> (Value<T>, Self) {
        (build_add(self.builder, lhs, rhs), self)
    }

    pub fn build_sub<T: Integer>(self, lhs: &Value<T>, rhs: &Value<T>) -> (Value<T>, Self) {
        (build_sub(self.builder, lhs, rhs), self)
    }

    pub fn build_eq<T: Integer>(self, lhs: &Value<T>, rhs: &Value<T>) -> (Value<T>, Self) {
        (build_eq(self.builder, lhs, rhs), self)
    }

    pub fn build_lt<T: Integer>(self, lhs: &Value<T>, rhs: &Value<T>) -> (Value<T>, Self) {
        (build_lt(self.builder, lhs, rhs), self)
    }

    pub fn build_conditional_value<T: Integer, U: ValueType>(
        self,
        value: &Value<T>,
        t: &Value<U>,
        f: &Value<U>,
    ) -> (Value<U>, Self) {
        (build_conditional_value(self.builder, value, t, f), self)
    }

    pub fn build_load<T: ValueType>(self, ptr: &Value<*mut T>) -> (Value<T>, Self) {
        (build_load(self.builder, ptr), self)
    }

    pub fn build_store<T: ValueType>(self, ptr: &Value<*mut T>, value: &Value<T>) -> Self {
        build_store(self.builder, ptr, value);
        self
    }

    pub fn build_index_load<T: ValueType, const N: usize, I: Integer>(
        self,
        array: &Value<*mut [T; N]>,
        index: &Value<I>,
    ) -> (Value<T>, Self) {
        (build_index_load(self.builder, array, index), self)
    }

    pub fn build_index_store<T: ValueType, const N: usize, I: Integer>(
        self,
        array: &Value<*mut [T; N]>,
        index: &Value<I>,
        value: &Value<T>,
    ) -> Self {
        build_index_store(self.builder, array, index, value);
        self
    }

    pub fn build_struct<A: ValueType, B: ValueType>(
        self,
        a: &Value<A>,
        b: &Value<B>,
    ) -> (Value<*mut (A, B)>, Self) {
        (build_struct(self.builder, a, b), self)
    }

    pub fn build_jump_table<T: ValueType>(
        self,
        switch: &Value<T>,
        default: &Block,
    ) -> JumpTable<T> {
        JumpTable::new(self, switch.value(), default)
    }

    pub fn build_unreachable(self) {
        build_unreachable(self.builder);
    }

    pub fn build_jump(self, block: &Block) {
        build_jump(self.builder, block);
    }

    pub fn build_conditional_jump<T: Integer>(self, value: &Value<T>, t: &Block, f: &Block) {
        build_conditional_jump(self.builder, value, t, f);
    }

    pub fn build_ret<T: ValueType>(self, value: &Value<T>) {
        build_ret(self.builder, value);
    }

    pub fn build_void_ret(self) {
        build_void_ret(self.builder);
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
        }
    }
}

fn build_call<T: FunctionType>(
    builder: *mut LLVMBuilder,
    function: &Function<T>,
    params: T::Params,
) -> T::Return {
    function.build_call(builder, params)
}

fn build_variadic_call<T: VariadicFunctionType>(
    builder: *mut LLVMBuilder,
    function: &Function<T>,
    params: T::Params,
    variadic_params: &[UntypedValue],
) -> T::Return {
    function.build_variadic_call(builder, params, variadic_params)
}

fn build_int_cast<T: Integer, U: Integer>(builder: *mut LLVMBuilder, from: &Value<T>) -> Value<U> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        Value::new(LLVMBuildIntCast(
            builder,
            from.value(),
            U::value_type(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_add<T: Integer>(builder: *mut LLVMBuilder, lhs: &Value<T>, rhs: &Value<T>) -> Value<T> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        Value::new(LLVMBuildAdd(
            builder,
            lhs.value(),
            rhs.value(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_sub<T: Integer>(builder: *mut LLVMBuilder, lhs: &Value<T>, rhs: &Value<T>) -> Value<T> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        Value::new(LLVMBuildSub(
            builder,
            lhs.value(),
            rhs.value(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_eq<T: Integer>(builder: *mut LLVMBuilder, lhs: &Value<T>, rhs: &Value<T>) -> Value<T> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        let result = LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntEQ,
            lhs.value(),
            rhs.value(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        );

        Value::new(LLVMBuildIntCast(
            builder,
            result,
            T::value_type(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_lt<T: Integer>(builder: *mut LLVMBuilder, lhs: &Value<T>, rhs: &Value<T>) -> Value<T> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        let result = LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntSLT,
            lhs.value(),
            rhs.value(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        );

        Value::new(LLVMBuildIntCast(
            builder,
            result,
            T::value_type(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_conditional_value<T: Integer, U: ValueType>(
    builder: *mut LLVMBuilder,
    value: &Value<T>,
    t: &Value<U>,
    f: &Value<U>,
) -> Value<U> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        let cast = LLVMBuildIntCast(
            builder,
            value.value(),
            LLVMInt1Type(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        );

        Value::new(LLVMBuildSelect(
            builder,
            cast,
            t.value(),
            f.value(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_load<T: ValueType>(builder: *mut LLVMBuilder, ptr: &Value<*mut T>) -> Value<T> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        Value::new(LLVMBuildLoad2(
            builder,
            T::value_type(),
            ptr.value(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_store<T: ValueType>(builder: *mut LLVMBuilder, ptr: &Value<*mut T>, value: &Value<T>) {
    unsafe {
        LLVMBuildStore(builder, value.value(), ptr.value());
    }
}

fn build_index_load<T: ValueType, const N: usize, I: Integer>(
    builder: *mut LLVMBuilder,
    array: &Value<*mut [T; N]>,
    index: &Value<I>,
) -> Value<T> {
    let ep = unsafe {
        let name = CString::new("").unwrap();
        let mut indices = [I::zero(), index.value()];

        LLVMBuildGEP2(
            builder,
            <[T; N] as ValueType>::value_type(),
            array.value(),
            indices.as_mut_ptr(),
            2,
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        )
    };

    let value = unsafe {
        let name = CString::new("").unwrap();

        Value::new(LLVMBuildLoad2(
            builder,
            T::value_type(),
            ep,
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        ))
    };

    value
}

fn build_index_store<T: ValueType, const N: usize, I: Integer>(
    builder: *mut LLVMBuilder,
    array: &Value<*mut [T; N]>,
    index: &Value<I>,
    value: &Value<T>,
) {
    let ep = unsafe {
        let name = CString::new("").unwrap();
        let mut indices = [I::zero(), index.value()];

        LLVMBuildGEP2(
            builder,
            <[T; N] as ValueType>::value_type(),
            array.value(),
            indices.as_mut_ptr(),
            2,
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        )
    };

    unsafe {
        LLVMBuildStore(builder, value.value(), ep);
    }
}

fn build_struct<A: ValueType, B: ValueType>(
    builder: *mut LLVMBuilder,
    a: &Value<A>,
    b: &Value<B>,
) -> Value<*mut (A, B)> {
    let value = unsafe {
        let name = CString::new("").unwrap();

        LLVMBuildAlloca(
            builder,
            <(A, B) as ValueType>::value_type(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        )
    };

    let first_ep = unsafe {
        let name = CString::new("").unwrap();

        LLVMBuildStructGEP2(
            builder,
            <(A, B) as ValueType>::value_type(),
            value,
            0,
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        )
    };

    let second_ep = unsafe {
        let name = CString::new("").unwrap();

        LLVMBuildStructGEP2(
            builder,
            <(A, B) as ValueType>::value_type(),
            value,
            1,
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        )
    };

    unsafe {
        LLVMBuildStore(builder, a.value(), first_ep);
        LLVMBuildStore(builder, b.value(), second_ep);
    }

    Value::new(value)
}

fn build_unreachable(builder: *mut LLVMBuilder) {
    unsafe {
        LLVMBuildUnreachable(builder);
    }
}

fn build_jump(builder: *mut LLVMBuilder, block: &Block) {
    unsafe {
        LLVMBuildBr(builder, block.value());
    }
}

fn build_conditional_jump<T: Integer>(
    builder: *mut LLVMBuilder,
    value: &Value<T>,
    t: &Block,
    f: &Block,
) {
    unsafe {
        let zero = T::zero();
        let name = CString::new("").unwrap();
        let cmp = LLVMBuildICmp(
            builder,
            LLVMIntPredicate::LLVMIntEQ,
            zero,
            value.value(),
            name.to_bytes_with_nul().as_ptr().cast::<i8>(),
        );
        LLVMBuildCondBr(builder, cmp, t.value(), f.value());
    }
}

fn build_ret<T: ValueType>(builder: *mut LLVMBuilder, value: &Value<T>) {
    unsafe {
        LLVMBuildRet(builder, value.value());
    }
}

fn build_void_ret(builder: *mut LLVMBuilder) {
    unsafe {
        LLVMBuildRetVoid(builder);
    }
}
