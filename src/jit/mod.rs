/// JIT Compiler module using Cranelift.
///
/// Baseline JIT: translates JVM bytecode to native machine code.
/// Supports: arithmetic, load/store, constants, returns.

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

use crate::classfile::ClassFile;

/// Manages JIT compilation state.
pub struct JitCompiler {
    module: JITModule,
    /// Compiled method -> native fn ptr
    compiled: HashMap<String, *const u8>,
    /// Method invocation counters
    counters: HashMap<String, u32>,
    /// Threshold for JIT compilation
    compile_threshold: u32,
}

/// The type of our compiled functions.
type JittedMethod = unsafe extern "C" fn() -> i64;

impl JitCompiler {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        JitCompiler {
            module,
            compiled: HashMap::new(),
            counters: HashMap::new(),
            compile_threshold: 100,
        }
    }

    /// Record a method invocation. Returns true if JIT compilation should be triggered.
    pub fn record_invocation(&mut self, key: &str) -> bool {
        if self.compiled.contains_key(key) {
            return false;
        }
        let count = self.counters.entry(key.to_string()).or_insert(0);
        *count += 1;
        *count >= self.compile_threshold
    }

    /// Check if a method has been JIT-compiled.
    pub fn is_compiled(&self, key: &str) -> bool {
        self.compiled.contains_key(key)
    }

    /// Get the compiled function pointer for a method.
    pub fn get_compiled_fn(&self, key: &str) -> Option<JittedMethod> {
        self.compiled.get(key).map(|&ptr| unsafe {
            std::mem::transmute::<*const u8, JittedMethod>(ptr)
        })
    }

    /// Try to compile a method. Returns Ok(key) on success, Err(reason) if can't compile.
    /// Methods with unsupported opcodes will return Err (interpreter fallback).
    pub fn compile_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
        max_locals: usize,
        code: &[u8],
        _class_file: &ClassFile,
    ) -> Result<String, String> {
        let key = format!("{}.{}{}", class_name, method_name, descriptor);

        if self.compiled.contains_key(&key) {
            return Ok(key);
        }

        let mut ctx = self.module.make_context();

        // Function signature: () -> i64 (standalone test)
        // In real integration, this would take thread/context params
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));

        let func_id = self.module.declare_function(&key, Linkage::Local, &sig)
            .map_err(|e| format!("declare_function failed: {}", e))?;

        ctx.func.signature = sig;

        let mut builder_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);

            // Single entry block — straight-line code for now (no branches)
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block); // No predecessors to worry about

            // Create variables for locals and stack
            let total_slots = max_locals + 20; // extra for operand stack
            let mut vars: Vec<Variable> = Vec::with_capacity(total_slots);
            for _ in 0..total_slots {
                let var = builder.declare_var(types::I64);
                vars.push(var);
            }

            // Initialize locals to 0
            for i in 0..max_locals {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.def_var(vars[i], zero);
            }

            let mut stack_depth: usize = 0;
            let mut pc = 0;

            while pc < code.len() {
                let opcode = code[pc];
                pc += 1;

                match opcode {
                    // iconst_m1 (0x02)
                    0x02 => {
                        let val = builder.ins().iconst(types::I64, -1);
                        builder.def_var(vars[max_locals + stack_depth], val);
                        stack_depth += 1;
                    }
                    // iconst_0..iconst_5 (0x03-0x08)
                    0x03..=0x08 => {
                        let n = (opcode - 0x03) as i64;
                        let val = builder.ins().iconst(types::I64, n);
                        builder.def_var(vars[max_locals + stack_depth], val);
                        stack_depth += 1;
                    }
                    // bipush (0x10)
                    0x10 => {
                        let n = code[pc] as i8 as i64;
                        pc += 1;
                        let val = builder.ins().iconst(types::I64, n);
                        builder.def_var(vars[max_locals + stack_depth], val);
                        stack_depth += 1;
                    }
                    // sipush (0x11)
                    0x11 => {
                        let hi = code[pc] as i16;
                        let lo = code[pc + 1] as i16;
                        let n = ((hi << 8) | lo) as i64;
                        pc += 2;
                        let val = builder.ins().iconst(types::I64, n);
                        builder.def_var(vars[max_locals + stack_depth], val);
                        stack_depth += 1;
                    }
                    // iload_<n> (0x1A-0x1D)
                    0x1A..=0x1D => {
                        let idx = (opcode - 0x1A) as usize;
                        let val = builder.use_var(vars[idx]);
                        builder.def_var(vars[max_locals + stack_depth], val);
                        stack_depth += 1;
                    }
                    // iload (0x15)
                    0x15 => {
                        let idx = code[pc] as usize;
                        pc += 1;
                        let val = builder.use_var(vars[idx]);
                        builder.def_var(vars[max_locals + stack_depth], val);
                        stack_depth += 1;
                    }
                    // istore_<n> (0x3B-0x3E)
                    0x3B..=0x3E => {
                        stack_depth -= 1;
                        let idx = (opcode - 0x3B) as usize;
                        let val = builder.use_var(vars[max_locals + stack_depth]);
                        builder.def_var(vars[idx], val);
                    }
                    // istore (0x36)
                    0x36 => {
                        stack_depth -= 1;
                        let idx = code[pc] as usize;
                        pc += 1;
                        let val = builder.use_var(vars[max_locals + stack_depth]);
                        builder.def_var(vars[idx], val);
                    }
                    // iadd (0x60)
                    0x60 => {
                        stack_depth -= 1;
                        let b = builder.use_var(vars[max_locals + stack_depth]);
                        stack_depth -= 1;
                        let a = builder.use_var(vars[max_locals + stack_depth]);
                        let result = builder.ins().iadd(a, b);
                        builder.def_var(vars[max_locals + stack_depth], result);
                        stack_depth += 1;
                    }
                    // isub (0x64)
                    0x64 => {
                        stack_depth -= 1;
                        let b = builder.use_var(vars[max_locals + stack_depth]);
                        stack_depth -= 1;
                        let a = builder.use_var(vars[max_locals + stack_depth]);
                        let result = builder.ins().isub(a, b);
                        builder.def_var(vars[max_locals + stack_depth], result);
                        stack_depth += 1;
                    }
                    // imul (0x68)
                    0x68 => {
                        stack_depth -= 1;
                        let b = builder.use_var(vars[max_locals + stack_depth]);
                        stack_depth -= 1;
                        let a = builder.use_var(vars[max_locals + stack_depth]);
                        let result = builder.ins().imul(a, b);
                        builder.def_var(vars[max_locals + stack_depth], result);
                        stack_depth += 1;
                    }
                    // idiv (0x6C)
                    0x6C => {
                        stack_depth -= 1;
                        let b = builder.use_var(vars[max_locals + stack_depth]);
                        stack_depth -= 1;
                        let a = builder.use_var(vars[max_locals + stack_depth]);
                        let result = builder.ins().sdiv(a, b);
                        builder.def_var(vars[max_locals + stack_depth], result);
                        stack_depth += 1;
                    }
                    // irem (0x70)
                    0x70 => {
                        stack_depth -= 1;
                        let b = builder.use_var(vars[max_locals + stack_depth]);
                        stack_depth -= 1;
                        let a = builder.use_var(vars[max_locals + stack_depth]);
                        let result = builder.ins().srem(a, b);
                        builder.def_var(vars[max_locals + stack_depth], result);
                        stack_depth += 1;
                    }
                    // ineg (0x74)
                    0x74 => {
                        stack_depth -= 1;
                        let a = builder.use_var(vars[max_locals + stack_depth]);
                        let zero = builder.ins().iconst(types::I64, 0);
                        let result = builder.ins().isub(zero, a);
                        builder.def_var(vars[max_locals + stack_depth], result);
                        stack_depth += 1;
                    }
                    // ireturn (0xAC)
                    0xAC => {
                        stack_depth -= 1;
                        let val = builder.use_var(vars[max_locals + stack_depth]);
                        builder.ins().return_(&[val]);
                        // After return, stop processing
                        break;
                    }
                    // return (0xB1)
                    0xB1 => {
                        let zero = builder.ins().iconst(types::I64, 0);
                        builder.ins().return_(&[zero]);
                        break;
                    }
                    // Unsupported opcode — bail out
                    _ => {
                        return Err(format!(
                            "Unsupported opcode 0x{:02X} at pc={} in {}", opcode, pc - 1, key
                        ));
                    }
                }
            }

            // If we didn't hit a return, add one
            if !code.is_empty() {
                // Check if last instruction was a return
                let last = code[code.len() - 1];
                if last != 0xAC && last != 0xB1 {
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().return_(&[zero]);
                }
            }

            builder.finalize();
        }

        self.module.define_function(func_id, &mut ctx)
            .map_err(|e| format!("define_function failed: {}", e))?;
        self.module.clear_context(&mut ctx);
        self.module.finalize_definitions().unwrap();

        let code_ptr = self.module.get_finalized_function(func_id);
        self.compiled.insert(key.clone(), code_ptr);

        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn minimal_class_bytes() -> Vec<u8> {
        vec![
            0xCA,0xFE,0xBA,0xBE,0x00,0x00,0x00,0x34,0x00,0x05,
            0x07,0x00,0x03,0x07,0x00,0x04,
            0x01,0x00,0x04,0x54,0x65,0x73,0x74,
            0x01,0x00,0x10,0x6A,0x61,0x76,0x61,0x2F,0x6C,0x61,0x6E,0x67,0x2F,0x4F,0x62,0x6A,0x65,0x63,0x74,
            0x00,0x21,0x00,0x01,0x00,0x02,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        ]
    }

    #[test]
    fn test_compile_and_run_add() {
        let mut jit = JitCompiler::new();
        let class_file = crate::classfile::parse_class_file(&minimal_class_bytes()).unwrap();
        let bytecode = vec![0x1A, 0x1B, 0x60, 0xAC];
        let result = jit.compile_method("Test", "add", "(II)I", 2, &bytecode, &class_file);
        assert!(result.is_ok(), "Failed: {:?}", result);
        let func = jit.get_compiled_fn(&result.unwrap()).unwrap();
        assert_eq!(unsafe { func() }, 0); // 0+0=0
    }

    #[test]
    fn test_compile_constants() {
        let mut jit = JitCompiler::new();
        let class_file = crate::classfile::parse_class_file(&minimal_class_bytes()).unwrap();
        let bytecode = vec![0x10, 0x2A, 0xAC]; // bipush 42, ireturn
        let result = jit.compile_method("Test", "answer", "()I", 1, &bytecode, &class_file);
        assert!(result.is_ok());
        let func = jit.get_compiled_fn(&result.unwrap()).unwrap();
        assert_eq!(unsafe { func() }, 42);
    }

    #[test]
    fn test_compile_arithmetic() {
        let mut jit = JitCompiler::new();
        let class_file = crate::classfile::parse_class_file(&minimal_class_bytes()).unwrap();
        let bytecode = vec![0x10, 0x06, 0x10, 0x07, 0x68, 0xAC]; // 6*7=42
        let result = jit.compile_method("Test", "mul", "()I", 1, &bytecode, &class_file);
        assert!(result.is_ok());
        let func = jit.get_compiled_fn(&result.unwrap()).unwrap();
        assert_eq!(unsafe { func() }, 42);
    }
}
