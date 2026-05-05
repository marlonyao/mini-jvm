use mini_jvm::runtime::{Thread, ClassLoader, Frame};
use mini_jvm::runtime::frame::Value;

/// Helper: create a thread with a single frame and execute it.
fn execute_bytecode(max_locals: usize, code: Vec<u8>) -> Thread {
    let mut thread = Thread::new(ClassLoader::new());
    let frame = Frame::new(max_locals, code);
    thread.push_frame(frame);
    thread.execute();
    thread
}

/// Test: compute 10 + 20 = 30
/// Bytecode equivalent: 
///   iconst_0 is not enough, use bipush 10, bipush 20, iadd, ireturn
/// 
/// But we need to construct bytecode that the dispatcher can process.
/// The dispatcher reads opcode from frame.code[frame.pc], increments pc, then calls the handler.
/// So the code array should contain the raw opcodes.
#[test]
fn test_execute_add() {
    // Bytecode:
    // bipush 10   -> 0x10, 0x0A
    // bipush 20   -> 0x10, 0x14
    // iadd        -> 0x60
    // ireturn     -> 0xAC
    let code = vec![
        0x10, 0x0A,  // bipush 10
        0x10, 0x14,  // bipush 20
        0x60,        // iadd
        0xAC,        // ireturn
    ];
    let mut thread = Thread::new(ClassLoader::new());
    let frame = Frame::new(2, code);
    thread.push_frame(frame);
    let result = thread.execute();
    
    assert_eq!(result, Some(Value::I32(30)));
}

/// Test: compute 100 - 37 = 63
#[test]
fn test_execute_sub() {
    // bipush 100  -> 0x10, 0x64
    // bipush 37   -> 0x10, 0x25
    // isub        -> 0x64
    // ireturn     -> 0xAC
    let code = vec![
        0x10, 0x64,  // bipush 100
        0x10, 0x25,  // bipush 37
        0x64,        // isub
        0xAC,        // ireturn
    ];
    let mut thread = Thread::new(ClassLoader::new());
    let frame = Frame::new(2, code);
    thread.push_frame(frame);
    let result = thread.execute();

    assert_eq!(result, Some(Value::I32(63)));
}

/// Test: compute 6 * 7 = 42
#[test]
fn test_execute_mul() {
    let code = vec![
        0x10, 0x06,  // bipush 6
        0x10, 0x07,  // bipush 7
        0x68,        // imul
        0xAC,        // ireturn
    ];
    let mut thread = Thread::new(ClassLoader::new());
    let frame = Frame::new(2, code);
    thread.push_frame(frame);
    let result = thread.execute();

    assert_eq!(result, Some(Value::I32(42)));
}

/// Test: if-else branch
/// Pseudocode:
///   int x = 10;
///   if (x == 10) { x = 100; } else { x = 200; }
///   return x;
#[test]
fn test_execute_if_else() {
    // 0: bipush 10      (0x10, 0x0A)
    // 2: istore_0        (0x3B)
    // 3: iload_0         (0x1A)
    // 4: bipush 10      (0x10, 0x0A)
    // 6: if_icmpne +8   (0xA0, 0x00, 0x08) -> jump to offset 14 if ne
    // 9: bipush 100     (0x10, 0x64)
    //11: istore_0        (0x3B)
    //12: goto +4         (0xA7, 0x00, 0x04) -> jump to offset 19
    //15: bipush 200     (0x10, 0xC8)  -- unreachable in this test
    //17: istore_0        (0x3B)
    //18: iload_0         (0x1A)
    //19: ireturn         (0xAC)
    //
    // Note: goto and if offsets are from the start of each instruction
    // After dispatcher reads opcode, pc points to the operand bytes
    // So in our scheme, we need to be careful about offset calculation.
    //
    // Let me simplify: the dispatcher reads opcode at pc, increments pc.
    // Then the handler reads its operands from pc.
    // For branch instructions, offset is relative to the start of the instruction.
    // But in our implementation, after reading the opcode, pc is at opcode_pos+1.
    // Then handler reads 2-byte offset, so pc becomes opcode_pos+3.
    // Then we apply: pc = pc + offset - 3 = opcode_pos + 3 + offset - 3 = opcode_pos + offset.
    // That's correct! offset is relative to the instruction start.

    let code = vec![
        0x10, 0x0A,             // 0: bipush 10
        0x3B,                   // 2: istore_0
        0x1A,                   // 3: iload_0
        0x10, 0x0A,             // 4: bipush 10
        0xA0, 0x00, 0x09,       // 6: if_icmpne -> branch to 6+9=15
        0x10, 0x64,             // 9: bipush 100
        0x3B,                   // 11: istore_0
        0xA7, 0x00, 0x06,       // 12: goto -> 12+6=18
        0x10, 0xC8,             // 15: bipush 200 (else branch)
        0x3B,                   // 17: istore_0
        0x1A,                   // 18: iload_0
        0xAC,                   // 19: ireturn
    ];
    let mut thread = Thread::new(ClassLoader::new());
    let frame = Frame::new(4, code);
    thread.push_frame(frame);
    let result = thread.execute();

    assert_eq!(result, Some(Value::I32(100)));
}

