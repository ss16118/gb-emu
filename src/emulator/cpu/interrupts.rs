use crate::emulator::cpu::CPU;
use crate::emulator::cpu::instruction::RegType;
use crate::emulator::address_bus::AddressBus;

const VBLANK_ADDR: u16 = 0x40;
const LCD_STAT_ADDR: u16 = 0x48;
const TIMER_ADDR: u16 = 0x50;
const SERIAL_ADDR: u16 = 0x58;
const JOYPAD_ADDR: u16 = 0x60;


#[allow(non_camel_case_types)]
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
fn set_interrupt_addr(cpu: &mut CPU, bus: &mut AddressBus, address: u16) -> () {
    // Pushes PC onto the stack
    cpu.stack_push16(bus, cpu.read_reg(&RegType::RT_PC));
    // Sets the PC to the given address
    cpu.set_register(&RegType::RT_PC, address);
}

/**
 * A helper function that checks if an interrupt should be triggered
 */
fn interrupt_check(cpu: &mut CPU, address: u16,
        bus: &mut AddressBus, int_type: InterruptType) -> bool {
    let int_type_u8 = int_type as u8;
    if ((cpu.get_int_flags() & int_type_u8) != 0) && 
       ((cpu.get_ie_register() & int_type_u8) != 0) {
        // FIXME should probably not use magic number
        set_interrupt_addr(cpu, bus, address);
        cpu.set_int_flags(cpu.get_int_flags() & !int_type_u8);
        cpu.halted = false;
        cpu.interrupt_master_enabled = false;
        return true;
    }
    return false;
}

/**
 * Handles interrupts
 */
pub fn handle_interrupts(cpu: &mut CPU, bus: &mut AddressBus) -> () {
    if interrupt_check(cpu, VBLANK_ADDR, bus, InterruptType::IT_VBLANK) {} 
    else if interrupt_check(cpu, LCD_STAT_ADDR, bus, InterruptType::IT_LCD_STAT) {}
    else if interrupt_check(cpu, TIMER_ADDR, bus, InterruptType::IT_TIMER) {}
    else if interrupt_check(cpu, SERIAL_ADDR, bus, InterruptType::IT_SERIAL) {} 
    else if interrupt_check(cpu, JOYPAD_ADDR, bus, InterruptType::IT_JOYPAD) {}
}


pub fn request_interrupt(cpu: *mut CPU, interrupt_type: InterruptType) -> () {
    unsafe { (*cpu).int_flags |= interrupt_type as u8; }
}