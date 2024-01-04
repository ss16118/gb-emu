use std::thread;
pub mod cartridge;
pub mod io;
pub mod dbg;
pub mod dma;
pub mod lcd;
use lcd::*;
use dma::DMA_CTX;
use std::sync::atomic::{AtomicU16, Ordering};
use cartridge::CARTRIDGE_CTX;
pub mod cpu;
use cpu::CPU_CTX;
use cpu::interrupts::*;
pub mod ram;
use ram::RAM;
pub mod address_bus;
use address_bus::*;
pub mod ppu;
use ppu::PPU_CTX;
pub mod timer;
use timer::TIMER_CTX;
pub mod ui;
use std::sync::Arc;
use std::sync::Mutex;

use crate::emulator::cpu::CPU;


/**
* Emulator context
*/
#[allow(dead_code)]
pub struct Emulator {
    running: bool,
    paused: bool,
}

unsafe impl Send for Emulator {}

pub static mut EMULATOR_CTX: Emulator = Emulator {
    running: false,
    paused: true,
};

fn cpu_run(debug: bool) -> () {
    log::info!("Emulator is running");
    unsafe {
        EMULATOR_CTX.running = true;
        EMULATOR_CTX.paused = false;
        while EMULATOR_CTX.running {
            if EMULATOR_CTX.paused {
                std::thread::sleep(std::time::Duration::from_millis(32));
            }
            CPU_CTX.step();
            if debug {
                CPU_CTX.print_state("trace_file");
            }
            Emulator::cycles(1);
        }
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
    pub fn init(rom_file: &str, trace: bool) -> () {
        log::info!("Initializing emulator...");

        // Cartridge initialization
        // Loads the ROM file into the cartridge
        unsafe {
            CARTRIDGE_CTX.load_rom_file(rom_file);
            CARTRIDGE_CTX.print_info(true);
            LCD::init();
            CPU::cpu_init(trace);
        }
        log::info!(target: "stdout", "Initialize emulator: SUCCESS");
    }

    /**
     * Starts running the emulator
     */
    pub fn run(debug: bool) -> () {
        let cpu_thread = 
            thread::spawn(move || cpu_run(debug));
        ui::init();
        ui::run();
        cpu_thread.join().unwrap();
    }

    /**
     * Increases the tick count for the emulator as well
     * as other components
     */
    pub fn cycles(cycles: u32) -> () {

        for _i in 0..cycles {
            for _n in 0..4 { 
                unsafe {
                    CPU_CTX.ticks.fetch_add(1, Ordering::Relaxed);
                    if TIMER_CTX.tick() {
                        request_interrupt(InterruptType::IT_TIMER);
                    }
                    PPU_CTX.tick();
                }
            }
            unsafe { DMA_CTX.tick(); }
        }
    }
}