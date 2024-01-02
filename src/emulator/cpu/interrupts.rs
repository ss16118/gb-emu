use crate::emulator::cpu::CPU;
use crate::emulator::cpu::instruction::RegType;
use crate::emulator::address_bus::*;
use crate::emulator::cpu::CPU_CTX;

const VBLANK_ADDR: u16 = 0x40;
const LCD_STAT_ADDR: u16 = 0x48;
const TIMER_ADDR: u16 = 0x50;
const SERIAL_ADDR: u16 = 0x58;
const JOYPAD_ADDR: u16 = 0x60;


#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum InterruptType {
    IT_VBLANK = 0x1,
    IT_LCD_STAT = 0x2,
    IT_TIMER = 0x4,
    IT_SERIAL = 0x8,
    IT_JOYPAD = 0x10,
}


/**
 * A helper function that sets the PC to the given address
 */
fn set_interrupt_addr(address: u16) -> () {
    unsafe {
        // Pushes PC onto the stack
        CPU_CTX.stack_push16(CPU_CTX.read_reg(&RegType::RT_PC));
        // Sets the PC to the given address
        CPU_CTX.set_register(&RegType::RT_PC, address);
    }
}

/**
 * A helper function that checks if an interrupt should be triggered
 */
fn interrupt_check(address: u16, int_type: InterruptType) -> bool {
    unsafe {
        let int_type_u8 = int_type as u8;
        if ((CPU_CTX.get_int_flags() & int_type_u8) != 0) && 
        ((CPU_CTX.get_ie_register() & int_type_u8) != 0) {
            // FIXME should probably not use magic number
            set_interrupt_addr(address);
            CPU_CTX.set_int_flags(CPU_CTX.get_int_flags() & !int_type_u8);
            CPU_CTX.halted = false;
            CPU_CTX.interrupt_master_enabled = false;
            return true;
        }
        return false;
    }
}

/**
 * Handles interrupts
 */
pub fn handle_interrupts() -> () {
    if interrupt_check(VBLANK_ADDR, InterruptType::IT_VBLANK) {} 
    else if interrupt_check(LCD_STAT_ADDR, InterruptType::IT_LCD_STAT) {}
    else if interrupt_check(TIMER_ADDR, InterruptType::IT_TIMER) {}
    else if interrupt_check(SERIAL_ADDR, InterruptType::IT_SERIAL) {} 
    else if interrupt_check(JOYPAD_ADDR, InterruptType::IT_JOYPAD) {}
}


pub fn request_interrupt(interrupt_type: InterruptType) -> () {
    log::info!(target: "trace_file", "Interrupt requested: {:?}", interrupt_type);
    unsafe { CPU_CTX.int_flags |= interrupt_type as u8; }
}