use std::sync::atomic::{AtomicU16, Ordering};
use crate::emulator::cpu::CPU;
use crate::emulator::cpu::interrupts::*;

pub const DIV_ADDR:  u16 = 0xFF04;
pub const TIMA_ADDR: u16 = 0xFF05;
pub const TMA_ADDR:  u16 = 0xFF06;
pub const TAC_ADDR:  u16 = 0xFF07;

const DEFAULT_ORDER: Ordering = Ordering::Relaxed;


/**
 * GameBoy Timer
 * https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
 */
pub struct Timer {
    // Divider Register (DIV)
    div: AtomicU16,
    // Timer Counter (TIMA)
    tima: u8,
    // Timer Modulo (TMA)
    tma: u8,
    // Timer Control (TAC)
    tac: u8,
}

impl Timer {
    pub fn new() -> Timer {
        log::info!("Initializing timer...");
        let timer = Timer {
            div: AtomicU16::new(0xABCC),
            tima: 0, tma: 0, tac: 0
        };
        log::info!(target: "stdout", "Initialize timer: SUCCESS");
        return timer;
    }

    /**
     * Performs one timer tick. Returns true if the timer
     * interrupt should be requested.
     */
    pub fn tick(&mut self) -> bool {
        // Increments the DIV register
        let prev_div = self.div.load(DEFAULT_ORDER);
        self.div.fetch_add(1, DEFAULT_ORDER);

        let mut timer_update: bool = false;
        
        match self.tac & 0b11 {
            0x00 => {
                timer_update = ((prev_div & (1 << 9)) != 0) && 
                               ((self.div.load(DEFAULT_ORDER) & (1 << 9)) == 0);
            },
            0x01 => {
                timer_update = ((prev_div & (1 << 3)) != 0) && 
                               ((self.div.load(DEFAULT_ORDER) & (1 << 3)) == 0);
            }
            0x02 => {
                timer_update = ((prev_div & (1 << 5)) != 0) && 
                               ((self.div.load(DEFAULT_ORDER) & (1 << 5)) == 0);
            }
            0x03 => {
                timer_update = ((prev_div & (1 << 7)) != 0) && 
                               ((self.div.load(DEFAULT_ORDER) & (1 << 7)) == 0);
            }
            _ => (),
        }
        // If the timer is enabled and the timer update flag is set
        if timer_update && self.is_enabled() {
            self.tima = self.tima.wrapping_add(1);
            if self.tima == 0xFF {
                self.tima = self.tma;
                return true;
            }
        }
        return false;
    }
    
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        return (self.tac & 0b100) != 0;
    }

    /**
     * Reads from the register managed by the timer given
     * the address.
     */
    pub fn read(&self, address: u16) -> u8 {
        match address {
            DIV_ADDR     => { return (self.div.load(DEFAULT_ORDER) >> 8) as u8; },
            TIMA_ADDR    => { return self.tima; },
            TMA_ADDR     => { return self.tma; },
            TAC_ADDR     => { return self.tac; },
            _ => {
                log::error!("Invalid timer read address: {:04X}", address);
                std::process::exit(-1);
            }
        }
    }

    /**
     * Writes the given value to the register managed
     * by the timer as per the address
     */
    pub fn write(&mut self, address: u16, data: u8) -> () {
        match address {
            // Resets DIV
            DIV_ADDR  => { self.div.store(0, DEFAULT_ORDER); },
            // TIMA
            TIMA_ADDR => { self.tima = data; }
            // TMA
            TMA_ADDR  => { self.tma = data; }
            // TAC
            TAC_ADDR  => { self.tac = data; }
            _ => {
                log::error!("Invalid timer write address: {:04X}", address);
                std::process::exit(-1);
            }
        }
    }
}