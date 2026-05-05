pub mod constant_pool;
pub mod models;
pub mod parser;

pub use constant_pool::ConstantPoolEntry;
pub use models::{AttributeData, AttributeInfo, ClassFile, CodeAttribute, ConstantPool, ExceptionTableEntry, FieldInfo, MethodInfo};
pub use parser::parse_class_file;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidMagic(u32),
    UnexpectedEof,
    InvalidConstantPoolTag(u8),
    InvalidUtf8,
    InvalidAttribute(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidMagic(v) => {
                write!(f, "Invalid magic number: 0x{:08X}, expected 0xCAFEBABE", v)
            }
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidConstantPoolTag(tag) => {
                write!(f, "Invalid constant pool tag: {}", tag)
            }
            ParseError::InvalidUtf8 => write!(f, "Invalid UTF-8 in constant pool"),
            ParseError::InvalidAttribute(msg) => write!(f, "Invalid attribute: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}
