use crate::runtime::frame::Value;
use crate::runtime::thread::{Thread, ExecutionResult};

/// Array types for newarray instruction
const T_BOOLEAN: u8 = 4;
const T_CHAR: u8 = 5;
const T_FLOAT: u8 = 6;
const T_DOUBLE: u8 = 7;
const T_BYTE: u8 = 8;
const T_SHORT: u8 = 9;
const T_INT: u8 = 10;
const T_LONG: u8 = 11;

/// newarray: create new array of primitive type
/// Stack: ..., count -> ..., arrayref
pub fn newarray(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let atype = frame.read_u1();
    let count = frame.pop_i32() as usize;

    // Create array as a heap object with indexed fields
    let class_name = match atype {
        T_INT => "[I",
        T_LONG => "[J",
        T_FLOAT => "[F",
        T_DOUBLE => "[D",
        T_BYTE => "[B",
        T_SHORT => "[S",
        T_CHAR => "[C",
        T_BOOLEAN => "[Z",
        _ => panic!("Unknown array type: {}", atype),
    };

    let idx = thread.heap.alloc_array(class_name.to_string(), count);
    thread.current_frame().push(Value::Object(idx));
    ExecutionResult::Continue
}

/// anewarray: create new array of reference type
/// Stack: ..., count -> ..., arrayref
pub fn anewarray(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let _index = frame.read_u2(); // class index (we don't strictly need it)
    let count = frame.pop_i32() as usize;

    let idx = thread.heap.alloc_array("[Ljava/lang/Object;".to_string(), count);
    thread.current_frame().push(Value::Object(idx));
    ExecutionResult::Continue
}

/// iaload: load int from array
/// Stack: ..., arrayref, index -> ..., value
pub fn iaload(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.pop_i32() as usize;
    let arrayref = frame.pop();

    match arrayref {
        Value::Object(idx) => {
            let val = thread.heap.get_array_int(idx, index);
            thread.current_frame().push(Value::I32(val));
        }
        Value::Null => panic!("NullPointerException: iaload on null"),
        _ => panic!("iaload: expected array reference"),
    }
    ExecutionResult::Continue
}

/// iastore: store int into array
/// Stack: ..., arrayref, index, value -> ...
pub fn iastore(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let value = frame.pop_i32();
    let index = frame.pop_i32() as usize;
    let arrayref = frame.pop();

    match arrayref {
        Value::Object(idx) => {
            thread.heap.set_array_int(idx, index, value);
        }
        Value::Null => panic!("NullPointerException: iastore on null"),
        _ => panic!("iastore: expected array reference"),
    }
    ExecutionResult::Continue
}

/// aaload: load reference from array
/// Stack: ..., arrayref, index -> ..., value
pub fn aaload(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.pop_i32() as usize;
    let arrayref = frame.pop();

    match arrayref {
        Value::Object(idx) => {
            let val = thread.heap.get_array_ref(idx, index);
            thread.current_frame().push(val);
        }
        Value::Null => panic!("NullPointerException: aaload on null"),
        _ => panic!("aaload: expected array reference"),
    }
    ExecutionResult::Continue
}

/// aastore: store reference into array
/// Stack: ..., arrayref, index, value -> ...
pub fn aastore(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let value = frame.pop();
    let index = frame.pop_i32() as usize;
    let arrayref = frame.pop();

    match arrayref {
        Value::Object(idx) => {
            thread.heap.set_array_ref(idx, index, value);
        }
        Value::Null => panic!("NullPointerException: aastore on null"),
        _ => panic!("aastore: expected array reference"),
    }
    ExecutionResult::Continue
}

/// arraylength: get length of array
/// Stack: ..., arrayref -> ..., length
pub fn arraylength(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let arrayref = frame.pop();

    match arrayref {
        Value::Object(idx) => {
            let len = thread.heap.get_array_length(idx);
            thread.current_frame().push(Value::I32(len as i32));
        }
        Value::Null => panic!("NullPointerException: arraylength on null"),
        _ => panic!("arraylength: expected array reference"),
    }
    ExecutionResult::Continue
}

/// baload: load byte/boolean from array
pub fn baload(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.pop_i32() as usize;
    let arrayref = frame.pop();

    match arrayref {
        Value::Object(idx) => {
            let val = thread.heap.get_array_int(idx, index);
            thread.current_frame().push(Value::I32(val));
        }
        Value::Null => panic!("NullPointerException: baload on null"),
        _ => panic!("baload: expected array reference"),
    }
    ExecutionResult::Continue
}

/// bastore: store byte/boolean into array
pub fn bastore(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let value = frame.pop_i32();
    let index = frame.pop_i32() as usize;
    let arrayref = frame.pop();

    match arrayref {
        Value::Object(idx) => {
            thread.heap.set_array_int(idx, index, value);
        }
        Value::Null => panic!("NullPointerException: bastore on null"),
        _ => panic!("bastore: expected array reference"),
    }
    ExecutionResult::Continue
}

/// caload: load char from array
pub fn caload(thread: &mut Thread) -> ExecutionResult {
    // Same as iaload for our purposes
    iaload(thread)
}

/// castore: store char into array
pub fn castore(thread: &mut Thread) -> ExecutionResult {
    iastore(thread)
}

/// saload: load short from array
pub fn saload(thread: &mut Thread) -> ExecutionResult {
    iaload(thread)
}

/// sastore: store short into array
pub fn sastore(thread: &mut Thread) -> ExecutionResult {
    iastore(thread)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::Frame;
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread() -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        let mut frame = Frame::new(4, vec![]);
        frame.class_name = "Test".to_string();
        thread.push_frame(frame);
        thread
    }

    #[test]
    fn test_newarray_and_iastore_iaload() {
        let mut t = make_thread();
        // Create int[3]
        t.current_frame().push(Value::I32(3));
        t.current_frame().code = vec![T_INT]; // atype
        t.current_frame().pc = 0;
        newarray(&mut t);

        // Should have array ref on stack
        let arrayref = t.current_frame().pop();
        assert!(matches!(arrayref, Value::Object(_)));

        // Store value 42 at index 1
        t.current_frame().push(arrayref.clone());
        t.current_frame().push(Value::I32(1));
        t.current_frame().push(Value::I32(42));
        iastore(&mut t);

        // Load value back
        t.current_frame().push(arrayref);
        t.current_frame().push(Value::I32(1));
        iaload(&mut t);

        assert_eq!(t.current_frame().pop_i32(), 42);
    }

    #[test]
    fn test_arraylength() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(10));
        t.current_frame().code = vec![T_INT];
        t.current_frame().pc = 0;
        newarray(&mut t);

        // Check length
        let arrayref = t.current_frame().pop();
        t.current_frame().push(arrayref);
        arraylength(&mut t);

        assert_eq!(t.current_frame().pop_i32(), 10);
    }
}
