use crate::emulator::ram::RAM_CTX;
use crate::emulator::io::{io_read, io_write};
use crate::emulator::cpu::CPU_CTX;
use crate::emulator::ppu::PPU_CTX;
use crate::emulator::dma::DMA_CTX;
use super::cartridge::CARTRIDGE_CTX;
/**
 * A struct that defines the address bus
 */
// pub struct AddressBus {
//     cartridge: Arc<Mutex<Cartridge>>,
//     ram: Arc<Mutex<RAM>>,
//     timer: Arc<Mutex<Timer>>,
//     ppu: Arc<Mutex<PPU>>
// }

/**
 * Memory map of Game Boy
 * http://gameboy.mongenel.com/dmg/asmmemmap.html
 * 0x0000 - 0x3FFF: 16 KiB ROM bank 00
 * 0x4000 - 0x7FFF: 16 KiB ROM Bank 01..NN
 * 0x8000 - 0x97FF: Character RAM
 * 0x9800 - 0x9BFF: BG Map Data 1
 * 0x9C00 - 0x9FFF: BG Map Data 2
 * 0xA000 - 0xBFFF: Cartridge RAM (If Available)
 * 0xC000 - 0xCFFF: 4 KiB Work RAM (WRAM) bank 0
 * 0xD000 - 0xDFFF: 4 KiB Work RAM (WRAM): In CGB mode, switchable bank 1-7
 * 0xE000 - 0xFDFF: Mirror of C000~DDFF (ECHO RAM)
 * 0xFE00 - 0xFE9F: Object Attribute Memory (OAM)
 * 0xFEA0 - 0xFEFF: Not Usable
 * 0xFF00 - 0xFF7F: I/O Registers
 * 0xFF80 - 0xFFFE: High RAM (HRAM) (Zero page)
 * 0xFFFF - 0xFFFF: Interrupts Enable Register (IE)
 */

/**
 * Reads a byte from the address bus
 */
pub fn bus_read(address: u16) -> u8 {
    // Given address indicates ROM address
    if address < 0x8000 {
        // Reads from ROM
        return unsafe { CARTRIDGE_CTX.read(address) };
    } else if address < 0xA000 {
        // Reads from BG Map Data 2
        return unsafe { PPU_CTX.vram_read(address) };
    } else if address < 0xC000 {
        // Reads from Cartridge RAM
        return unsafe { CARTRIDGE_CTX.read(address) };
    } else if address < 0xE000 {
        // Reads from Work RAM (WRAM)
        return unsafe { RAM_CTX.wram_read(address) };
    } else if address < 0xFE00 {
        // Reads from ECHO RAM
        return 0;
    } else if address < 0xFEA0 {
        // Reads from Object Attribute Memory (OAM)
        if unsafe { DMA_CTX.is_transferring() } {
            return 0xFF;
        }
        return unsafe { PPU_CTX.oam_read(address) };
    } else if address < 0xFF00 {
        // Reads from reserved memory (UNUSABLE)
        return 0;
    } else if address < 0xFF80 {
        // Reads from I/O Registers
        return io_read(address);
    } else if address < 0xFFFF {
        // Reads from High RAM (HRAM)
        return unsafe { RAM_CTX.hram_read(address) };
    } else if address == 0xFFFF {
        // Reads from Interrupts Enable Register (IE)
        return unsafe { CPU_CTX.get_ie_register() };
        // return self.cpu.borrow().get_ie_register();
    }
    // Raises an error if the address is out of range
    log::error!(target: "stdout",
        "Reading from address 0x{:04X} currently not supported", address);
    std::process::exit(-5);
}

/**
 * Writes a byte to the address bus
 */
pub fn bus_write(address: u16, data: u8) -> () {
    // Given address indicates ROM address
    if address < 0x8000 {
        // Writes to ROM
        unsafe { CARTRIDGE_CTX.write(address, data) };
    } else if address < 0xA000 {
        // Writes to BG Map Data
        unsafe { PPU_CTX.vram_write(address, data) };
    } else if address < 0xC000 {
        // Writes to Cartridge RAM
        unsafe { CARTRIDGE_CTX.write(address, data) };
    } else if address < 0xE000 {
        // Writes to Work RAM (WRAM)
        unsafe { RAM_CTX.wram_write(address, data) };
        return;
    } else if address < 0xFE00 {
        // Writes to ECHO RAM
        return;
    } else if address < 0xFEA0 {
        // Writes to Object Attribute Memory (OAM)
        if unsafe { DMA_CTX.is_transferring() } {
            return;
        }
        unsafe { PPU_CTX.oam_write(address, data) };
    } else if address < 0xFF00 {
        // Writes to reserved memory (UNUSABLE)
        return;
    } else if address < 0xFF80 {
        // Writes to I/O Registers
        io_write(address, data);
        return;
        // std::process::exit(-5);
    } else if address < 0xFFFF {
        // Writes to High RAM (HRAM)
        unsafe { RAM_CTX.hram_write(address, data) };
        return;
    } else if address == 0xFFFF {
        // Writes to Interrupts Enable Register (IE)
        // self.cpu.borrow_mut().set_ie_register(data);
        unsafe { CPU_CTX.set_ie_register(data) };
        return;
    }
}

/**
 * Read a 16-bit value from the given address
 */
pub fn bus_read_16(address: u16) -> u16 {
    let low = bus_read(address);
    let high = bus_read(address + 1);
    return (high as u16) << 8 | low as u16;
}
    
    /**
     * Write a 16-bit value to the given address
     */
pub fn bus_write_16(address: u16, data: u16) -> () {
    let low = data as u8;
    let high = (data >> 8) as u8;
    bus_write(address, low);
    bus_write(address + 1, high);
}
