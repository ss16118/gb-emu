pub struct RAM {
    // Work RAM (WRAM)
    wram: [u8; 0x2000],
    // High RAM (HRAM)
    hram: [u8; 0x80]
}


pub static mut RAM_CTX: RAM = RAM {
    wram: [0; 0x2000],
    hram: [0; 0x80]
};

impl RAM {
    /**
     * Reads a byte from the WRAM
     */    
    pub fn wram_read(&self, mut address: u16) -> u8 {
        address -= 0xC000;
        if (address as usize) < self.wram.len() {
            return self.wram[address as usize];
        } else {
            log::error!("Invalid read from RAM address {:04X}", address);
            std::process::exit(-1);
        }
    }

    /**
     * Writes a byte to the WRAM
     */
    pub fn wram_write(&mut self, mut address: u16, value: u8) -> () {
        address -= 0xC000;
        if (address as usize) < self.wram.len() {
            self.wram[address as usize] = value;
        } else {
            log::error!("Invalid write to RAM address {:04X}", address);
            std::process::exit(-1);
        }
    }

    /**
     * Reads a byte from the HRAM
     */
    pub fn hram_read(&self, mut address: u16) -> u8 {
        address -= 0xFF80;
        if (address as usize) < self.hram.len() {
            return self.hram[address as usize];
        } else {
            log::error!("Invalid read from RAM address {:04X}", address);
            std::process::exit(-1);
        }
    }

    /**
     * Writes a byte to the HRAM
     */
    pub fn hram_write(&mut self, mut address: u16, value: u8) -> () {
        address -= 0xFF80;
        if (address as usize) < self.hram.len() {
            self.hram[address as usize] = value;
        } else {
            log::error!("Invalid write to RAM address {:04X}", address);
            std::process::exit(-1);
        }
    }
}