use crate::runtime::thread::{Thread, ExecutionResult};

/// invokevirtual: invoke instance method
pub fn invokevirtual(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();
    
    // Resolve method reference from current class's constant pool
    // For now, we need to find the class and method
    // The thread will handle creating the new frame
    
    // TODO: Full implementation requires resolving the method ref and dispatching
    // For Phase 2, return a stub that prints the method being called
    let _ = index;
    ExecutionResult::Return(None)
}

/// invokespecial: invoke instance initialization / super / private method
pub fn invokespecial(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();
    // For <init> methods (constructors), we need to:
    // 1. Pop objectref and args
    // 2. Find the method
    // 3. Create a new frame and execute it
    // For now, we just skip the method call for Object.<init>
    let _ = index;
    // Pop objectref (the 'this' pointer)
    let _obj = frame.pop();
    ExecutionResult::Return(None)
}

/// invokestatic: invoke static method
pub fn invokestatic(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();
    
    // Resolve the method reference
    // We need to look up the current class's constant pool
    // This requires access to the class file - for now, use a simplified approach
    let _ = index;
    ExecutionResult::Return(None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::{Frame, Value};
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread(code: Vec<u8>) -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        thread.push_frame(Frame::new(4, code));
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
}
