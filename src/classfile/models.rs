use std::fmt;
use crate::classfile::ConstantPoolEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

impl fmt::Display for ClassFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ClassFile {{")?;
        writeln!(f, "  magic: 0x{:08X}", self.magic)?;
        writeln!(f, "  minor_version: {}", self.minor_version)?;
        writeln!(f, "  major_version: {}", self.major_version)?;
        writeln!(
            f,
            "  constant_pool_count: {}",
            self.constant_pool.entries.len() + 1
        )?;
        for (i, entry) in self.constant_pool.entries.iter().enumerate() {
            if let Some(e) = entry {
                writeln!(f, "  #{}: {:?}", i + 1, e)?;
            }
        }
        writeln!(f, "  access_flags: 0x{:04X}", self.access_flags)?;
        writeln!(f, "  this_class: {}", self.this_class)?;
        writeln!(f, "  super_class: {}", self.super_class)?;
        writeln!(f, "  interfaces_count: {}", self.interfaces.len())?;
        for (i, iface) in self.interfaces.iter().enumerate() {
            writeln!(f, "    interface[{}]: {}", i, iface)?;
        }
        writeln!(f, "  fields_count: {}", self.fields.len())?;
        for field in &self.fields {
            writeln!(f, "    {:?}", field)?;
        }
        writeln!(f, "  methods_count: {}", self.methods.len())?;
        for method in &self.methods {
            writeln!(f, "    {:?}", method)?;
        }
        writeln!(f, "  attributes_count: {}", self.attributes.len())?;
        for attr in &self.attributes {
            writeln!(f, "    {:?}", attr)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantPool {
    pub entries: Vec<Option<ConstantPoolEntry>>,
}

impl ConstantPool {
    pub fn get(&self, index: u16) -> Option<&ConstantPoolEntry> {
        let idx = index as usize;
        if idx == 0 || idx > self.entries.len() {
            return None;
        }
        self.entries[idx - 1].as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeInfo {
    pub name_index: u16,
    pub data: AttributeData,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeData {
    Code(CodeAttribute),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntry>,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}
