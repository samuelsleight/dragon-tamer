use std::marker::PhantomData;

use llvm_sys::{
    core::{LLVMAddCase, LLVMBuildSwitch},
    LLVMBasicBlock, LLVMValue,
};

use crate::{Block, Builder, Value, ValueType};

struct Case {
    value: *mut LLVMValue,
    block: *mut LLVMBasicBlock,
}

#[must_use]
pub struct JumpTable<T: ValueType> {
    builder: Builder,
    value: *mut LLVMValue,
    default: *mut LLVMBasicBlock,
    cases: Vec<Case>,
    phantom: PhantomData<T>,
}

impl Case {
    pub fn new(value: *mut LLVMValue, block: &Block) -> Self {
        Self {
            value,
            block: block.value(),
        }
    }
}

impl<T: ValueType> JumpTable<T> {
    pub(crate) fn new(builder: Builder, value: *mut LLVMValue, default: &Block) -> Self {
        Self {
            builder,
            value,
            default: default.value(),
            cases: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn case(mut self, value: &Value<T>, block: &Block) -> Self {
        self.cases.push(Case::new(value.value(), block));
        self
    }

    pub fn finish(self) {
        unsafe {
            let switch = LLVMBuildSwitch(
                self.builder.builder,
                self.value,
                self.default,
                self.cases.len() as u32,
            );

            for Case { value, block } in self.cases {
                LLVMAddCase(switch, value, block);
            }
        }
    }
}
