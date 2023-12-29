use std::{cell::RefCell, rc::Rc};
use crate::emulator::cartridge::Cartridge;
use crate::emulator::cpu::CPU;
use crate::emulator::ram::RAM;
use super::cartridge;
/**
 * A struct that defines the address bus
 */
pub struct AddressBus {
    cartridge: Rc<RefCell<Cartridge>>,
    // I know this is not a good practice in Rust,
    // since Emulator is technically the owner of CPU,
    // and Rust does not allow multiple mutable references
    // That's why I am using a raw pointer here
    cpu: *mut CPU,
    ram: Rc<RefCell<RAM>>,
}

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

impl AddressBus {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>,
            cpu: *mut CPU, ram: Rc<RefCell<RAM>>) 
            -> AddressBus {
        
        log::info!("Initializing address bus...");
        let bus = AddressBus { 
            cartridge: cartridge,
            cpu: cpu,
            ram: ram,
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
            return self.cartridge.borrow().read(address);
        } else if address < 0xA000 {
            // Reads from BG Map Data 2
            log::error!("Reading from Char/Map data {:04X} currently not supported", address);
            return 0;
        } else if address < 0xC000 {
            // Reads from Cartridge RAM
            self.cartridge.borrow().read(address);
        } else if address < 0xE000 {
            // Reads from Work RAM (WRAM)
            return self.ram.borrow().wram_read(address);
        } else if address < 0xFEA0 {
            // Reads from Object Attribute Memory (OAM)
            log::error!("Reading from Object Attribute Memory (OAM) currently not supported");
            // std::process::exit(-5);
            return 0;
        } else if address < 0xFF00 {
            // Reads from reserved memory (UNUSABLE)
            return 0;
        } else if address < 0xFF80 {
            // Reads from I/O Registers
            log::error!("Reading from I/O Registers currently not supported");
            return 0;
        } else if address < 0xFFFF {
            // Reads from High RAM (HRAM)
            return self.ram.borrow().hram_read(address);
        } else if address == 0xFFFF {
            // Reads from Interrupts Enable Register (IE)
            return unsafe { (*self.cpu).get_ie_register() };
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
    pub fn write(&mut self, address: u16, data: u8) -> () {
        // Given address indicates ROM address
        if address <= 0x8000 {
            // Writes to ROM
            self.cartridge.borrow_mut().write(address, data);
        } else if address < 0xA000 {
            // Writes to BG Map Data 2
            log::error!("Writing to Char/Map data {:04X} currently not supported", address);
            // std::process::exit(-5);
        } else if address < 0xC000 {
            // Writes to Cartridge RAM
            self.cartridge.borrow_mut().write(address, data);
        } else if address < 0xE000 {
            // Writes to Work RAM (WRAM)
            self.ram.borrow_mut().wram_write(address, data);
        } else if address < 0xFEA0 {
            // Writes to Object Attribute Memory (OAM)
            log::error!("Writing to Object Attribute Memory (OAM) currently not supported");
            std::process::exit(-5);
        } else if address < 0xFF00 {
            // Writes to reserved memory (UNUSABLE)
            return;
        } else if address < 0xFF80 {
            // Writes to I/O Registers
            log::error!("Writing to I/O Registers currently not supported {:04X}", address);
            // std::process::exit(-5);
        } else if address < 0xFFFF {
            // Writes to High RAM (HRAM)
            self.ram.borrow_mut().hram_write(address, data);
        } else if address == 0xFFFF {
            // Writes to Interrupts Enable Register (IE)
            // self.cpu.borrow_mut().set_ie_register(data);
            return unsafe { (*self.cpu).set_ie_register(data); };
        }
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