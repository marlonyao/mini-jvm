use crate::runtime::frame::Value;
use crate::runtime::thread::{Thread, ExecutionResult};

/// ifeq: branch if int == 0
pub fn ifeq(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let val = frame.pop_i32();
    if val == 0 {
        // offset is relative to the start of this instruction (opcode position)
        // pc is now at opcode_pos + 3 (opcode + 2 byte offset)
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// ifne: branch if int != 0
pub fn ifne(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let val = frame.pop_i32();
    if val != 0 {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// if_icmpeq: branch if ints equal
pub fn if_icmpeq(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    if a == b {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// if_icmpne: branch if ints not equal
pub fn if_icmpne(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    if a != b {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// if_icmplt: branch if int a < b
pub fn if_icmplt(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    if a < b {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// if_icmpge: branch if int a >= b
pub fn if_icmpge(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    if a >= b {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// if_icmpgt: branch if int a > b
pub fn if_icmpgt(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    if a > b {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// if_icmple: branch if int a <= b
pub fn if_icmple(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let b = frame.pop_i32();
    let a = frame.pop_i32();
    if a <= b {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// goto: unconditional branch
pub fn goto(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    frame.pc = (frame.pc as i32 + offset - 3) as usize;
    ExecutionResult::Return(None)
}

/// ifnull: branch if null
pub fn ifnull(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let val = frame.pop();
    if val == crate::runtime::frame::Value::Null {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

/// ifnonnull: branch if not null
pub fn ifnonnull(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let offset = frame.read_i16() as i32;
    let val = frame.pop();
    if val != crate::runtime::frame::Value::Null {
        frame.pc = (frame.pc as i32 + offset - 3) as usize;
    }
    ExecutionResult::Return(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::Frame;
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread(code: Vec<u8>) -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        thread.push_frame(Frame::new(4, code));
        thread
    }

    #[test]
    fn test_ifeq_branch_taken() {
        // ifeq offset=5, value=0 -> should branch
        let code = vec![0x00, 0x05]; // offset bytes (after opcode, which dispatch already consumed)
        let mut t = make_thread(code);
        t.current_frame().push(Value::I32(0));
        t.current_frame().pc = 0;
        ifeq(&mut t);
        // pc should have been adjusted by offset
    }

    #[test]
    fn test_ifeq_branch_not_taken() {
        let code = vec![0x00, 0x05];
        let mut t = make_thread(code);
        t.current_frame().push(Value::I32(1));
        t.current_frame().pc = 0;
        ifeq(&mut t);
        assert_eq!(t.current_frame().pc, 2); // past the offset bytes
    }

    #[test]
    fn test_if_icmpeq_taken() {
        let code = vec![0x00, 0x0A]; // offset 10
        let mut t = make_thread(code);
        t.current_frame().push(Value::I32(5));
        t.current_frame().push(Value::I32(5));
        t.current_frame().pc = 0;
        if_icmpeq(&mut t);
    }

    #[test]
    fn test_goto() {
        let code = vec![0x00, 0x05]; // offset 5
        let mut t = make_thread(code);
        t.current_frame().pc = 0;
        goto(&mut t);
        // pc was at 0, read 2 bytes -> pc=2, then offset=5: new_pc = 2 + 5 - 3 = 4
        assert_eq!(t.current_frame().pc, 4);
    }
}
