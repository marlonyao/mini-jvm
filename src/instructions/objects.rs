use crate::runtime::frame::Value;
use crate::runtime::thread::{Thread, ExecutionResult};

/// ldc: push item from constant pool (index = u1)
pub fn ldc(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u1() as u16;
    resolve_and_push_cp_item(thread, index)
}

/// ldc_w: push item from constant pool (index = u2)
pub fn ldc_w(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();
    resolve_and_push_cp_item(thread, index)
}

fn resolve_and_push_cp_item(thread: &mut Thread, index: u16) -> ExecutionResult {
    let class_name = thread.current_frame().class_name.clone();
    let cp_item = {
        let class_file = thread.class_loader.get_class(&class_name)
            .expect(&format!("Class not found: {}", class_name));
        class_file.constant_pool.get(index)
            .expect(&format!("Constant pool index {} not found", index))
            .clone()
    };

    match cp_item {
        crate::classfile::ConstantPoolEntry::Integer(v) => {
            thread.current_frame().push(Value::I32(v));
        }
        crate::classfile::ConstantPoolEntry::Float(v) => {
            thread.current_frame().push(Value::F32(v));
        }
        crate::classfile::ConstantPoolEntry::String { string_index } => {
            // Resolve the string from utf8 constant pool entry
            let class_file = thread.class_loader.get_class(&class_name).unwrap();
            match class_file.constant_pool.get(string_index) {
                Some(crate::classfile::ConstantPoolEntry::Utf8(s)) => {
                    // For now, we represent strings as a special object on the heap
                    let idx = thread.heap.alloc_string(s.clone());
                    thread.current_frame().push(Value::Object(idx));
                }
                _ => panic!("Invalid string constant at index {}", string_index),
            }
        }
        _ => panic!("Unsupported ldc item type at index {}", index),
    }
    ExecutionResult::Continue
}

/// getstatic: get static field from class
pub fn getstatic(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();

    // Resolve the field reference from constant pool
    let class_name = thread.current_frame().class_name.clone();
    let (field_class, field_name, _field_desc) = {
        let class_file = thread.class_loader.get_class(&class_name).unwrap();
        match class_file.constant_pool.get(index) {
            Some(crate::classfile::ConstantPoolEntry::Fieldref { class_index, name_and_type_index }) => {
                let fc = thread.class_loader.resolve_class_name(class_file, *class_index)
                    .unwrap();
                match class_file.constant_pool.get(*name_and_type_index) {
                    Some(crate::classfile::ConstantPoolEntry::NameAndType { name_index, descriptor_index: _ }) => {
                        match class_file.constant_pool.get(*name_index) {
                            Some(crate::classfile::ConstantPoolEntry::Utf8(n)) => (fc, n.clone(), String::new()),
                            _ => panic!("Invalid field name"),
                        }
                    }
                    _ => panic!("Invalid NameAndType for field ref"),
                }
            }
            _ => panic!("Expected Fieldref at index {}", index),
        }
    };

    // Check if this is System.out — our native stub
    if field_class == "java/lang/System" && field_name == "out" {
        let idx = thread.heap.alloc("java/io/PrintStream".to_string());
        thread.current_frame().push(Value::Object(idx));
    } else {
        // Look up static field from heap
        let key = format!("{}.{}", field_class, field_name);
        let val = thread.heap.get_static_field(&key);
        thread.current_frame().push(val);
    }
    ExecutionResult::Continue
}

/// putstatic: set static field in class
pub fn putstatic(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();
    let val = frame.pop();

    let class_name = thread.current_frame().class_name.clone();
    let (field_class, field_name) = {
        let class_file = thread.class_loader.get_class(&class_name).unwrap();
        match class_file.constant_pool.get(index) {
            Some(crate::classfile::ConstantPoolEntry::Fieldref { class_index, name_and_type_index }) => {
                let fc = thread.class_loader.resolve_class_name(class_file, *class_index).unwrap();
                match class_file.constant_pool.get(*name_and_type_index) {
                    Some(crate::classfile::ConstantPoolEntry::NameAndType { name_index, descriptor_index: _ }) => {
                        match class_file.constant_pool.get(*name_index) {
                            Some(crate::classfile::ConstantPoolEntry::Utf8(n)) => (fc, n.clone()),
                            _ => panic!("Invalid field name"),
                        }
                    }
                    _ => panic!("Invalid NameAndType for putstatic"),
                }
            }
            _ => panic!("Expected Fieldref at index {}", index),
        }
    };

    let key = format!("{}.{}", field_class, field_name);
    thread.heap.set_static_field(key, val);
    ExecutionResult::Continue
}

