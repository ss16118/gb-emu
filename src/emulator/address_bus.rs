use std::rc::Rc;

use crate::emulator::cartridge::Cartridge;

use super::cartridge;
/**
 * A struct that defines the address bus
 */
pub struct AddressBus {
    cartridge: Rc<Cartridge>,
}

/**
 * Memory map of Game Boy
 * 0x0000 - 0x3FFF: 16 KiB ROM bank 00
 * 0x4000 - 0x7FFF: 16 KiB ROM Bank 01..NN
 * 0x8000 - 0x9FFF: 8 KiB Video RAM (VRAM)
 * 0xA000 - 0xBFFF: 8 KiB External RAM
 * 0xC000 - 0xCFFF: 4 KiB Work RAM (WRAM) bank 0
 * 0xD000 - 0xDFFF: 4 KiB Work RAM (WRAM): In CGB mode, switchable bank 1-7
 * 0xE000 - 0xFDFF: Mirror of C000~DDFF (ECHO RAM)
 * 0xFE00 - 0xFE9F: Object Attribute Memory (OAM)
 * 0xFEA0 - 0xFEFF: Not Usable
 * 0xFF00 - 0xFF7F: I/O Registers
 * 0xFF80 - 0xFFFE: High RAM (HRAM)
 * 0xFFFF - 0xFFFF: Interrupts Enable Register (IE)
 */

impl AddressBus {
    pub fn new(cartridge: Rc<Cartridge>) -> AddressBus {
        log::info!("Initializing address bus...");
        let bus = AddressBus { 
            cartridge: cartridge,
         };
        log::info!(target: "stdout", "Initialize address: SUCCESS");
        return bus;
    }

    /**
     * Reads a byte from the address bus
     */
    pub fn read(&self, address: u16) -> u8 {
        // Given address indicates ROM address
        if address <= 0x8000 {
            // Reads from ROM
            return self.cartridge.read(address);
        }
        // Raises an error if the address is out of range
        log::error!("Reading from address 0x{:04X} currently not supported", address);
        std::process::exit(-5);
    }

    /**
     * Writes a byte to the address bus
     */
    pub fn write(&mut self, address: u16, data: u8) -> () {
        // Given address indicates ROM address
        if address <= 0x8000 {
            // Writes to ROM
            if let Some(cartridge) = Rc::get_mut(&mut self.cartridge) {
                cartridge.write(address, data);
            } else {
                log::error!("Failed to obtain mutable reference to cartridge");
                std::process::exit(-1);
            }
        }
        
        log::error!("Writing to address 0x{:04X} currently not supported", address);
        std::process::exit(-5);
    }

    /**
     * Read a 16-bit value from the given address
     */
    pub fn read_16(&mut self, address: u16) -> u16 {
        let low = self.read(address);
        let high = self.read(address + 1);
        return (high as u16) << 8 | low as u16;
    }
    
    /**
     * Write a 16-bit value to the given address
     */
    pub fn write_16(&mut self, address: u16, data: u16) -> () {
        let low = data as u8;
        let high = (data >> 8) as u8;
        self.write(address, low);
        self.write(address + 1, high);
    }

}