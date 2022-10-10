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
    hlist::{Cons, HList, IndexableBy},
    jump_table::JumpTable,
    value::Integer,
    Block, Function, FunctionType, Value, ValueType,
};

#[must_use]
pub struct Builder {
    pub(crate) builder: *mut LLVMBuilder,
}

#[must_use]
pub struct BuilderEnv<List: HList> {
    builder: Builder,
    env: List,
}

pub struct BuiltEnv<List: HList> {
    env: List,
}

pub trait EnvValue<List: HList, T: ValueType> {
    fn as_value<'a, 'b>(&'b self, list: &'a List) -> &'a Value<T>
    where
        'b: 'a;
}

pub trait EnvParams<List: HList, Params> {
    fn as_value<'a, 'b>(&'b self, list: &'a List) -> Params
    where
        'b: 'a;
}

impl<List: HList, T: ValueType> EnvValue<List, T> for Value<T> {
    fn as_value<'a, 'b>(&'b self, list: &'a List) -> &'a Value<T>
    where
        'b: 'a,
    {
        self
    }
}

pub struct Env<const Index: usize>();

impl<List: HList, T: ValueType, const Index: usize> EnvValue<List, T> for Env<Index>
where
    List: IndexableBy<Index, Result = Value<T>>,
{
    fn as_value<'a, 'b>(&'b self, list: &'a List) -> &'a Value<T>
    where
        'b: 'a,
    {
        list.index()
    }
}

impl<List: HList> EnvParams<List, ()> for () {
    fn as_value<'a, 'b>(&'b self, list: &'a List)
    where
        'b: 'a,
    {
    }
}

impl<List: HList, T: ValueType + Sized, U: EnvValue<List, T>> EnvParams<List, (Value<T>,)> for (U,) {
    fn as_value<'a, 'b>(&'b self, list: &'a List) -> (Value<T>,)
    where
        'b: 'a,
    {
        (*self.0.as_value(list),)
    }
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
    ) -> BuilderEnv<Cons<T::Return, ()>> {
        BuilderEnv::new(build_call(self.builder, function, params), self)
    }

    pub fn build_add<T: Integer>(
        self,
        lhs: Value<T>,
        rhs: Value<T>,
    ) -> BuilderEnv<Cons<Value<T>, ()>> {
        BuilderEnv::new(build_add(self.builder, &lhs, &rhs), self)
    }

    pub fn build_sub<T: Integer>(
        self,
        lhs: Value<T>,
        rhs: Value<T>,
    ) -> BuilderEnv<Cons<Value<T>, ()>> {
        BuilderEnv::new(build_sub(self.builder, &lhs, &rhs), self)
    }

    pub fn build_eq<T: Integer>(
        self,
        lhs: Value<T>,
        rhs: Value<T>,
    ) -> BuilderEnv<Cons<Value<T>, ()>> {
        BuilderEnv::new(build_eq(self.builder, &lhs, &rhs), self)
    }

    pub fn build_load<T: ValueType>(self, ptr: Value<*mut T>) -> BuilderEnv<Cons<Value<T>, ()>> {
        BuilderEnv::new(build_load(self.builder, &ptr), self)
    }

    pub fn build_store<T: ValueType>(self, ptr: Value<*mut T>, value: Value<T>) -> Self {
        build_store(self.builder, &ptr, &value);
        self
    }

    pub fn build_index_load<T: ValueType, const N: usize, I: Integer>(
        self,
        array: Value<*mut [T; N]>,
        index: Value<I>,
    ) -> BuilderEnv<Cons<Value<T>, ()>> {
        BuilderEnv::new(build_index_load(self.builder, &array, &index), self)
    }

    pub fn build_index_store<T: ValueType, const N: usize, I: Integer>(
        self,
        array: Value<*mut [T; N]>,
        index: Value<I>,
        value: Value<T>,
    ) -> Self {
        build_index_store(self.builder, &array, &index, &value);
        self
    }

    pub fn build_struct<A: ValueType, B: ValueType>(
        self,
        a: Value<A>,
        b: Value<B>,
    ) -> BuilderEnv<Cons<Value<*mut (A, B)>, ()>> {
        BuilderEnv::new(build_struct(self.builder, &a, &b), self)
    }

    pub fn build_jump_table<T: ValueType>(self, switch: Value<T>, default: &Block) -> JumpTable<T> {
        JumpTable::new(self, switch.value(), default)
    }

    pub fn build_unreachable(self) {
        build_unreachable(self.builder);
    }

    pub fn build_jump(self, block: &Block) {
        build_jump(self.builder, block);
    }

    pub fn build_conditional_jump<T: Integer>(self, value: Value<T>, t: &Block, f: &Block) {
        build_conditional_jump(self.builder, &value, t, f);
    }

    pub fn build_ret<T: ValueType>(self, value: Value<T>) {
        build_ret(self.builder, &value);
    }

    pub fn build_void_ret(self) {
        build_void_ret(self.builder);
    }
}

impl<Value> BuilderEnv<Cons<Value, ()>> {
    pub fn new(value: Value, builder: Builder) -> Self {
        Self {
            builder,
            env: Cons::new(value),
        }
    }
}

