use crate::classfile::constant_pool::ConstantPoolEntry;
use crate::classfile::models::*;
use crate::classfile::ParseError;
use std::convert::TryInto;

pub struct ByteReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    pub fn u1(&mut self) -> Result<u8, ParseError> {
        if self.pos >= self.bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let val = self.bytes[self.pos];
        self.pos += 1;
        Ok(val)
    }

    pub fn u2(&mut self) -> Result<u16, ParseError> {
        if self.pos + 2 > self.bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let val = u16::from_be_bytes(self.bytes[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;
        Ok(val)
    }

    pub fn u4(&mut self) -> Result<u32, ParseError> {
        if self.pos + 4 > self.bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let val = u32::from_be_bytes(self.bytes[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(val)
    }

    pub fn u8(&mut self) -> Result<u64, ParseError> {
        if self.pos + 8 > self.bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let val = u64::from_be_bytes(self.bytes[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(val)
    }

    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        if self.pos + n > self.bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let val = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(val)
    }
}

pub fn parse_class_file(bytes: &[u8]) -> Result<ClassFile, ParseError> {
    let mut r = ByteReader::new(bytes);

    let magic = r.u4()?;
    if magic != 0xCAFEBABE {
        return Err(ParseError::InvalidMagic(magic));
    }

    let minor_version = r.u2()?;
    let major_version = r.u2()?;
    let constant_pool_count = r.u2()?;
    let constant_pool = parse_constant_pool(&mut r, constant_pool_count)?;

    let access_flags = r.u2()?;
    let this_class = r.u2()?;
    let super_class = r.u2()?;
    let interfaces_count = r.u2()?;
    let mut interfaces = Vec::with_capacity(interfaces_count as usize);
    for _ in 0..interfaces_count {
        interfaces.push(r.u2()?);
    }

    let fields = parse_fields(&mut r, &constant_pool)?;
    let methods = parse_methods(&mut r, &constant_pool)?;
    let attributes = parse_attributes(&mut r, &constant_pool)?;

    Ok(ClassFile {
        magic,
        minor_version,
        major_version,
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    })
}

fn parse_constant_pool(r: &mut ByteReader, count: u16) -> Result<ConstantPool, ParseError> {
    let mut entries = Vec::with_capacity(count.saturating_sub(1) as usize);
    let mut i = 1u16;
    while i < count {
        let tag = r.u1()?;
        let entry = match tag {
            1 => {
                let length = r.u2()?;
                let bytes = r.bytes(length as usize)?;
                let s = String::from_utf8(bytes.to_vec()).map_err(|_| ParseError::InvalidUtf8)?;
                ConstantPoolEntry::Utf8(s)
            }
            3 => ConstantPoolEntry::Integer(r.u4()? as i32),
            4 => ConstantPoolEntry::Float(f32::from_bits(r.u4()?)),
            5 => {
                let val = r.u8()?;
                entries.push(Some(ConstantPoolEntry::Long(val as i64)));
                entries.push(None);
                i = i.wrapping_add(2);
                continue;
            }
            6 => {
                let val = r.u8()?;
                entries.push(Some(ConstantPoolEntry::Double(f64::from_bits(val))));
                entries.push(None);
                i = i.wrapping_add(2);
                continue;
            }
            7 => ConstantPoolEntry::Class { name_index: r.u2()? },
            8 => ConstantPoolEntry::String { string_index: r.u2()? },
            9 => ConstantPoolEntry::Fieldref {
                class_index: r.u2()?,
                name_and_type_index: r.u2()?,
            },
            10 => ConstantPoolEntry::Methodref {
                class_index: r.u2()?,
                name_and_type_index: r.u2()?,
            },
            11 => ConstantPoolEntry::InterfaceMethodref {
                class_index: r.u2()?,
                name_and_type_index: r.u2()?,
            },
            12 => ConstantPoolEntry::NameAndType {
                name_index: r.u2()?,
                descriptor_index: r.u2()?,
            },
            15 => ConstantPoolEntry::MethodHandle {
                reference_kind: r.u1()?,
                reference_index: r.u2()?,
            },
            16 => ConstantPoolEntry::MethodType {
                descriptor_index: r.u2()?,
            },
            18 => ConstantPoolEntry::InvokeDynamic {
                bootstrap_method_attr_index: r.u2()?,
                name_and_type_index: r.u2()?,
            },
            19 => ConstantPoolEntry::Module { name_index: r.u2()? },
            20 => ConstantPoolEntry::Package { name_index: r.u2()? },
            17 => ConstantPoolEntry::Dynamic {
                bootstrap_method_attr_index: r.u2()?,
                name_and_type_index: r.u2()?,
            },
            _ => return Err(ParseError::InvalidConstantPoolTag(tag)),
        };
        entries.push(Some(entry));
        i = i.wrapping_add(1);
    }
    Ok(ConstantPool { entries })
}

fn parse_fields(r: &mut ByteReader, cp: &ConstantPool) -> Result<Vec<FieldInfo>, ParseError> {
    let count = r.u2()?;
    let mut fields = Vec::with_capacity(count as usize);
    for _ in 0..count {
        fields.push(FieldInfo {
            access_flags: r.u2()?,
            name_index: r.u2()?,
            descriptor_index: r.u2()?,
            attributes: parse_attributes(r, cp)?,
        });
    }
    Ok(fields)
}

fn parse_methods(r: &mut ByteReader, cp: &ConstantPool) -> Result<Vec<MethodInfo>, ParseError> {
    let count = r.u2()?;
    let mut methods = Vec::with_capacity(count as usize);
    for _ in 0..count {
        methods.push(MethodInfo {
            access_flags: r.u2()?,
            name_index: r.u2()?,
            descriptor_index: r.u2()?,
            attributes: parse_attributes(r, cp)?,
        });
    }
    Ok(methods)
}

fn parse_attributes(
    r: &mut ByteReader,
    cp: &ConstantPool,
) -> Result<Vec<AttributeInfo>, ParseError> {
    let count = r.u2()?;
    let mut attrs = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name_index = r.u2()?;
        let attribute_length = r.u4()?;
        let info = r.bytes(attribute_length as usize)?.to_vec();

        let attr = if let Some(ConstantPoolEntry::Utf8(name)) = cp.get(name_index) {
            match name.as_str() {
                "Code" => {
                    let mut cr = ByteReader::new(&info);
                    let max_stack = cr.u2()?;
                    let max_locals = cr.u2()?;
                    let code_length = cr.u4()?;
                    let code = cr.bytes(code_length as usize)?.to_vec();
                    let exception_table_length = cr.u2()?;
                    let mut exception_table = Vec::with_capacity(exception_table_length as usize);
                    for _ in 0..exception_table_length {
                        exception_table.push(ExceptionTableEntry {
                            start_pc: cr.u2()?,
                            end_pc: cr.u2()?,
                            handler_pc: cr.u2()?,
                            catch_type: cr.u2()?,
                        });
                    }
                    let attributes = parse_attributes(&mut cr, cp)?;
                    AttributeInfo {
                        name_index,
                        data: AttributeData::Code(CodeAttribute {
                            max_stack,
                            max_locals,
                            code,
                            exception_table,
                            attributes,
                        }),
                    }
                }
                _ => AttributeInfo {
                    name_index,
                    data: AttributeData::Raw(info),
                },
            }
        } else {
            AttributeInfo {
                name_index,
                data: AttributeData::Raw(info),
            }
        };
        attrs.push(attr);
    }
    Ok(attrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_class_bytes() -> Vec<u8> {
        vec![
            0xCA, 0xFE, 0xBA, 0xBE, // magic
            0x00, 0x00,             // minor_version
            0x00, 0x34,             // major_version (52)
            0x00, 0x0D,             // constant_pool_count (13)
            // #1 Methodref
            0x0A, 0x00, 0x03, 0x00, 0x0A,
            // #2 Class
            0x07, 0x00, 0x04,
            // #3 Class
            0x07, 0x00, 0x05,
            // #4 Utf8 "Hello"
            0x01, 0x00, 0x05, b'H', b'e', b'l', b'l', b'o',
            // #5 Utf8 "java/lang/Object"
            0x01, 0x00, 0x10,
            b'j', b'a', b'v', b'a', b'/', b'l', b'a', b'n', b'g', b'/', b'O', b'b', b'j', b'e', b'c', b't',
            // #6 Utf8 "<init>"
            0x01, 0x00, 0x06, b'<', b'i', b'n', b'i', b't', b'>',
            // #7 Utf8 "()V"
            0x01, 0x00, 0x03, b'(', b')', b'V',
            // #8 Utf8 "Code"
            0x01, 0x00, 0x04, b'C', b'o', b'd', b'e',
            // #9 Utf8 "LineNumberTable"
            0x01, 0x00, 0x0F,
            b'L', b'i', b'n', b'e', b'N', b'u', b'm', b'b', b'e', b'r', b'T', b'a', b'b', b'l', b'e',
            // #10 NameAndType
            0x0C, 0x00, 0x06, 0x00, 0x07,
            // #11 Utf8 "SourceFile"
            0x01, 0x00, 0x0A,
            b'S', b'o', b'u', b'r', b'c', b'e', b'F', b'i', b'l', b'e',
            // #12 Utf8 "Hello.java"
            0x01, 0x00, 0x0A,
            b'H', b'e', b'l', b'l', b'o', b'.', b'j', b'a', b'v', b'a',
            // access_flags
            0x00, 0x21,
            // this_class
            0x00, 0x02,
            // super_class
            0x00, 0x03,
            // interfaces_count
            0x00, 0x00,
            // fields_count
            0x00, 0x00,
            // methods_count
            0x00, 0x01,
            // method[0]
            0x00, 0x01, // access_flags
            0x00, 0x06, // name_index
            0x00, 0x07, // descriptor_index
            0x00, 0x01, // attributes_count
            // attribute[0] Code
            0x00, 0x08, // name_index
            0x00, 0x00, 0x00, 0x1D, // attribute_length
            0x00, 0x01, // max_stack
            0x00, 0x01, // max_locals
            0x00, 0x00, 0x00, 0x05, // code_length
            0x2A, 0xB7, 0x00, 0x01, 0xB1, // code
            0x00, 0x00, // exception_table_length
            0x00, 0x01, // attributes_count
            // attribute[0] LineNumberTable
            0x00, 0x09, // name_index
            0x00, 0x00, 0x00, 0x06, // attribute_length
            0x00, 0x01, // line_number_table_length
            0x00, 0x00, // start_pc
            0x00, 0x01, // line_number
            // class attributes_count
            0x00, 0x01,
            // attribute[0] SourceFile
            0x00, 0x0B, // name_index
            0x00, 0x00, 0x00, 0x02, // attribute_length
            0x00, 0x0C, // sourcefile_index
        ]
    }

    #[test]
    fn test_parse_minimal_class() {
        let bytes = minimal_class_bytes();
        let class = parse_class_file(&bytes).expect("parse should succeed");
        assert_eq!(class.magic, 0xCAFEBABE);
        assert_eq!(class.major_version, 52);
        assert_eq!(class.constant_pool.entries.len(), 12);
        assert_eq!(class.methods.len(), 1);
        assert_eq!(class.methods[0].name_index, 6);
        assert_eq!(class.methods[0].descriptor_index, 7);
        assert_eq!(class.methods[0].attributes.len(), 1);

        let attr = &class.methods[0].attributes[0];
        assert_eq!(attr.name_index, 8);
        match &attr.data {
            AttributeData::Code(code) => {
                assert_eq!(code.max_stack, 1);
                assert_eq!(code.max_locals, 1);
                assert_eq!(code.code, vec![0x2A, 0xB7, 0x00, 0x01, 0xB1]);
            }
            _ => panic!("Expected Code attribute"),
        }
    }

    #[test]
    fn test_invalid_magic() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00];
        let result = parse_class_file(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::InvalidMagic(0));
    }
}
