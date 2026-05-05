use crate::runtime::thread::{Thread, ExecutionResult};

/// invokevirtual: invoke instance method
/// Stack: ..., objectref, [arg1, [arg2 ...]] -> ...
pub fn invokevirtual(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();

    let class_name_str = thread.current_frame().class_name.clone();
    let (target_class, method_name, descriptor) = {
        let class_file = thread.class_loader.get_class(&class_name_str)
            .unwrap_or_else(|| panic!("Class not found: {}", class_name_str));
        thread.class_loader.resolve_method_ref(class_file, index)
            .unwrap_or_else(|| panic!("Cannot resolve method ref #{}", index))
    };

    // Count arguments from descriptor to know how many values to pop
    let arg_count = count_args(&descriptor);
    let total_count = arg_count + 1; // +1 for objectref

    // Pop args + objectref from stack (in reverse order)
    let mut args = Vec::new();
    for _ in 0..total_count {
        args.push(thread.current_frame().pop());
    }
    args.reverse(); // Now args[0] = objectref, args[1..] = method args

    ExecutionResult::Invoke {
        class_name: target_class,
        method_name,
        descriptor,
        args,
    }
}

/// invokespecial: invoke constructor / super / private method
/// Stack: ..., objectref, [arg1, [arg2 ...]] -> ...
pub fn invokespecial(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();

    let class_name_str = thread.current_frame().class_name.clone();
    let (target_class, method_name, descriptor) = {
        let class_file = thread.class_loader.get_class(&class_name_str)
            .unwrap_or_else(|| panic!("Class not found: {}", class_name_str));
        thread.class_loader.resolve_method_ref(class_file, index)
            .unwrap_or_else(|| panic!("Cannot resolve method ref #{}", index))
    };

    // For Object.<init>(), just skip - no real code to execute
    if target_class == "java/lang/Object" && method_name == "<init>" {
        let _obj = thread.current_frame().pop();
        return ExecutionResult::Continue;
    }

    let arg_count = count_args(&descriptor);
    let total_count = arg_count + 1;
    let mut args = Vec::new();
    for _ in 0..total_count {
        args.push(thread.current_frame().pop());
    }
    args.reverse();

    ExecutionResult::Invoke {
        class_name: target_class,
        method_name,
        descriptor,
        args,
    }
}

/// invokestatic: invoke static method
/// Stack: ..., [arg1, [arg2 ...]] -> ...
pub fn invokestatic(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();

    let class_name_str = thread.current_frame().class_name.clone();
    let (target_class, method_name, descriptor) = {
        let class_file = thread.class_loader.get_class(&class_name_str)
            .unwrap_or_else(|| panic!("Class not found: {}", class_name_str));
        thread.class_loader.resolve_method_ref(class_file, index)
            .unwrap_or_else(|| panic!("Cannot resolve method ref #{}", index))
    };

    let arg_count = count_args(&descriptor);
    let mut args = Vec::new();
    for _ in 0..arg_count {
        args.push(thread.current_frame().pop());
    }
    args.reverse();

    ExecutionResult::Invoke {
        class_name: target_class,
        method_name,
        descriptor,
        args,
    }
}

/// ireturn: return int from method
pub fn ireturn(thread: &mut Thread) -> ExecutionResult {
    let val = thread.current_frame().pop();
    ExecutionResult::Return(Some(val))
}

/// areturn: return reference from method
pub fn areturn(thread: &mut Thread) -> ExecutionResult {
    let val = thread.current_frame().pop();
    ExecutionResult::Return(Some(val))
}

/// return: return void from method
pub fn r#return(_thread: &mut Thread) -> ExecutionResult {
    ExecutionResult::Return(None)
}

/// Count the number of arguments from a method descriptor.
/// Example: "(II)I" -> 2, "(Ljava/lang/String;)V" -> 1, "()V" -> 0
fn count_args(descriptor: &str) -> usize {
    let mut count = 0;
    let mut chars = descriptor.chars().peekable();
    
    // Skip '('
    if chars.peek() == Some(&'(') {
        chars.next();
    }
    
    while let Some(&c) = chars.peek() {
        match c {
            ')' => break,
            'I' | 'Z' | 'B' | 'C' | 'S' | 'F' => {
                count += 1;
                chars.next();
            }
            'J' | 'D' => {
                count += 1; // long and double take 1 slot in our Value enum
                chars.next();
            }
            'L' => {
                count += 1;
                // Skip to ';'
                while let Some(ch) = chars.next() {
                    if ch == ';' { break; }
                }
            }
            '[' => {
                chars.next(); // skip '['
                // The next char is the base type
                continue;
            }
            _ => {
                chars.next();
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::{Frame, Value};
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread(code: Vec<u8>) -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        let mut frame = Frame::new(4, code);
        frame.class_name = "Test".to_string();
        thread.push_frame(frame);
        thread
    }

    #[test]
    fn test_ireturn() {
        let mut t = make_thread(vec![]);
        t.current_frame().push(Value::I32(42));
        let result = ireturn(&mut t);
        assert_eq!(result, ExecutionResult::Return(Some(Value::I32(42))));
    }

    #[test]
    fn test_return() {
        let mut t = make_thread(vec![]);
        let result = r#return(&mut t);
        assert_eq!(result, ExecutionResult::Return(None));
    }

    #[test]
    fn test_count_args() {
        assert_eq!(count_args("()V"), 0);
        assert_eq!(count_args("(I)V"), 1);
        assert_eq!(count_args("(II)I"), 2);
        assert_eq!(count_args("(Ljava/lang/String;)V"), 1);
        assert_eq!(count_args("(Ljava/lang/String;I)V"), 2);
    }
}
