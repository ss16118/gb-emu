use std::collections::LinkedList;

/**
 * Implementation of everything related to the Pixel FIFO
 * https://gbdev.io/pandocs/pixel_fifo.html
 */


/**
 * Enum representation of different states
 * of fetching pixels
 */
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum FetchState {
    FS_TILE,
    FS_TILE_DATA_LOW,
    FS_TILE_DATA_HIGH,
    FS_IDLE,
    FS_PUSH,
}


pub struct PixelFifo {
    pub curr_state: FetchState,
    fifo: LinkedList<u32>,
    pub line_x: u8,
    pub pushed_x: u8,
    pub fetch_x: u8,
    pub bgw_fetch_data: [u8; 3],
    // OAM data
    pub fetch_entry_data: [u8; 6],
    pub map_y: u8,
    pub map_x: u8,
    pub tile_y: u8,
    pub fifo_x: u8,
}


impl PixelFifo {
    pub fn new() -> PixelFifo {
        PixelFifo {
            curr_state: FetchState::FS_TILE,
            fifo: LinkedList::new(),
            line_x: 0,
            pushed_x: 0,
            fetch_x: 0,
            bgw_fetch_data: [0; 3],
            fetch_entry_data: [0; 6],
            map_y: 0,
            map_x: 0,
            tile_y: 0,
            fifo_x: 0,
        }
    }

    /**
     * Pushes a pixel to the FIFO
     */
    pub fn push(&mut self, data: u32) -> () {
        self.fifo.push_back(data);
    }

    /**
     * Pops a pixel from the FIFO
     */
    pub fn pop(&mut self) -> u32 {
        if self.fifo.len() == 0 {
            log::error!("Attempted to pop from an empty Pixel FIFO");
            std::process::exit(1);
        }
        return self.fifo.pop_front().unwrap();
    }

    /**
     * Returns the size of the FIFO.
     * A wrapper around the LinkedList's len() method of
     * the internal linked list.
     */
    pub fn get_size(&self) -> usize {
        return self.fifo.len();
    }

    /**
     * Clears the FIFO.
     */
    pub fn clear(&mut self) -> () {
        self.fifo.clear();
    }

    /**
     * Resets some of the internal fields of the FIFO.
     * Note that this function does not clear the FIFO queue.
     */
    pub fn reset(&mut self) -> () {
        self.curr_state = FetchState::FS_TILE;
        self.line_x = 0;
        self.pushed_x = 0;
        self.fetch_x = 0;
        self.fifo_x = 0;
    }

}