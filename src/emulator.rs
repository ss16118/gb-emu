use std::rc::Rc;
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
#[allow(dead_code)]
pub struct Emulator {
    running: bool,
    paused: bool,
    cartridge: Rc<Cartridge>,
    cpu: Box<CPU>,
    address_bus: Box<AddressBus>,
    // Pixel Processing Unit
    ppu: Box<PPU>,
    timer: Box<Timer>,
}

/**
* Emulator implementation
*/
#[allow(dead_code)]
impl Emulator {
    /**
    * Create a new emulator instance given the path to
    * the ROM file.
    */
    pub fn new(rom_file: &str, trace: bool) -> Emulator {
        log::info!("Initializing emulator...");

        let mut cartridge = Cartridge::new();
        // Loads the ROM file into the cartridge
        cartridge.load_rom_file(rom_file);
        cartridge.print_info(true);
        let cartridge_ptr = Rc::new(cartridge);
        let cpu = CPU::new(trace);
        let address_bus = AddressBus::new(cartridge_ptr.clone());
        let ppu = PPU::new();
        let timer = Timer::new();        

        let emulator = Emulator {
            running: false,
            paused: true,
            cartridge: cartridge_ptr.clone(),
            cpu: Box::new(cpu),
            address_bus: Box::new(address_bus),
            ppu: Box::new(ppu),
            timer: Box::new(timer),
        };
        log::info!(target: "stdout", "Initialize emulator: SUCCESS");
        return emulator;
    }

    pub fn run(&mut self) -> () {
        log::info!("Emulator is running");
        self.running = true;
        self.paused = false;
        while self.running {
            if self.paused {
               std::thread::sleep(std::time::Duration::from_millis(32));
            }
            if !(*self.cpu).step(&self.address_bus) {
                log::error!(target: "stdout", "CPU stopped");
                std::process::exit(-1);
            }
            self.tick();
        }
    }

    fn tick(&mut self) -> () {

    }
}