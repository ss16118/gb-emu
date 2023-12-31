use crate::emulator::cpu::CPU;
use crate::emulator::cpu::instruction::RegType;
use crate::emulator::address_bus::AddressBus;

const VBLANK_ADDR: u16 = 0x40;
const LCD_STAT_ADDR: u16 = 0x48;
const TIMER_ADDR: u16 = 0x50;
const SERIAL_ADDR: u16 = 0x58;
const JOYPAD_ADDR: u16 = 0x60;


enum InterruptType {
    IT_VBlank = 0x1,
    IT_LCD_Stat = 0x2,
    IT_Timer = 0x4,
    IT_Serial = 0x8,
    IT_Joypad = 0x10,
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
 * A helper function 
 */
fn interrupt_check(cpu: &mut CPU, address: u16,
        bus: &mut AddressBus, int_type: InterruptType) -> bool {
    let int_type_u8 = int_type as u8;
    if ((cpu.int_flags & int_type_u8) > 0) && 
       ((cpu.ie_register & int_type_u8 as u8) > 0) {
        // FIXME should probably not use magic number
        set_interrupt_addr(cpu, bus, address);
        cpu.set_int_flags(cpu.get_int_flags() & !int_type_u8);
        cpu.halted = false;
        cpu.interrupt_master_enabled = false;
        return true;
    }
    return false;
}

pub fn handle_interrupts(cpu: &mut CPU, bus: &mut AddressBus) -> () {
    if interrupt_check(cpu, VBLANK_ADDR, bus, InterruptType::IT_VBlank) {} 
    else if interrupt_check(cpu, LCD_STAT_ADDR, bus, InterruptType::IT_LCD_Stat) {}
    else if interrupt_check(cpu, TIMER_ADDR, bus, InterruptType::IT_Timer) {}
    else if interrupt_check(cpu, SERIAL_ADDR, bus, InterruptType::IT_Serial) {} 
    else if interrupt_check(cpu, JOYPAD_ADDR, bus, InterruptType::IT_Joypad) {}
}