use std::env;
use std::fs;
use std::process;

use mini_jvm::classfile::{ConstantPoolEntry, parser::parse_class_file};
use mini_jvm::runtime::{Thread, ClassLoader, Frame};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <class-file>", args[0]);
        process::exit(1);
    }

    let path = &args[1];
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path, e);
            process::exit(1);
        }
    };

    let class_file = match parse_class_file(&bytes) {
        Ok(cf) => cf,
        Err(e) => {
            eprintln!("Failed to parse class file: {}", e);
            process::exit(1);
        }
    };

    // Print parsed class info
    println!("{}", class_file);

    // Try to find and execute main method
    let class_name = match class_file.constant_pool.get(class_file.this_class) {
        Some(ConstantPoolEntry::Class { name_index }) => {
            match class_file.constant_pool.get(*name_index) {
                Some(ConstantPoolEntry::Utf8(name)) => name.clone(),
                _ => {
                    eprintln!("Could not resolve class name");
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Could not resolve class name");
            process::exit(1);
        }
    };

    // Look for main method: public static void main(String[] args)
    let mut class_loader = ClassLoader::new();
    class_loader.load_class_from_bytes(&class_name, &bytes)
        .expect("Failed to load class");

    if let Some(main_method) = class_loader.find_method(
        class_loader.get_class(&class_name).unwrap(),
        "main",
        "([Ljava/lang/String;)V"
    ) {
        if let Some((_max_stack, max_locals, code)) = ClassLoader::get_method_code(
            main_method,
            class_loader.get_class(&class_name).unwrap(),
        ) {
            println!("\n--- Executing main ---");
            let mut thread = Thread::new(class_loader);
            let frame = Frame::new(max_locals as usize, code);
            thread.push_frame(frame);
            thread.execute();
            println!("--- Execution complete ---");
        }
    } else {
        println!("\nNo main method found, skipping execution.");
    }
}
