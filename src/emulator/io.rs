use std::ptr;

use crate::emulator::timer::*;
use crate::emulator::dma::*;
use crate::emulator::cpu::{CPU_CTX, INT_FLAGS_ADDR};
use crate::emulator::ppu::PPU_CTX;
static mut serial_data: [u8; 2] = [0, 0];


/**
 * Reads a byte from the given address from the I/O registers
 */
pub fn io_read(address: u16) -> u8 {
    if address == 0xFF01 {
        return unsafe { serial_data[0] };
    }
    if address == 0xFF02 {
        return unsafe { serial_data[1] };
    }
    if DIV_ADDR <= address && address <= TAC_ADDR {
        return unsafe { TIMER_CTX.read(address) };
    }
    if address == INT_FLAGS_ADDR {
        return unsafe { CPU_CTX.get_int_flags() };
    }
    log::error!("Reading from I/O address 0x{:04X} currently not supported", address);
    return 0;
}


/**
 * Writes a byte to the given address
 */
pub fn io_write(address: u16, data: u8) -> () {
    
    if address == 0xFF01 {
        unsafe { serial_data[0] = data };
        return;
    }
    if address == 0xFF02 {
        unsafe { serial_data[1] = data };
        return;
    }
    if DIV_ADDR <= address && address <= TAC_ADDR {
        unsafe { TIMER_CTX.write(address, data) };
        return;
    }
    if address == INT_FLAGS_ADDR {
        unsafe { CPU_CTX.set_int_flags(data) };
        return;
    }
    if address == DMA_ADDR {
        unsafe { DMA_CTX.start(data); }
        return;
    }
    log::error!("Writing to I/O address 0x{:04X} currently not supported", address);
}