pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl Cartridge {
    pub fn new(rom_file: &str) -> Cartridge {
        log::info!("Initializing cartridge...");
        let rom = std::fs::read(rom_file).unwrap();
        let ram = vec![0; 0x2000];

        Cartridge {
            rom,
            ram,
        }
    }
}