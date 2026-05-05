use crate::classfile::{ClassFile, ConstantPoolEntry, MethodInfo, AttributeData};

/// Loads and caches class files from the filesystem.
pub struct ClassLoader {
    /// Cache of loaded classes: class name -> ClassFile
    classes: std::collections::HashMap<String, ClassFile>,
    /// Search paths for class files
    classpath: Vec<String>,
}

impl ClassLoader {
    pub fn new() -> Self {
        ClassLoader {
            classes: std::collections::HashMap::new(),
            classpath: vec![".".to_string()],
        }
    }

    pub fn with_classpath(classpath: Vec<String>) -> Self {
        ClassLoader {
            classes: std::collections::HashMap::new(),
            classpath,
        }
    }

    /// Load a class by name (e.g., "java/lang/Object").
    pub fn load_class(&mut self, name: &str) -> Result<&ClassFile, String> {
        if self.classes.contains_key(name) {
            return Ok(self.classes.get(name).unwrap());
        }

        let filename = format!("{}.class", name);
        for dir in &self.classpath {
            let path = format!("{}/{}", dir, filename);
            if let Ok(bytes) = std::fs::read(&path) {
                let class_file = crate::classfile::parse_class_file(&bytes)
                    .map_err(|e| format!("Failed to parse {}: {}", path, e))?;
                self.classes.insert(name.to_string(), class_file);
                return Ok(self.classes.get(name).unwrap());
            }
        }

        Err(format!("Class not found: {}", name))
    }

    /// Load a class directly from bytes (useful for testing).
    pub fn load_class_from_bytes(&mut self, name: &str, bytes: &[u8]) -> Result<&ClassFile, String> {
        let class_file = crate::classfile::parse_class_file(bytes)
            .map_err(|e| format!("Failed to parse class: {}", e))?;
        self.classes.insert(name.to_string(), class_file);
        Ok(self.classes.get(name).unwrap())
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassFile> {
        self.classes.get(name)
    }

    /// Resolve the class name from a class constant pool index.
    pub fn resolve_class_name(&self, class_file: &ClassFile, class_index: u16) -> Option<String> {
        match class_file.constant_pool.get(class_index)? {
            ConstantPoolEntry::Class { name_index } => {
                match class_file.constant_pool.get(*name_index)? {
                    ConstantPoolEntry::Utf8(name) => Some(name.clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Resolve a method name and descriptor from a method ref index.
    pub fn resolve_method_ref(&self, class_file: &ClassFile, index: u16) -> Option<(String, String, String)> {
        match class_file.constant_pool.get(index)? {
            ConstantPoolEntry::Methodref { class_index, name_and_type_index } => {
                let class_name = self.resolve_class_name(class_file, *class_index)?;
                match class_file.constant_pool.get(*name_and_type_index)? {
                    ConstantPoolEntry::NameAndType { name_index, descriptor_index } => {
                        let method_name = match class_file.constant_pool.get(*name_index)? {
                            ConstantPoolEntry::Utf8(n) => n.clone(),
                            _ => return None,
                        };
                        let descriptor = match class_file.constant_pool.get(*descriptor_index)? {
                            ConstantPoolEntry::Utf8(d) => d.clone(),
                            _ => return None,
                        };
                        Some((class_name, method_name, descriptor))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Find a method in a class file by name and descriptor.
    pub fn find_method<'a>(&self, class_file: &'a ClassFile, name: &str, descriptor: &str) -> Option<&'a MethodInfo> {
        for method in &class_file.methods {
            let method_name = match class_file.constant_pool.get(method.name_index) {
                Some(ConstantPoolEntry::Utf8(n)) => n.clone(),
                _ => continue,
            };
            let method_desc = match class_file.constant_pool.get(method.descriptor_index) {
                Some(ConstantPoolEntry::Utf8(d)) => d.clone(),
                _ => continue,
            };
            if method_name == name && method_desc == descriptor {
                return Some(method);
            }
        }
        None
    }

    /// Get the Code attribute's bytecode from a method.
    pub fn get_method_code(method: &MethodInfo, _class_file: &ClassFile) -> Option<(u16, u16, Vec<u8>)> {
        for attr in &method.attributes {
            match &attr.data {
                AttributeData::Code(code) => {
                    return Some((code.max_stack, code.max_locals, code.code.clone()));
                }
                _ => continue,
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_class_from_bytes() {
        let mut loader = ClassLoader::new();
        let bytes = vec![
            0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x34,
            0x00, 0x0D,
            0x0A, 0x00, 0x03, 0x00, 0x0A,
            0x07, 0x00, 0x04,
            0x07, 0x00, 0x05,
            0x01, 0x00, 0x05, b'H', b'e', b'l', b'l', b'o',
            0x01, 0x00, 0x10,
            b'j', b'a', b'v', b'a', b'/', b'l', b'a', b'n', b'g', b'/', b'O', b'b', b'j', b'e', b'c', b't',
            0x01, 0x00, 0x06, b'<', b'i', b'n', b'i', b't', b'>',
            0x01, 0x00, 0x03, b'(', b')', b'V',
            0x01, 0x00, 0x04, b'C', b'o', b'd', b'e',
            0x01, 0x00, 0x0F,
            b'L', b'i', b'n', b'e', b'N', b'u', b'm', b'b', b'e', b'r', b'T', b'a', b'b', b'l', b'e',
            0x0C, 0x00, 0x06, 0x00, 0x07,
            0x01, 0x00, 0x0A,
            b'S', b'o', b'u', b'r', b'c', b'e', b'F', b'i', b'l', b'e',
            0x01, 0x00, 0x0A,
            b'H', b'e', b'l', b'l', b'o', b'.', b'j', b'a', b'v', b'a',
            0x00, 0x21, 0x00, 0x02, 0x00, 0x03,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,
            0x00, 0x01, 0x00, 0x06, 0x00, 0x07, 0x00, 0x01,
            0x00, 0x08,
            0x00, 0x00, 0x00, 0x1D,
            0x00, 0x01, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x05,
            0x2A, 0xB7, 0x00, 0x01, 0xB1,
            0x00, 0x00,
            0x00, 0x01,
            0x00, 0x09,
            0x00, 0x00, 0x00, 0x06,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x01,
            0x00, 0x0B,
            0x00, 0x00, 0x00, 0x02,
            0x00, 0x0C,
        ];
        loader.load_class_from_bytes("Hello", &bytes).unwrap();
        assert!(loader.get_class("Hello").is_some());
    }
}
