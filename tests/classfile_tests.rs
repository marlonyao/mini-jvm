use mini_jvm::classfile::parser::parse_class_file;
use mini_jvm::classfile::{AttributeData, ParseError};

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
fn test_integration_parse_minimal_class() {
    let bytes = minimal_class_bytes();
    let class = parse_class_file(&bytes).expect("parse should succeed");
    assert_eq!(class.magic, 0xCAFEBABE);
    assert_eq!(class.minor_version, 0);
    assert_eq!(class.major_version, 52);
    assert_eq!(class.constant_pool.entries.len(), 12);

    // Verify some constant pool entries
    match class.constant_pool.get(4) {
        Some(mini_jvm::classfile::ConstantPoolEntry::Utf8(s)) => {
            assert_eq!(s, "Hello");
        }
        _ => panic!("Expected Utf8 'Hello' at index 4"),
    }

    match class.constant_pool.get(10) {
        Some(mini_jvm::classfile::ConstantPoolEntry::NameAndType { name_index, descriptor_index }) => {
            assert_eq!(*name_index, 6);
            assert_eq!(*descriptor_index, 7);
        }
        _ => panic!("Expected NameAndType at index 10"),
    }

    assert_eq!(class.access_flags, 0x0021);
    assert_eq!(class.this_class, 2);
    assert_eq!(class.super_class, 3);
    assert!(class.interfaces.is_empty());
    assert!(class.fields.is_empty());
    assert_eq!(class.methods.len(), 1);

    let method = &class.methods[0];
    assert_eq!(method.access_flags, 1);
    assert_eq!(method.name_index, 6);
    assert_eq!(method.descriptor_index, 7);
    assert_eq!(method.attributes.len(), 1);

    let attr = &method.attributes[0];
    assert_eq!(attr.name_index, 8);
    match &attr.data {
        AttributeData::Code(code) => {
            assert_eq!(code.max_stack, 1);
            assert_eq!(code.max_locals, 1);
            assert_eq!(code.code, vec![0x2A, 0xB7, 0x00, 0x01, 0xB1]);
            assert!(code.exception_table.is_empty());
            assert_eq!(code.attributes.len(), 1);
        }
        _ => panic!("Expected Code attribute"),
    }

    assert_eq!(class.attributes.len(), 1);
    assert_eq!(class.attributes[0].name_index, 11);
}

#[test]
fn test_integration_invalid_magic() {
    let bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let result = parse_class_file(&bytes);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ParseError::InvalidMagic(0xDEADBEEF));
}
