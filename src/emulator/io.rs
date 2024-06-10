use std::ptr;

use crate::emulator::timer::*;
use crate::emulator::dma::*;
use crate::emulator::cpu::{CPU_CTX, INT_FLAGS_ADDR};
use crate::emulator::ppu::PPU_CTX;
use crate::emulator::lcd::*;
use crate::emulator::gamepad::*;
static mut serial_data: [u8; 2] = [0, 0];

static mut read_sound_warning: bool = false;
static mut write_sound_warning: bool = false;

/**
 * Reads a byte from the given address from the I/O registers
 */
pub fn io_read(address: u16) -> u8 {
    if address == 0xFF00 {
        return unsafe { GAMEPAD_CTX.get_output() };
    }
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
    if LCD_START_ADDR <= address && address <= LCD_END_ADDR {
        return unsafe { LCD_CTX.read(address) };
    }

    if 0xFF10 <= address && address <= 0xFF3F && !unsafe { read_sound_warning } {
        log::warn!("Reading from sound registers not supported");
        unsafe { read_sound_warning = true };
        return 0;
    }
    return 0;
}


/**
 * Writes a byte to the given address
 */
pub fn io_write(address: u16, data: u8) -> () {
    if address == 0xFF00 {
        unsafe { GAMEPAD_CTX.set_select(data) };
        return;
    }
    
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
    if LCD_START_ADDR <= address && address <= LCD_END_ADDR {
        unsafe { LCD_CTX.write(address, data) };
        return;
    }
    if 0xFF10 <= address && address <= 0xFF3F && !unsafe { write_sound_warning } {
        log::warn!("Writing to sound registers not supported");
        unsafe { write_sound_warning = true };
        return;
    }
}