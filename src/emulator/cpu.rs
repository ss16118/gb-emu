
pub struct CPU {

}

impl CPU {
    pub fn new() -> CPU {
        log::info!("Initializing CPU...");
        let cpu = CPU {
            
        };
        log::info!(target: "stdout", "Initializing CPU: SUCCESS");
        return cpu;
    }

    pub fn step(&mut self) -> () {
        // Fetch
        // Decode
        // Execute
    }
}