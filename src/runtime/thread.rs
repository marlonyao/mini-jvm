use crate::runtime::frame::Frame;
use crate::runtime::heap::Heap;
use crate::runtime::class_loader::ClassLoader;

/// Execution result from running a frame.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
    /// Continue executing the next instruction.
    Continue,
    /// Method returned normally, optionally with a value.
    Return(Option<crate::runtime::frame::Value>),
    /// Invoking another method — need to create a new frame.
    Invoke { class_name: String, method_name: String, descriptor: String, args: Vec<crate::runtime::frame::Value> },
}

/// A thread of execution with its own call stack.
pub struct Thread {
    pub stack: Vec<Frame>,
    pub heap: Heap,
    pub class_loader: ClassLoader,
    /// JIT 编译代码执行期间持有的对象引用索引
    /// JIT 代码分配对象时 push，方法返回前 pop
    pub jit_roots: Vec<usize>,
}

/// Check if a method is a native method we handle with a stub.
fn is_native_stub(class_name: &str, method_name: &str, descriptor: &str) -> bool {
    match (class_name, method_name) {
        ("java/io/PrintStream", "println") => true,
        ("java/io/PrintStream", "print") => true,
        _ => false,
    }
}

/// Execute a native method stub.
fn execute_native_stub(
    thread: &mut Thread,
    class_name: &str,
    method_name: &str,
    descriptor: &str,
    args: Vec<crate::runtime::frame::Value>,
) -> Option<crate::runtime::frame::Value> {
    match (class_name, method_name, descriptor) {
        ("java/io/PrintStream", "println", "(I)V") => {
            // println(int): pop the int value and print it
            if args.len() >= 2 {
                // args[0] = objectref (PrintStream), args[1] = int value
                println!("{}", args[1].as_i32());
            }
            None
        }
        ("java/io/PrintStream", "println", "(Ljava/lang/String;)V") => {
            // println(String): pop the string and print it
            if args.len() >= 2 {
                match &args[1] {
                    crate::runtime::frame::Value::Object(idx) => {
                        if let Some(s) = thread.heap.get_string(*idx) {
                            println!("{}", s);
                        } else {
                            println!("<non-string object>");
                        }
                    }
                    _ => println!("{:?}", args[1]),
                }
            }
            None
        }
        ("java/io/PrintStream", "println", "()V") => {
            // println(): just print newline
            println!();
            None
        }
        ("java/io/PrintStream", "println", "(J)V") => {
            if args.len() >= 2 {
                println!("{}", args[1].as_i64());
            }
            None
        }
        ("java/io/PrintStream", "print", "(I)V") => {
            if args.len() >= 2 {
                print!("{}", args[1].as_i32());
            }
            None
        }
        ("java/io/PrintStream", "print", "(Ljava/lang/String;)V") => {
            if args.len() >= 2 {
                match &args[1] {
                    crate::runtime::frame::Value::Object(idx) => {
                        if let Some(s) = thread.heap.get_string(*idx) {
                            print!("{}", s);
                        }
                    }
                    _ => print!("{:?}", args[1]),
                }
            }
            None
        }
        _ => {
            eprintln!("Warning: unhandled native stub {}.{}{}", class_name, method_name, descriptor);
            None
        }
    }
}

impl Thread {
    pub fn new(class_loader: ClassLoader) -> Self {
        Thread {
            stack: Vec::new(),
            heap: Heap::new(),
            class_loader,
            jit_roots: Vec::new(),
        }
    }

    /// Register an object reference as a JIT root (prevents GC from collecting it)
    pub fn jit_root_push(&mut self, obj_idx: usize) {
        self.jit_roots.push(obj_idx);
    }

    /// Remove the last JIT root (e.g., when JIT method returns)
    pub fn jit_root_pop(&mut self) -> Option<usize> {
        self.jit_roots.pop()
    }

    /// Get current JIT roots for GC scanning
    pub fn jit_roots(&self) -> &[usize] {
        &self.jit_roots
    }

    /// Push a frame onto the call stack.
    pub fn push_frame(&mut self, frame: Frame) {
        self.stack.push(frame);
    }

    /// Pop a frame from the call stack.
    pub fn pop_frame(&mut self) -> Option<Frame> {
        self.stack.pop()
    }

    /// Get the current (top) frame.
    pub fn current_frame(&mut self) -> &mut Frame {
        self.stack.last_mut().expect("Call stack underflow")
    }

    /// Execute the current frame until it returns.
    /// Returns the return value (if any).
    pub fn execute(&mut self) -> Option<crate::runtime::frame::Value> {
        loop {
            let frame = self.current_frame();
            if frame.pc >= frame.code.len() {
                break;
            }

            let opcode = frame.code[frame.pc];
            frame.pc += 1;

            let result = crate::instructions::execute_instruction(self, opcode);

            match result {
                ExecutionResult::Continue => {}
                ExecutionResult::Return(val) => {
                    self.pop_frame();
                    // If there's a caller frame, push the return value
                    if let Some(caller) = self.stack.last_mut() {
                        if let Some(v) = &val {
                            caller.push(v.clone());
                        }
                    }
                    // If no more frames, this is the top-level return
                    if self.stack.is_empty() {
                        return val;
                    }
                    // Otherwise, continue executing the caller frame
                }
                ExecutionResult::Invoke { class_name, method_name, descriptor, args } => {
                    // Check for native method stubs first
                    if is_native_stub(&class_name, &method_name, &descriptor) {
                        let return_val = execute_native_stub(self, &class_name, &method_name, &descriptor, args);
                        // Push return value onto caller's operand stack
                        if let Some(v) = &return_val {
                            self.current_frame().push(v.clone());
                        }
                        continue;
                    }

                    // Resolve the method and create a new frame
                    let class = self.class_loader.get_class(&class_name)
                        .unwrap_or_else(|| panic!("Class not found: {}", class_name));
                    let method = self.class_loader.find_method(class, &method_name, &descriptor)
                        .unwrap_or_else(|| panic!("Method not found: {}.{}{}", class_name, method_name, descriptor));
                    let (_max_stack, max_locals, code) = ClassLoader::get_method_code(method, class)
                        .unwrap_or_else(|| panic!("No Code attribute for {}.{}{}", class_name, method_name, descriptor));
                    
                    let mut new_frame = Frame::with_args(max_locals as usize, code, args);
                    new_frame.class_name = class_name.clone();
                    self.push_frame(new_frame);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_new() {
        let thread = Thread::new(ClassLoader::new());
        assert!(thread.stack.is_empty());
    }
}
