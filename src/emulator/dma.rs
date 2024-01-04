use crate::emulator::ppu::PPU_CTX;
use crate::emulator::address_bus::*;

pub const DMA_ADDR: u16 = 0xFF46;

pub struct DMA {
    active: bool,
    byte: u8,
    value: u8,
    start_delay: u8,
}

// A global instance of DMA context
pub static mut DMA_CTX: DMA = DMA {
    active: false,
    byte: 0,
    value: 0,
    start_delay: 0,
};


impl DMA {
    pub fn start(&mut self, start: u8) -> () {
        self.active = true;
        self.byte = 0;
        self.value = start;
        self.start_delay = 2;
    }

    pub fn is_transferring(&self) -> bool {
        return self.active;
    }

    pub fn tick(&mut self) -> () {
        if !self.active { return; }

        if self.start_delay > 0 {
            self.start_delay -= 1;
            return;
        }
        let addr = (self.value as u16 * 0x100) + self.byte as u16;
        unsafe {
            PPU_CTX.oam_write(self.byte as u16, bus_read(addr));
        }

        // Moves to the next byte
        self.byte += 1;
        self.active = self.byte < 0xA0;
    }
}