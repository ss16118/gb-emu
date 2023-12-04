use log::{error, info, warn};


pub mod cartridge;
use cartridge::*;
pub mod cpu;
use cpu::CPU;
pub mod address_bus;
use address_bus::AddressBus;
pub mod ppu;
use ppu::PPU;
pub mod timer;
use timer::Timer;


    /**
    * Emulator context
    */
pub struct Emulator {
    cartridge: Box<Cartridge>,
    cpu: Box<CPU>,
    address_bus: Box<AddressBus>,
    // Pixel Processing Unit
    ppu: Box<PPU>,
    timer: Box<Timer>,
}

/**
* Emulator implementation
*/
impl Emulator {
    /**
    * Create a new emulator instance given the path to
    * the ROM file.
    */
    pub fn new(rom_file: &str) -> Emulator {
        log::info!("Initializing emulator...");
        let mut cartridge = Cartridge::new();
        let cpu = CPU::new();
        let address_bus = AddressBus::new();
        let ppu = PPU::new();
        let timer = Timer::new();
        
        // Loads the ROM file into the cartridge
        cartridge.load_rom_file(rom_file);

        log::info!(target: "stdout", "Initialize emulator: SUCCESS");

        Emulator {
            cartridge: Box::new(cartridge),
            cpu: Box::new(cpu),
            address_bus: Box::new(address_bus),
            ppu: Box::new(ppu),
            timer: Box::new(timer),
        }
    }

    pub fn run(&mut self) -> () {
        log::info!("Emulator is running");
    }
}