use std::{
    ffi::{CStr, CString},
    fmt::{self, Debug, Formatter},
    path::{Path, PathBuf},
    ptr::null_mut,
};

use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMArrayType2, LLVMConstArray2, LLVMConstBitCast,
        LLVMConstString, LLVMDisposeMessage, LLVMDisposeModule, LLVMGetModuleIdentifier,
        LLVMInt8Type, LLVMModuleCreateWithName, LLVMPrintModuleToString, LLVMSetGlobalConstant,
        LLVMSetInitializer, LLVMSetLinkage, LLVMSetSourceFileName,
    },
    target::{
        LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter,
        LLVM_InitializeNativeDisassembler, LLVM_InitializeNativeTarget,
    },
    target_machine::{
        LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine,
        LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetHostCPUFeatures,
        LLVMGetHostCPUName, LLVMGetTargetFromTriple, LLVMRelocMode, LLVMTargetMachineEmitToFile,
        LLVMTargetRef,
    },
    LLVMLinkage, LLVMModule,
};
use tempfile::{tempdir, TempDir};

use crate::{
    types::ValueType,
    value::{Constant, Integer},
    Function, FunctionType, Value,
};

#[derive(Clone, Copy)]
pub enum CompileOutput {
    Assembly,
    Object,
}

pub struct OutputFile {
    _dir: TempDir,
    path: PathBuf,
}

pub struct Module {
    module: *mut LLVMModule,
}

impl CompileOutput {
    fn file_type(&self) -> LLVMCodeGenFileType {
        match self {
            CompileOutput::Assembly => LLVMCodeGenFileType::LLVMAssemblyFile,
            CompileOutput::Object => LLVMCodeGenFileType::LLVMObjectFile,
        }
    }

    fn file_name(&self, module: *mut LLVMModule) -> String {
        let module_name = unsafe {
            let mut length = 0;
            let name = LLVMGetModuleIdentifier(module, &mut length);
            CStr::from_ptr(name).to_owned().into_string().unwrap()
        };

        module_name
            + match self {
                CompileOutput::Assembly => ".s",
                CompileOutput::Object => ".o",
            }
    }
}

impl OutputFile {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Debug for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        unsafe {
            let s = LLVMPrintModuleToString(self.module);
            let result = writeln!(f, "{}", CStr::from_ptr(s).to_string_lossy());
            LLVMDisposeMessage(s);
            result
        }
    }
}

impl Module {
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(name: S, source: P) -> Self {
        let name = CString::new(name.as_ref()).unwrap();
        let source = CString::new(source.as_ref().as_os_str().to_str().unwrap()).unwrap();

        let module = unsafe {
            let module = LLVMModuleCreateWithName(name.to_bytes_with_nul().as_ptr().cast::<i8>());

            let source_bytes = source.to_bytes();
            LLVMSetSourceFileName(
                module,
                source_bytes.as_ptr().cast::<i8>(),
                source_bytes.len(),
            );

            module
        };

        Self { module }
    }

    pub fn add_function<S: AsRef<str>, T: FunctionType>(&self, name: S) -> Function<T> {
        let name = CString::new(name.as_ref()).unwrap();

        let function = unsafe {
            LLVMAddFunction(
                self.module,
                name.to_bytes_with_nul().as_ptr().cast::<i8>(),
                T::function_type(),
            )
        };

        Function::new(function)
    }

