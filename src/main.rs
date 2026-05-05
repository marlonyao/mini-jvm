use std::env;
use std::fs;
use mini_jvm::classfile::parser::parse_class_file;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <class-file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let bytes = fs::read(path).expect("Failed to read class file");
    match parse_class_file(&bytes) {
        Ok(class_file) => {
            println!("{}", class_file);
        }
        Err(e) => {
            eprintln!("Failed to parse class file: {}", e);
            std::process::exit(1);
        }
    }
}
