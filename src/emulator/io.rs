
static mut serial_data: [u8; 2] = [0, 0];


/**
 * Reads a byte from the given address from the I/O registers
 */
pub fn io_read(address: u16) -> u8 {
    if address == 0xFF01 {
        return unsafe { serial_data[0] };
    } else if address == 0xFF02 {
        return unsafe { serial_data[1] };
    }
    log::error!("Reading from I/O address 0x{:04X} currently not supported", address);
    return 0;
}


/**
 * Writes a byte to the given address
 */
pub fn io_write(address: u16, data: u8) -> () {
    if address == 0xFF01 {
        unsafe { serial_data[0] = data };
    } else if address == 0xFF02 {
        unsafe { serial_data[1] = data };
    }
    log::error!("Writing to I/O address 0x{:04X} currently not supported", address);
    return;
}