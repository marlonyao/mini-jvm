use crate::runtime::thread::{Thread, ExecutionResult};

/// iload_<n>: load int from local variable
pub fn iload_n(thread: &mut Thread, index: usize) -> ExecutionResult {
    let frame = thread.current_frame();
    let val = frame.locals[index].clone();
    frame.push(val);
    ExecutionResult::Return(None)
}

/// iload: load int from local variable (with index operand)
pub fn iload(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u1() as usize;
    let val = frame.locals[index].clone();
    frame.push(val);
    ExecutionResult::Return(None)
}

/// aload_<n>: load reference from local variable
pub fn aload_n(thread: &mut Thread, index: usize) -> ExecutionResult {
    let frame = thread.current_frame();
    let val = frame.locals[index].clone();
    frame.push(val);
    ExecutionResult::Return(None)
}

/// aload: load reference from local variable (with index operand)
pub fn aload(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u1() as usize;
    let val = frame.locals[index].clone();
    frame.push(val);
    ExecutionResult::Return(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::{Frame, Value};
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread_with_locals(locals: Vec<Value>) -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        let mut frame = Frame::new(locals.len(), vec![]);
        for (i, v) in locals.into_iter().enumerate() {
            frame.locals[i] = v;
        }
        thread.push_frame(frame);
        thread
    }

    #[test]
    fn test_iload_n() {
        let mut t = make_thread_with_locals(vec![Value::I32(42), Value::I32(0)]);
        iload_n(&mut t, 0);
        assert_eq!(t.current_frame().pop_i32(), 42);
    }

    #[test]
    fn test_iload() {
        let mut t = make_thread_with_locals(vec![Value::I32(0), Value::I32(99)]);
        // Set up code: iload 1
        t.current_frame().code = vec![0x01];
        t.current_frame().pc = 0;
        iload(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 99);
    }
}
