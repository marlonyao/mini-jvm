use crate::runtime::frame::Value;
use crate::runtime::thread::{Thread, ExecutionResult};

/// nop: do nothing
pub fn nop(_thread: &mut Thread) -> ExecutionResult {
    ExecutionResult::Return(None)
}

pub fn iconst_m1(thread: &mut Thread) -> ExecutionResult {
    thread.current_frame().push(Value::I32(-1));
    ExecutionResult::Return(None)
}

pub fn iconst(thread: &mut Thread, val: i32) -> ExecutionResult {
    thread.current_frame().push(Value::I32(val));
    ExecutionResult::Return(None)
}

/// bipush: push a byte as an integer
pub fn bipush(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let val = frame.read_i8() as i32;
    frame.push(Value::I32(val));
    ExecutionResult::Return(None)
}

/// sipush: push a short as an integer
pub fn sipush(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let val = frame.read_i16() as i32;
    frame.push(Value::I32(val));
    ExecutionResult::Return(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::Frame;
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread(code: Vec<u8>) -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        let frame = Frame::new(4, code);
        thread.push_frame(frame);
        thread
    }

    #[test]
    fn test_iconst() {
        let mut t = make_thread(vec![0x03]); // iconst_0
        iconst(&mut t, 0);
        assert_eq!(t.current_frame().pop_i32(), 0);
    }

    #[test]
    fn test_bipush() {
        let mut t = make_thread(vec![0x0A]); // just the operand byte (10)
        t.current_frame().pc = 0;
        bipush(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 10);
    }

    #[test]
    fn test_sipush() {
        let mut t = make_thread(vec![0x00, 0x64]); // operand bytes for 100
        t.current_frame().pc = 0;
        sipush(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 100);
    }

    #[test]
    fn test_bipush_negative() {
        let mut t = make_thread(vec![0xFF]); // operand byte (-1)
        t.current_frame().pc = 0;
        bipush(&mut t);
        assert_eq!(t.current_frame().pop_i32(), -1);
    }
}