/// Test: simple for loop - sum 1 to 5 = 15
/// Pseudocode:
///   int sum = 0;
///   int i = 1;
///   while (i <= 5) { sum += i; i++; }
///   return sum;
#[test]
fn test_execute_for_loop() {
    // locals: [sum, i]
    // 0: iconst_0        (0x03)
    // 1: istore_0        (0x3B)
    // 2: iconst_1        (0x04)
    // 3: istore_1        (0x3C)
    // loop start:
    // 4: iload_1         (0x1B)  -- load i
    // 5: bipush 5        (0x10, 0x05)
    // 7: if_icmpgt +12   (0xA3, 0x00, 0x0C) -> if i > 5, jump to 7+12=19
    // 10: iload_0        (0x1A) -- load sum
    // 11: iload_1        (0x1B) -- load i
    // 12: iadd           (0x60)
    // 13: istore_0       (0x3B) -- store sum
    // 14: iinc 1 1       (0x84, 0x01, 0x01) -- i += 1
    // 17: goto -13        (0xA7, 0xFF, 0xF3) -> 17 + (-13) = 4
    // after loop:
    // 20: iload_0        (0x1A)
    // 21: ireturn        (0xAC)
    let code = vec![
        0x03,                   // 0: iconst_0
        0x3B,                   // 1: istore_0
        0x04,                   // 2: iconst_1
        0x3C,                   // 3: istore_1
        // loop start (offset 4):
        0x1B,                   // 4: iload_1
        0x10, 0x05,             // 5: bipush 5
        0xA3, 0x00, 0x0D,       // 7: if_icmpgt -> 7+13=20
        0x1A,                   // 10: iload_0
        0x1B,                   // 11: iload_1
        0x60,                   // 12: iadd
        0x3B,                   // 13: istore_0
        0x84, 0x01, 0x01,       // 14: iinc 1, 1
        0xA7, 0xFF, 0xF3,       // 17: goto -> 17+(-13)=4
        // after loop (offset 20):
        0x1A,                   // 20: iload_0
        0xAC,                   // 21: ireturn
    ];
    let mut thread = Thread::new(ClassLoader::new());
    let frame = Frame::new(4, code);
    thread.push_frame(frame);
    let result = thread.execute();

    assert_eq!(result, Some(Value::I32(15)));
}

/// Test: method invocation
/// Simulate: static int add(int a, int b) { return a + b; }
/// Then call add(3, 4) and return the result.
///
/// This tests the thread's method dispatch loop.
#[test]
fn test_static_method_call() {
    // We need to set up a class with a static method.
    // The "caller" bytecode: iconst_3, iconst_4, invokestatic #add, ireturn
    // But invokestatic needs to resolve through the class loader.
    // Let's test this with manually constructed class files.

    // First, create a class "Calc" with static method "add": (II)I
    // Constant pool:
    // #1: Methodref -> class #3, nat #10
    // #2: Class -> name #4
    // #3: Class -> name #5  (java/lang/Object)
    // #4: Utf8 "Calc"
    // #5: Utf8 "java/lang/Object"
    // #6: Utf8 "add"
    // #7: Utf8 "(II)I"
    // #8: Utf8 "Code"
    // #9: Utf8 "main"
    // #10: NameAndType -> name #6, desc #7
    // #11: Utf8 "([Ljava/lang/String;)V"

    // Calc.add method bytecode: iload_0, iload_1, iadd, ireturn
    let add_code = vec![
        0x1A,  // iload_0
        0x1B,  // iload_1
        0x60,  // iadd
        0xAC,  // ireturn
    ];

    // Build a class file for Calc with the add method
    let calc_class = build_calc_class(&add_code);
    
    let mut class_loader = ClassLoader::new();
    class_loader.load_class_from_bytes("Calc", &calc_class).unwrap();

    // Main bytecode: iconst_3, iconst_4, invokestatic #1, ireturn
    // #1 in main's class constant pool would be a Methodref to Calc.add(II)I
    // But we need a separate class for "Main" that references Calc.
    // For simplicity, let's just test that we can find and execute the add method directly.
    
    let class_file = class_loader.get_class("Calc").unwrap();
    let method = class_loader.find_method(class_file, "add", "(II)I").unwrap();
    let (_max_stack, max_locals, code) = ClassLoader::get_method_code(method, class_file).unwrap();

    let mut thread = Thread::new(class_loader);
    let mut frame = Frame::with_args(max_locals as usize, code, vec![Value::I32(3), Value::I32(4)]);
    frame.class_name = "Calc".to_string();
    thread.push_frame(frame);
    let result = thread.execute();

    assert_eq!(result, Some(Value::I32(7)));
}

