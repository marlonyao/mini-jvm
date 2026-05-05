use crate::runtime::frame::Value;
use crate::runtime::thread::{Thread, ExecutionResult};

/// iadd: add two integers
pub fn iadd(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    frame.push(Value::I32(a.wrapping_add(b)));
    ExecutionResult::Return(None)
}

/// isub: subtract integers
pub fn isub(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    frame.push(Value::I32(a.wrapping_sub(b)));
    ExecutionResult::Return(None)
}

/// imul: multiply integers
pub fn imul(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    frame.push(Value::I32(a.wrapping_mul(b)));
    ExecutionResult::Return(None)
}

/// idiv: divide integers
pub fn idiv(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    frame.push(Value::I32(a.wrapping_div(b)));
    ExecutionResult::Return(None)
}

/// irem: integer remainder
pub fn irem(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    frame.push(Value::I32(a.wrapping_rem(b)));
    ExecutionResult::Return(None)
}

/// ineg: negate integer
pub fn ineg(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let a = frame.pop_i32();
    frame.push(Value::I32(-a));
    ExecutionResult::Return(None)
}

/// iinc: increment local variable by constant
pub fn iinc(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u1() as usize;
    let const_val = frame.read_i8() as i32;
    let old = frame.locals[index].as_i32();
    frame.locals[index] = Value::I32(old.wrapping_add(const_val));
    ExecutionResult::Return(None)
}

/// ladd: add two longs
pub fn ladd(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i64();
    let a = frame.pop_i64();
    frame.push(Value::I64(a.wrapping_add(b)));
    ExecutionResult::Return(None)
}

/// lsub: subtract longs
pub fn lsub(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i64();
    let a = frame.pop_i64();
    frame.push(Value::I64(a.wrapping_sub(b)));
    ExecutionResult::Return(None)
}

/// lmul: multiply longs
pub fn lmul(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i64();
    let a = frame.pop_i64();
    frame.push(Value::I64(a.wrapping_mul(b)));
    ExecutionResult::Return(None)
}

/// ldiv: divide longs
pub fn ldiv(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let b = frame.pop_i64();
    let a = frame.pop_i64();
    frame.push(Value::I64(a.wrapping_div(b)));
    ExecutionResult::Return(None)
}

/// lneg: negate long
pub fn lneg(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let a = frame.pop_i64();
    frame.push(Value::I64(-a));
    ExecutionResult::Return(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::Frame;
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread() -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        thread.push_frame(Frame::new(4, vec![]));
        thread
    }

    #[test]
    fn test_iadd() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(10));
        t.current_frame().push(Value::I32(20));
        iadd(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 30);
    }

    #[test]
    fn test_isub() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(20));
        t.current_frame().push(Value::I32(7));
        isub(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 13);
    }

    #[test]
    fn test_imul() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(6));
        t.current_frame().push(Value::I32(7));
        imul(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 42);
    }

    #[test]
    fn test_idiv() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(100));
        t.current_frame().push(Value::I32(3));
        idiv(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 33);
    }

    #[test]
    fn test_irem() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(100));
        t.current_frame().push(Value::I32(3));
        irem(&mut t);
        assert_eq!(t.current_frame().pop_i32(), 1);
    }

    #[test]
    fn test_ineg() {
        let mut t = make_thread();
        t.current_frame().push(Value::I32(42));
        ineg(&mut t);
        assert_eq!(t.current_frame().pop_i32(), -42);
    }

    #[test]
    fn test_iinc() {
        let mut t = make_thread();
        t.current_frame().locals[1] = Value::I32(10);
        // Manually set up: iinc index=1, const=5
        t.current_frame().code = vec![0x01, 0x05];
        t.current_frame().pc = 0;
        iinc(&mut t);
        assert_eq!(t.current_frame().locals[1].as_i32(), 15);
    }

    #[test]
    fn test_ladd() {
        let mut t = make_thread();
        t.current_frame().push(Value::I64(100));
        t.current_frame().push(Value::I64(200));
        ladd(&mut t);
        assert_eq!(t.current_frame().pop_i64(), 300);
    }
}
