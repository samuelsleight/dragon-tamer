use llvm_sys::{core::LLVMPositionBuilderAtEnd, LLVMBasicBlock};

use crate::Builder;

#[derive(Copy, Clone)]
pub struct Block {
    value: *mut LLVMBasicBlock,
}

impl Block {
    pub(crate) fn new(value: *mut LLVMBasicBlock) -> Self {
        Self { value }
    }

    pub fn build(&self) -> Builder {
        let builder = Builder::new();

        unsafe {
            LLVMPositionBuilderAtEnd(builder.builder, self.value);
        }

        builder
    }

    pub(crate) fn value(&self) -> *mut LLVMBasicBlock {
        self.value
    }
}