/// new: create new object
pub fn new_op(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();

    let class_name = thread.current_frame().class_name.clone();
    let target_class = {
        let class_file = thread.class_loader.get_class(&class_name).unwrap();
        thread.class_loader.resolve_class_name(class_file, index)
            .unwrap()
    };

    let obj_idx = thread.heap.alloc(target_class);
    thread.current_frame().push(Value::Object(obj_idx));
    ExecutionResult::Continue
}

/// getfield: get field from object
/// Stack: ..., objectref -> ..., value
pub fn getfield(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();

    let class_name = thread.current_frame().class_name.clone();
    let field_name = {
        let class_file = thread.class_loader.get_class(&class_name).unwrap();
        match class_file.constant_pool.get(index) {
            Some(crate::classfile::ConstantPoolEntry::Fieldref { class_index: _, name_and_type_index }) => {
                match class_file.constant_pool.get(*name_and_type_index) {
                    Some(crate::classfile::ConstantPoolEntry::NameAndType { name_index, descriptor_index: _ }) => {
                        match class_file.constant_pool.get(*name_index) {
                            Some(crate::classfile::ConstantPoolEntry::Utf8(n)) => n.clone(),
                            _ => panic!("Invalid field name"),
                        }
                    }
                    _ => panic!("Invalid NameAndType for getfield"),
                }
            }
            _ => panic!("Expected Fieldref at index {}", index),
        }
    };

    let obj_ref = thread.current_frame().pop();
    match obj_ref {
        Value::Object(idx) => {
            let obj = thread.heap.get(idx).unwrap();
            let val = obj.fields.get(&field_name).cloned().unwrap_or(Value::I32(0));
            thread.current_frame().push(val);
        }
        Value::Null => panic!("NullPointerException: getfield {}", field_name),
        _ => panic!("getfield: expected object reference"),
    }
    ExecutionResult::Continue
}

/// putfield: set field in object
/// Stack: ..., objectref, value -> ...
pub fn putfield(thread: &mut Thread) -> ExecutionResult {
    let frame = thread.current_frame();
    let index = frame.read_u2();

    let class_name = thread.current_frame().class_name.clone();
    let field_name = {
        let class_file = thread.class_loader.get_class(&class_name).unwrap();
        match class_file.constant_pool.get(index) {
            Some(crate::classfile::ConstantPoolEntry::Fieldref { class_index: _, name_and_type_index }) => {
                match class_file.constant_pool.get(*name_and_type_index) {
                    Some(crate::classfile::ConstantPoolEntry::NameAndType { name_index, descriptor_index: _ }) => {
                        match class_file.constant_pool.get(*name_index) {
                            Some(crate::classfile::ConstantPoolEntry::Utf8(n)) => n.clone(),
                            _ => panic!("Invalid field name"),
                        }
                    }
                    _ => panic!("Invalid NameAndType for putfield"),
                }
            }
            _ => panic!("Expected Fieldref at index {}", index),
        }
    };

    let value = thread.current_frame().pop();
    let obj_ref = thread.current_frame().pop();
    match obj_ref {
        Value::Object(idx) => {
            thread.heap.get_mut(idx).unwrap().fields.insert(field_name, value);
        }
        Value::Null => panic!("NullPointerException: putfield {}", field_name),
        _ => panic!("putfield: expected object reference"),
    }
    ExecutionResult::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::Frame;
    use crate::runtime::class_loader::ClassLoader;

    fn make_thread(code: Vec<u8>) -> Thread {
        let mut thread = Thread::new(ClassLoader::new());
        let mut frame = Frame::new(4, code);
        frame.class_name = "Test".to_string();
        thread.push_frame(frame);
        thread
    }

    #[test]
    fn test_new_object() {
        let mut t = make_thread(vec![0x00, 0x01]);
        // We need to set up a class with a proper constant pool
        // For simplicity, just test the heap allocation directly
        let idx = t.heap.alloc("MyClass".to_string());
        let obj = t.heap.get(idx).unwrap();
        assert_eq!(obj.class_name, "MyClass");
    }
}
