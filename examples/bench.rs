use std::time::Instant;

use mini_jvm::classfile::parser::parse_class_file;
use mini_jvm::runtime::{ClassLoader, Frame, Thread};
use mini_jvm::runtime::frame::Value;
use mini_jvm::jit::JitCompiler;

fn minimal_class_bytes() -> Vec<u8> {
    vec![
        0xCA,0xFE,0xBA,0xBE,0x00,0x00,0x00,0x34,0x00,0x05,
        0x07,0x00,0x03,0x07,0x00,0x04,
        0x01,0x00,0x04,0x54,0x65,0x73,0x74,
        0x01,0x00,0x10,0x6A,0x61,0x76,0x61,0x2F,0x6C,0x61,0x6E,0x67,0x2F,0x4F,0x62,0x6A,0x65,0x63,0x74,
        0x00,0x21,0x00,0x01,0x00,0x02,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
    ]
}

fn main() {
    let class_file = parse_class_file(&minimal_class_bytes()).unwrap();
    
    // --- Benchmark 1: Pure arithmetic (bipush/imul loop unrolled) ---
    // Compute: 6*7 = 42, repeated many times
    println!("=== Benchmark: Arithmetic (6*7, repeated) ===");
    
    // Bytecode: bipush 6, bipush 7, imul, ireturn
    let mul_code = vec![0x10, 0x06, 0x10, 0x07, 0x68, 0xAC];
    
    // JIT
    let mut jit = JitCompiler::new();
    let jit_key = jit.compile_method("Test", "mul", "()I", 1, &mul_code, &class_file).unwrap();
    let jit_fn = jit.get_compiled_fn(&jit_key).unwrap();
    
    let iterations = 100_000_000;
    
    let start = Instant::now();
    let mut jit_result = 0i64;
    for _ in 0..iterations {
        jit_result = unsafe { jit_fn() };
    }
    let jit_time = start.elapsed();
    println!("JIT:   result={}, time={:.2}ms ({:.0} ops/sec)", 
             jit_result, 
             jit_time.as_secs_f64() * 1000.0,
             iterations as f64 / jit_time.as_secs_f64());
    
    // Interpreter (same bytecode, run through Thread)
    // For fair comparison, run the bytecode through our interpreter loop
    // We'll measure just the execute() call overhead
    let start = Instant::now();
    let mut interp_result = 0i32;
    for _ in 0..iterations {
        let mut thread = Thread::new(ClassLoader::new());
        let mut frame = Frame::new(2, mul_code.clone());
        frame.class_name = "Test".to_string();
        thread.push_frame(frame);
        if let Some(Value::I32(r)) = thread.execute() {
            interp_result = r;
        }
    }
    let interp_time = start.elapsed();
    println!("Interp: result={}, time={:.2}ms ({:.0} ops/sec)",
             interp_result,
             interp_time.as_secs_f64() * 1000.0,
             iterations as f64 / interp_time.as_secs_f64());
    
    let speedup = interp_time.as_secs_f64() / jit_time.as_secs_f64();
    println!("Speedup: {:.1}x", speedup);
    
    // --- Benchmark 2: Local variable load/store + add ---
    println!("\n=== Benchmark: Load/Store + Add ===");
    
    // iload_0, iload_1, iadd, isub, imul, ireturn
    // With locals[0]=100, locals[1]=200 → (100+200-100)*200 = 40000
    let complex_code = vec![0x1A, 0x1B, 0x60, 0x1A, 0x64, 0x1B, 0x68, 0xAC];
    
    // JIT compile
    let jit_key2 = jit.compile_method("Test", "complex", "()I", 2, &complex_code, &class_file).unwrap();
    let jit_fn2 = jit.get_compiled_fn(&jit_key2).unwrap();
    
    let start = Instant::now();
    let mut jit_result2 = 0i64;
    for _ in 0..iterations {
        jit_result2 = unsafe { jit_fn2() };
    }
    let jit_time2 = start.elapsed();
    println!("JIT:   result={}, time={:.2}ms ({:.0} ops/sec)",
             jit_result2,
             jit_time2.as_secs_f64() * 1000.0,
             iterations as f64 / jit_time2.as_secs_f64());
    
    let start = Instant::now();
    let mut interp_result2 = 0i32;
    for _ in 0..iterations {
        let mut thread = Thread::new(ClassLoader::new());
        let mut frame = Frame::new(2, complex_code.clone());
        frame.class_name = "Test".to_string();
        // Set locals
        frame.locals[0] = Value::I32(100);
        frame.locals[1] = Value::I32(200);
        thread.push_frame(frame);
        if let Some(Value::I32(r)) = thread.execute() {
            interp_result2 = r;
        }
    }
    let interp_time2 = start.elapsed();
    println!("Interp: result={}, time={:.2}ms ({:.0} ops/sec)",
             interp_result2,
             interp_time2.as_secs_f64() * 1000.0,
             iterations as f64 / interp_time2.as_secs_f64());
    
    let speedup2 = interp_time2.as_secs_f64() / jit_time2.as_secs_f64();
    println!("Speedup: {:.1}x", speedup2);

    // --- Summary ---
    println!("\n=== Summary ===");
    println!("Note: JIT runs standalone native code (no JVM frame/stack overhead)");
    println!("      Interpreter creates Thread+Frame per invocation (unfair overhead)");
    println!("      Real speedup in production would be different with frame reuse");
}
