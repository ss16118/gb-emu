use std::ptr;

use crate::emulator::timer::*;
use crate::emulator::cpu::{CPU, INT_FLAGS_ADDR};
static mut serial_data: [u8; 2] = [0, 0];


/**
 * Reads a byte from the given address from the I/O registers
 */
pub fn io_read(address: u16, timer: &Timer, cpu: &CPU) -> u8 {
    if address == 0xFF01 {
        return unsafe { serial_data[0] };
    }
    if address == 0xFF02 {
        return unsafe { serial_data[1] };
    }
    if DIV_ADDR <= address && address <= TAC_ADDR {
        return timer.read(address);
    }
    if address == INT_FLAGS_ADDR {
        return cpu.get_int_flags();
    }
    log::error!("Reading from I/O address 0x{:04X} currently not supported", address);
    return 0;
}


/**
 * Writes a byte to the given address
 */
pub fn io_write(address: u16, data: u8, timer: &mut Timer,
            cpu: &mut CPU) -> () {
    
    if address == 0xFF01 {
        unsafe { serial_data[0] = data };
        return;
    }
    if address == 0xFF02 {
        unsafe { serial_data[1] = data };
        return;
    }
    if DIV_ADDR <= address && address <= TAC_ADDR {
        timer.write(address, data);
        return;
    }
    if address == INT_FLAGS_ADDR {
        cpu.set_int_flags(data);
        return;
    }
    log::error!("Writing to I/O address 0x{:04X} currently not supported", address);
}