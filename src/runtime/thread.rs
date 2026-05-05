use crate::runtime::frame::Frame;
use crate::runtime::heap::Heap;
use crate::runtime::class_loader::ClassLoader;

/// Execution result from running a frame.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
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
}

impl Thread {
    pub fn new(class_loader: ClassLoader) -> Self {
        Thread {
            stack: Vec::new(),
            heap: Heap::new(),
            class_loader,
        }
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
                // Ran past the end of bytecode
                break;
            }

            let opcode = frame.code[frame.pc];
            frame.pc += 1;

            let result = crate::instructions::execute_instruction(self, opcode);

            match result {
                ExecutionResult::Return(val) => {
                    self.pop_frame();
                    // If there's a caller frame, push the return value
                    if let Some(caller) = self.stack.last_mut() {
                        if let Some(v) = &val {
                            caller.push(v.clone());
                        }
                    }
                    return val;
                }
                ExecutionResult::Invoke { class_name, method_name, descriptor, args } => {
                    // Resolve the method and create a new frame
                    let class = self.class_loader.get_class(&class_name)
                        .expect(&format!("Class not found: {}", class_name));
                    let method = self.class_loader.find_method(class, &method_name, &descriptor)
                        .expect(&format!("Method not found: {}.{}{}", class_name, method_name, descriptor));
                    let (_max_stack, max_locals, code) = ClassLoader::get_method_code(method, class)
                        .expect(&format!("No Code attribute for {}.{}{}", class_name, method_name, descriptor));
                    
                    let new_frame = Frame::with_args(max_locals as usize, code, args);
                    self.push_frame(new_frame);
                    // Continue the loop to execute the new frame
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
