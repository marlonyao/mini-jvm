use crate::runtime::thread::{Thread, ExecutionResult};

/// pop: pop top operand stack value
pub fn pop_op(thread: &mut Thread) -> ExecutionResult {
    thread.current_frame().pop();
    ExecutionResult::Return(None)
}

/// dup: duplicate top operand stack value
pub fn dup(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let val = frame.pop();
    frame.push(val.clone());
    frame.push(val);
    ExecutionResult::Return(None)
}

/// swap: swap top two operand stack values
pub fn swap(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let v1 = frame.pop();
    let v2 = frame.pop();
    frame.push(v1);
    frame.push(v2);
    ExecutionResult::Return(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::{Frame, Value};
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread() -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        thread.push_frame(Frame::new(4, vec![]));
        thread
    }

    #[test]
    fn test_pop() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(1));
        t.current_frame().push(Value::I32(2));
        pop_op(&mut t);
        assert_eq!(t.current_frame().operand_stack.len(), 1);
    }

    #[test]
    fn test_dup() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(42));
        dup(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 42);
        assert_eq!(t.current_frame().pop_i32(), 42);
    }

    #[test]
    fn test_swap() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(1));
        t.current_frame().push(Value::I32(2));
        swap(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 1);
        assert_eq!(t.current_frame().pop_i32(), 2);
    }
}
