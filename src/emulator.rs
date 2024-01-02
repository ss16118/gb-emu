use std::thread;
pub mod cartridge;
pub mod io;
pub mod dbg;
use cartridge::*;
pub mod cpu;
use cpu::CPU;
pub mod ram;
use ram::RAM;
pub mod address_bus;
use address_bus::AddressBus;
pub mod ppu;
use ppu::PPU;
pub mod timer;
use timer::Timer;
pub mod ui;
use ui::UI;
use std::sync::Arc;
use std::sync::Mutex;


/**
* Emulator context
*/
#[allow(dead_code)]
pub struct Emulator {
    running: bool,
    paused: bool,
    cartridge: Arc<Mutex<Cartridge>>,
    cpu: Arc<Mutex<CPU>>,
    ram: Arc<Mutex<RAM>>,
    address_bus: Arc<Mutex<AddressBus>>,
    // Pixel Processing Unit
    ppu: Arc<Mutex<PPU>>,
    timer: Arc<Mutex<Timer>>,
}

unsafe impl Send for Emulator {}


fn cpu_run(arc_emulator: Arc<Mutex<Emulator>>, debug: bool) -> () {
    log::info!("Emulator is running");
    let mut emulator = arc_emulator.lock().unwrap();
    emulator.running = true;
    emulator.paused = false;
    while emulator.running {
        if emulator.paused {
            std::thread::sleep(std::time::Duration::from_millis(32));
        }
        let mut cpu = emulator.cpu.lock().unwrap();
        let mut bus = emulator.address_bus.lock().unwrap();
        cpu.step(&mut bus);
        if debug {
            cpu.print_state("trace_file");
        }
        drop(cpu);
        drop(bus);
        emulator.cycles(1);
    }
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

        // Cartridge initialization
        let mut cartridge = Cartridge::new();
        // Loads the ROM file into the cartridge
        cartridge.load_rom_file(rom_file);
        cartridge.print_info(true);
        let cartridge_ptr = Arc::new(Mutex::new(cartridge));

        // Timer initialization
        let timer = Timer::new();
        let timer_ptr = Arc::new(Mutex::new(timer));

        // CPU initialization
        let mut cpu = CPU::new(trace, timer_ptr.clone());
        
        // RAM initialization
        let ram = RAM::new();
        let ram_ptr = Arc::new(Mutex::new(ram));

        let ppu = PPU::new();

        // Address bus initialization
        let address_bus = AddressBus::new(
            cartridge_ptr.clone(), ram_ptr.clone(), timer_ptr.clone());
        
        let cpu_ptr = Arc::new(Mutex::new(cpu));

        let emulator = Emulator {
            running: false,
            paused: true,
            cartridge: cartridge_ptr.clone(),
            cpu: cpu_ptr.clone(),
            ram: ram_ptr.clone(),
            address_bus: Arc::new(Mutex::new(address_bus)),
            ppu: Arc::new(Mutex::new(ppu)),
            timer: timer_ptr.clone(),
        };
        log::info!(target: "stdout", "Initialize emulator: SUCCESS");
        return emulator;
    }

    /**
     * Starts running the emulator
     */
    pub fn run(arc_emulator: Arc<Mutex<Emulator>>, debug: bool) -> () {
        let cpu_thread = 
            thread::spawn(move || cpu_run(arc_emulator, debug));
        let mut ui = UI::new();
        ui.handle_events();

        cpu_thread.join().unwrap();
    }

    /**
     * Increases the tick count for the emulator as well
     * as other components
     */
    pub fn cycles(&mut self, cycles: u32) -> () {
        self.cpu.lock().unwrap().cycles(cycles);
    }
}