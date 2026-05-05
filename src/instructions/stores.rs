use crate::runtime::thread::{Thread, ExecutionResult};

/// istore_<n>: store int into local variable
pub fn istore_n(thread: &mut Thread, index: usize) -> ExecutionResult {
    let frame = thread.current_frame();
    let val = frame.pop();
    frame.locals[index] = val;
    ExecutionResult::Return(None)
}

/// istore: store int into local variable (with index operand)
pub fn istore(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u1() as usize;
    let val = frame.pop();
    frame.locals[index] = val;
    ExecutionResult::Return(None)
}

/// astore_<n>: store reference into local variable
pub fn astore_n(thread: &mut Thread, index: usize) -> ExecutionResult {
    let frame = thread.current_frame();
    let val = frame.pop();
    frame.locals[index] = val;
    ExecutionResult::Return(None)
}

/// astore: store reference into local variable (with index operand)
pub fn astore(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u1() as usize;
    let val = frame.pop();
    frame.locals[index] = val;
    ExecutionResult::Return(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::{Frame, Value};
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread(code: Vec<u8>) -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        let frame = Frame::new(4, code);
        thread.push_frame(frame);
        thread
    }

    #[test]
    fn test_istore_n() {
        let mut t = make_thread(vec![]);
        t.current_frame().push(Value::I32(42));
        istore_n(&mut t, 1);
        assert_eq!(t.current_frame().locals[1], Value::I32(42));
    }

    #[test]
    fn test_istore() {
        let mut t = make_thread(vec![0x02]); // index = 2
        t.current_frame().push(Value::I32(99));
        t.current_frame().pc = 0;
        istore(&mut t);
        assert_eq!(t.current_frame().locals[2], Value::I32(99));
    }
}