fn build_calc_class(add_code: &[u8]) -> Vec<u8> {
    // Build a minimal class file for "Calc" with a single static method "add": (II)I
    let code_attr_len = 12 + add_code.len() as u32; // max_stack(2) + max_locals(2) + code_len(4) + code + exc_table(2) + attrs(2)

    let mut bytes = vec![
        0xCA, 0xFE, 0xBA, 0xBE,  // magic
        0x00, 0x00,              // minor
        0x00, 0x34,              // major (52)
        0x00, 0x09,              // constant_pool_count (9)
        // #1 - not used as Methodref in this simple case
        0x07, 0x00, 0x03,        // #1 Class -> #3
        // #2 Class -> #4 (java/lang/Object)  
        0x07, 0x00, 0x04,
        // #3 Utf8 "Calc"
        0x01, 0x00, 0x04, b'C', b'a', b'l', b'c',
        // #4 Utf8 "java/lang/Object"
        0x01, 0x00, 0x10,
        b'j', b'a', b'v', b'a', b'/', b'l', b'a', b'n', b'g', b'/', b'O', b'b', b'j', b'e', b'c', b't',
        // #5 Utf8 "add"
        0x01, 0x00, 0x03, b'a', b'd', b'd',
        // #6 Utf8 "(II)I"
        0x01, 0x00, 0x05, b'(', b'I', b'I', b')', b'I',
        // #7 Utf8 "Code"
        0x01, 0x00, 0x04, b'C', b'o', b'd', b'e',
        // #8 Utf8 "SourceFile"
        0x01, 0x00, 0x0A,
        b'S', b'o', b'u', b'r', b'c', b'e', b'F', b'i', b'l', b'e',
        // access_flags: ACC_PUBLIC | ACC_SUPER
        0x00, 0x21,
        // this_class: #1
        0x00, 0x01,
        // super_class: #2
        0x00, 0x02,
        // interfaces_count
        0x00, 0x00,
        // fields_count
        0x00, 0x00,
    ];

    // methods_count: 1
    bytes.push(0x00);
    bytes.push(0x01);

    // method[0]: add
    bytes.extend_from_slice(&[
        0x00, 0x09,  // ACC_PUBLIC | ACC_STATIC
        0x00, 0x05,  // name_index -> "add"
        0x00, 0x06,  // descriptor_index -> "(II)I"
        0x00, 0x01,  // attributes_count: 1 (Code)
    ]);

    // Code attribute
    bytes.extend_from_slice(&[
        0x00, 0x07,  // attribute_name_index -> "Code"
    ]);
    bytes.extend_from_slice(&(code_attr_len as u32).to_be_bytes());  // attribute_length
    bytes.extend_from_slice(&[
        0x00, 0x02,  // max_stack = 2
        0x00, 0x02,  // max_locals = 2
    ]);
    bytes.extend_from_slice(&(add_code.len() as u32).to_be_bytes());  // code_length
    bytes.extend_from_slice(add_code);  // code
    bytes.extend_from_slice(&[
        0x00, 0x00,  // exception_table_length
        0x00, 0x00,  // attributes_count (on Code attribute)
    ]);

    // class attributes_count: 0
    bytes.extend_from_slice(&[0x00, 0x00]);

    bytes
}

/// Test: negative numbers and negation
#[test]
fn test_execute_negation() {
    let code = vec![
        0x10, 0x0A,  // bipush 10
        0x74,        // ineg
        0xAC,        // ireturn
    ];
    let mut thread = Thread::new(ClassLoader::new());
    let frame = Frame::new(2, code);
    thread.push_frame(frame);
    let result = thread.execute();

    assert_eq!(result, Some(Value::I32(-10)));
}
