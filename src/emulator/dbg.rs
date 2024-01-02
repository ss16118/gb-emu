use crate::emulator::address_bus::*;

static mut dbg_msg: [u8; 1024] = [0x00_u8; 1024];
static mut msg_size: usize = 0;

pub fn dbg_update() -> () {
    unsafe {
        if bus_read(0xFF02) == 0x81 {
            let c = bus_read(0xFF01);
            dbg_msg[msg_size] = c;
            msg_size += 1;
            bus_write(0xFF02, 0);
        }
    }
}


pub fn dbg_print() -> () {
    if unsafe { dbg_msg[0] as u8 } > 0 {
        println!("{}", unsafe { std::str::from_utf8_unchecked(&dbg_msg) });
    }
}