    pub fn add_named_string<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        name: S1,
        value: S2,
    ) -> Value<String> {
        let cstring = CString::new(value.as_ref()).unwrap();
        let bytes = cstring.to_bytes_with_nul();

        let global = {
            let name = CString::new(name.as_ref()).unwrap();

            unsafe {
                LLVMAddGlobal(
                    self.module,
                    LLVMArrayType2(LLVMInt8Type(), bytes.len() as u64),
                    name.to_bytes_with_nul().as_ptr().cast::<i8>(),
                )
            }
        };

        let value = unsafe { LLVMConstString(bytes.as_ptr().cast::<i8>(), bytes.len() as u32, 1) };

        unsafe {
            LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);
            LLVMSetGlobalConstant(global, 1);
            LLVMSetInitializer(global, value);

            Value::new(LLVMConstBitCast(global, String::value_type()))
        }
    }

    pub fn add_string<S: AsRef<str>>(&self, value: S) -> Value<String> {
        self.add_named_string("string", value)
    }

    pub fn add_named_array<S: AsRef<str>, T: Integer, const N: usize>(
        &self,
        name: S,
    ) -> Value<*mut [T; N]> {
        let global = {
            let name = CString::new(name.as_ref()).unwrap();

            unsafe {
                LLVMAddGlobal(
                    self.module,
                    LLVMArrayType2(T::value_type(), N as u64),
                    name.to_bytes_with_nul().as_ptr().cast::<i8>(),
                )
            }
        };

        unsafe {
            LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);
            LLVMSetGlobalConstant(global, 0);

            let mut vals = [T::zero(); N];
            let value = LLVMConstArray2(T::value_type(), vals.as_mut_ptr(), N as u64);
            LLVMSetInitializer(global, value);

            Value::new(global)
        }
    }

    pub fn add_array<T: Integer, const N: usize>(&self) -> Value<*mut [T; N]> {
        self.add_named_array("array")
    }

    pub fn add_named_global<S: AsRef<str>, T: ValueType + Constant>(
        &self,
        name: S,
        value: T,
    ) -> Value<*mut T> {
        let global = {
            let name = CString::new(name.as_ref()).unwrap();

            unsafe {
                LLVMAddGlobal(
                    self.module,
                    T::value_type(),
                    name.to_bytes_with_nul().as_ptr().cast::<i8>(),
                )
            }
        };

        let value = value.constant();

        unsafe {
            LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);
            LLVMSetGlobalConstant(global, 0);
            LLVMSetInitializer(global, value);

            Value::new(global)
        }
    }

    pub fn add_global<T: ValueType + Constant>(&self, value: T) -> Value<*mut T> {
        self.add_named_global("global", value)
    }

    pub fn compile_for_host(&self, output: CompileOutput) -> Result<OutputFile, String> {
        unsafe {
            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmParser();
            LLVM_InitializeNativeAsmPrinter();
            LLVM_InitializeNativeDisassembler();

            let triple = LLVMGetDefaultTargetTriple();

            let mut error_message: *mut i8 = null_mut();

            let mut target: LLVMTargetRef = null_mut();
            LLVMGetTargetFromTriple(triple, &mut target, &mut error_message as *mut _);

            if !error_message.is_null() {
                let string = CStr::from_ptr(error_message)
                    .to_owned()
                    .into_string()
                    .unwrap();

                LLVMDisposeMessage(error_message);
                return Err(string);
            }

            let cpu = LLVMGetHostCPUName();
            let features = LLVMGetHostCPUFeatures();

            let machine = LLVMCreateTargetMachine(
                target,
                triple,
                cpu,
                features,
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocStatic,
                LLVMCodeModel::LLVMCodeModelDefault,
            );

            LLVMDisposeMessage(cpu);
            LLVMDisposeMessage(features);

            let filename = output.file_name(self.module);

            let dir = tempdir().map_err(|_| "Unable to create temporary directory")?;
            let path = dir.path().join(filename);

            LLVMTargetMachineEmitToFile(
                machine,
                self.module,
                path.as_os_str().as_encoded_bytes().as_ptr() as *const _,
                output.file_type(),
                &mut error_message as *mut _,
            );

            if !error_message.is_null() {
                let string = CStr::from_ptr(error_message)
                    .to_owned()
                    .into_string()
                    .unwrap();

                LLVMDisposeMessage(error_message);
                return Err(string);
            }

            LLVMDisposeTargetMachine(machine);

            Ok(OutputFile { _dir: dir, path })
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module);
        }
    }
}