impl<List: HList> BuilderEnv<List> {
    fn cons<Value>(self, value: Value) -> BuilderEnv<Cons<Value, List>> {
        BuilderEnv {
            builder: self.builder,
            env: self.env.cons(value),
        }
    }

    pub fn unwrap(self) -> (BuiltEnv<List>, Builder) {
        (BuiltEnv::new(self.env), self.builder)
    }
    pub fn build_call<T: FunctionType>(
        self,
        function: &Function<T>,
        params: impl EnvParams<List, T::Params>,
    ) -> BuilderEnv<Cons<T::Return, List>> {
        let value = build_call(self.builder.builder, function, params.as_value(&self.env));
        self.cons(value)
    }

    pub fn build_add<T: Integer>(
        self,
        lhs: impl EnvValue<List, T>,
        rhs: impl EnvValue<List, T>,
    ) -> BuilderEnv<Cons<Value<T>, List>> {
        let value = build_add(
            self.builder.builder,
            lhs.as_value(&self.env),
            rhs.as_value(&self.env),
        );

        self.cons(value)
    }

    pub fn build_sub<T: Integer>(
        self,
        lhs: impl EnvValue<List, T>,
        rhs: impl EnvValue<List, T>,
    ) -> BuilderEnv<Cons<Value<T>, List>> {
        let value = build_sub(
            self.builder.builder,
            lhs.as_value(&self.env),
            rhs.as_value(&self.env),
        );

        self.cons(value)
    }

    pub fn build_eq<T: Integer>(
        self,
        lhs: impl EnvValue<List, T>,
        rhs: impl EnvValue<List, T>,
    ) -> BuilderEnv<Cons<Value<T>, List>> {
        let value = build_eq(
            self.builder.builder,
            lhs.as_value(&self.env),
            rhs.as_value(&self.env),
        );

        self.cons(value)
    }

    pub fn build_load<T: ValueType>(
        self,
        ptr: impl EnvValue<List, *mut T>,
    ) -> BuilderEnv<Cons<Value<T>, List>> {
        let value = build_load(self.builder.builder, ptr.as_value(&self.env));
        self.cons(value)
    }

    pub fn build_store<T: ValueType>(
        self,
        ptr: impl EnvValue<List, *mut T>,
        value: impl EnvValue<List, T>,
    ) -> Self {
        build_store(
            self.builder.builder,
            ptr.as_value(&self.env),
            value.as_value(&self.env),
        );

        self
    }

    pub fn build_index_load<T: ValueType, const N: usize, I: Integer>(
        self,
        array: impl EnvValue<List, *mut [T; N]>,
        index: impl EnvValue<List, I>,
    ) -> BuilderEnv<Cons<Value<T>, List>> {
        let value = build_index_load(
            self.builder.builder,
            array.as_value(&self.env),
            index.as_value(&self.env),
        );

        self.cons(value)
    }

    pub fn build_index_store<T: ValueType, const N: usize, I: Integer>(
        self,
        array: impl EnvValue<List, *mut [T; N]>,
        index: impl EnvValue<List, I>,
        value: impl EnvValue<List, T>,
    ) -> Self {
        build_index_store(
            self.builder.builder,
            array.as_value(&self.env),
            index.as_value(&self.env),
            value.as_value(&self.env),
        );

        self
    }

    pub fn build_struct<A: ValueType, B: ValueType>(
        self,
        a: impl EnvValue<List, A>,
        b: impl EnvValue<List, B>,
    ) -> BuilderEnv<Cons<Value<*mut (A, B)>, List>> {
        let value = build_struct(
            self.builder.builder,
            a.as_value(&self.env),
            b.as_value(&self.env),
        );

        self.cons(value)
    }

    pub fn build_jump_table<T: ValueType>(
        self,
        switch: impl EnvValue<List, T>,
        default: &Block,
    ) -> JumpTable<T> {
        JumpTable::new(self.builder, switch.as_value(&self.env).value(), default)
    }

    pub fn build_unreachable(self) -> BuiltEnv<List> {
        build_unreachable(self.builder.builder);
        BuiltEnv::new(self.env)
    }

    pub fn build_jump(self, block: &Block) -> BuiltEnv<List> {
        build_jump(self.builder.builder, block);
        BuiltEnv::new(self.env)
    }

    pub fn build_conditional_jump<T: Integer>(
        self,
        value: impl EnvValue<List, T>,
        t: &Block,
        f: &Block,
    ) -> BuiltEnv<List> {
        build_conditional_jump(self.builder.builder, value.as_value(&self.env), t, f);
        BuiltEnv::new(self.env)
    }

    pub fn build_ret<T: ValueType>(self, value: impl EnvValue<List, T>) -> BuiltEnv<List> {
        build_ret(self.builder.builder, value.as_value(&self.env));
        BuiltEnv::new(self.env)
    }

    pub fn build_void_ret(self) -> BuiltEnv<List> {
        build_void_ret(self.builder.builder);
        BuiltEnv::new(self.env)
    }
}

impl<List: HList> BuiltEnv<List> {
    fn new(env: List) -> Self {
        Self { env }
    }

    pub fn get_value<const Index: usize>(&self) -> &<List as IndexableBy<Index>>::Result
    where
        List: IndexableBy<Index>,
    {
        self.env.index()
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
