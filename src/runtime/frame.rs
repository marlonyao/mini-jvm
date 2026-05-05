/// Value types that can be stored on the operand stack and in local variables.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Object(usize), // index into heap
    Null,
    ReturnAddress(usize), // for ret instruction
}

impl Value {
    pub fn as_i32(&self) -> i32 {
        match self {
            Value::I32(v) => *v,
            _ => panic!("Expected I32, got {:?}", self),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Value::I64(v) => *v,
            _ => panic!("Expected I64, got {:?}", self),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Value::F32(v) => *v,
            _ => panic!("Expected F32, got {:?}", self),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Value::F64(v) => *v,
            _ => panic!("Expected F64, got {:?}", self),
        }
    }

    pub fn as_object(&self) -> usize {
        match self {
            Value::Object(idx) => *idx,
            Value::Null => panic!("Null pointer dereference"),
            _ => panic!("Expected Object, got {:?}", self),
        }
    }
}

/// A stack frame for method execution.
pub struct Frame {
    /// Local variable table
    pub locals: Vec<Value>,
    /// Operand stack
    pub operand_stack: Vec<Value>,
    /// Index into the constant pool of the class this frame belongs to
    pub constant_pool_idx: usize,
    /// The bytecode being executed
    pub code: Vec<u8>,
    /// Program counter (index into code)
    pub pc: usize,
}

impl Frame {
    pub fn new(max_locals: usize, code: Vec<u8>) -> Self {
        let mut locals = Vec::with_capacity(max_locals);
        for _ in 0..max_locals {
            locals.push(Value::I32(0));
        }
        Frame {
            locals,
            operand_stack: Vec::new(),
            constant_pool_idx: 0,
            code,
            pc: 0,
        }
    }

    pub fn with_args(max_locals: usize, code: Vec<u8>, args: Vec<Value>) -> Self {
        let mut frame = Self::new(max_locals, code);
        for (i, arg) in args.into_iter().enumerate() {
            if i < frame.locals.len() {
                frame.locals[i] = arg;
            }
        }
        frame
    }

    // --- Operand stack operations ---

    pub fn push(&mut self, value: Value) {
        self.operand_stack.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.operand_stack.pop().expect("Operand stack underflow")
    }

    pub fn pop_i32(&mut self) -> i32 {
        self.pop().as_i32()
    }

    pub fn pop_i64(&mut self) -> i64 {
        self.pop().as_i64()
    }

    pub fn pop_f32(&mut self) -> f32 {
        self.pop().as_f32()
    }

    pub fn pop_f64(&mut self) -> f64 {
        self.pop().as_f64()
    }

    // --- Bytecode reading ---

    pub fn read_u1(&mut self) -> u8 {
        let val = self.code[self.pc];
        self.pc += 1;
        val
    }

    pub fn read_i8(&mut self) -> i8 {
        self.read_u1() as i8
    }

    pub fn read_u2(&mut self) -> u16 {
        let high = self.code[self.pc] as u16;
        let low = self.code[self.pc + 1] as u16;
        self.pc += 2;
        (high << 8) | low
    }

    pub fn read_i16(&mut self) -> i16 {
        self.read_u2() as i16
    }

    pub fn read_i32(&mut self) -> i32 {
        let b1 = self.code[self.pc] as i32;
        let b2 = self.code[self.pc + 1] as i32;
        let b3 = self.code[self.pc + 2] as i32;
        let b4 = self.code[self.pc + 3] as i32;
        self.pc += 4;
        (b1 << 24) | (b2 << 16) | (b3 << 8) | b4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_new() {
        let frame = Frame::new(4, vec![0x10, 0x05]);
        assert_eq!(frame.locals.len(), 4);
        assert!(frame.operand_stack.is_empty());
        assert_eq!(frame.code, vec![0x10, 0x05]);
    }

    #[test]
    fn test_frame_with_args() {
        let args = vec![Value::I32(42), Value::I32(10)];
        let frame = Frame::with_args(4, vec![], args);
        assert_eq!(frame.locals[0], Value::I32(42));
        assert_eq!(frame.locals[1], Value::I32(10));
        assert_eq!(frame.locals[2], Value::I32(0)); // default
    }

    #[test]
    fn test_operand_stack() {
        let mut frame = Frame::new(1, vec![]);
        frame.push(Value::I32(1));
        frame.push(Value::I32(2));
        assert_eq!(frame.pop_i32(), 2);
        assert_eq!(frame.pop_i32(), 1);
    }

    #[test]
    fn test_read_u1() {
        let mut frame = Frame::new(1, vec![0xCA, 0xFE]);
        assert_eq!(frame.read_u1(), 0xCA);
        assert_eq!(frame.read_u1(), 0xFE);
    }

    #[test]
    fn test_read_i16() {
        let mut frame = Frame::new(1, vec![0xFF, 0xFE]); // -2
        assert_eq!(frame.read_i16(), -2);
    }
}
