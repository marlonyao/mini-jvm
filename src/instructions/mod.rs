pub mod constants;
pub mod loads;
pub mod stores;
pub mod math;
pub mod stack;
pub mod control;
pub mod invoke;
pub mod objects;

use crate::runtime::thread::{Thread, ExecutionResult};

/// Main instruction dispatch. Called by the thread's execute loop.
pub fn execute_instruction(thread: &mut Thread, opcode: u8) -> ExecutionResult {
    match opcode {
        // Constants
        0x00 => constants::nop(thread),
        0x02 => constants::iconst_m1(thread),
        0x03 => constants::iconst(thread, 0),
        0x04 => constants::iconst(thread, 1),
        0x05 => constants::iconst(thread, 2),
        0x06 => constants::iconst(thread, 3),
        0x07 => constants::iconst(thread, 4),
        0x08 => constants::iconst(thread, 5),
        0x10 => constants::bipush(thread),
        0x11 => constants::sipush(thread),
        0x12 => objects::ldc(thread),
        0x13 => objects::ldc_w(thread),

        // Iload
        0x1A => loads::iload_n(thread, 0),
        0x1B => loads::iload_n(thread, 1),
        0x1C => loads::iload_n(thread, 2),
        0x1D => loads::iload_n(thread, 3),
        0x15 => loads::iload(thread),

        // Aload
        0x2A => loads::aload_n(thread, 0),
        0x2B => loads::aload_n(thread, 1),
        0x2C => loads::aload_n(thread, 2),
        0x2D => loads::aload_n(thread, 3),
        0x19 => loads::aload(thread),

        // Istore
        0x3B => stores::istore_n(thread, 0),
        0x3C => stores::istore_n(thread, 1),
        0x3D => stores::istore_n(thread, 2),
        0x3E => stores::istore_n(thread, 3),
        0x36 => stores::istore(thread),

        // Astore
        0x4B => stores::astore_n(thread, 0),
        0x4C => stores::astore_n(thread, 1),
        0x4D => stores::astore_n(thread, 2),
        0x4E => stores::astore_n(thread, 3),
        0x3A => stores::astore(thread),

        // Math
        0x60 => math::iadd(thread),
        0x64 => math::isub(thread),
        0x68 => math::imul(thread),
        0x6C => math::idiv(thread),
        0x70 => math::irem(thread),
        0x74 => math::ineg(thread),
        0x84 => math::iinc(thread),
        0x61 => math::ladd(thread),
        0x65 => math::lsub(thread),
        0x69 => math::lmul(thread),
        0x6D => math::ldiv(thread),
        0x75 => math::lneg(thread),

        // Stack
        0x57 => stack::pop_op(thread),
        0x59 => stack::dup(thread),
        0x5F => stack::swap(thread),

        // Control flow
        0x99 => control::ifeq(thread),
        0x9A => control::ifne(thread),
        0x9F => control::if_icmpeq(thread),
        0xA0 => control::if_icmpne(thread),
        0xA1 => control::if_icmplt(thread),
        0xA2 => control::if_icmpge(thread),
        0xA3 => control::if_icmpgt(thread),
        0xA4 => control::if_icmple(thread),
        0xA7 => control::goto(thread),
        0xC6 => control::ifnull(thread),
        0xC7 => control::ifnonnull(thread),

        // Object operations
        0xBB => objects::new_op(thread),     // new
        0xB2 => objects::getstatic(thread),  // getstatic
        0xB3 => objects::putstatic(thread),  // putstatic

        // Invoke & return
        0xB6 => invoke::invokevirtual(thread),
        0xB7 => invoke::invokespecial(thread),
        0xB8 => invoke::invokestatic(thread),
        0xAC => invoke::ireturn(thread),
        0xB0 => invoke::areturn(thread),
        0xB1 => invoke::r#return(thread),

        _ => panic!("Unimplemented opcode: 0x{:02X} at pc={}", opcode, {
            let f = thread.current_frame();
            f.pc.saturating_sub(1)
        }),
    }
}
