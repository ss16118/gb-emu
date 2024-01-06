
/**
 * Game Pad state
 * FIXME: I know it is not the best practice
 * to make all the fields public, but I
 * don't want to write getters and setters, lol
 */
pub struct GamePadState {
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub struct GamePad {
    button_select: bool,
    dir_select: bool,
    pub controller: GamePadState,
}

pub static mut GAMEPAD_CTX: GamePad = GamePad {
    button_select: false,
    dir_select: false,
    controller: GamePadState {
        start: false,
        select: false,
        a: false,
        b: false,
        up: false,
        down: false,
        left: false,
        right: false,
    },
};

impl GamePad {
    /**
     * Returns **FALSE** if the button mode is selected, 
     * i.e., the lower nibble of the input indicates which
     * button (SsBA) is pressed.
     * true otherwise.
     */
    pub fn button_select(&mut self) -> bool {
        return self.button_select;
    }

    /**
     * Returns **FALSE** if the direction mode is selected, 
     * i.e, the lower nibble of the input indicates which
     * direction key is pressed.
     * true otherwise.
     */
    pub fn dir_select(&mut self) -> bool {
        return self.dir_select;
    }

    /**
     * Set the state of the game pad
     */
    pub fn set_select(&mut self, value: u8) -> () {
        self.button_select = (value & 0x20) != 0;
        self.dir_select = (value & 0x10) != 0;
    }

    /**
     * 
     */
    pub fn get_output(&mut self) -> u8 {
        let mut output = 0xCF;

        // If the button mode is selected
        if !self.button_select() {
            if self.controller.start {
                // If start is pressed, turn off bit 3
                output &= !(1 << 3);
            }
            if self.controller.select {
                // If select is pressed, turn off bit 2
                output &= !(1 << 2);
            }
            if self.controller.b {
                // If b is pressed, turn off bit 1
                output &= !(1 << 1);
            }
            if self.controller.a {
                // If a is pressed, turn off bit 0
                output &= !(1 << 0);
            }
        }

        // If the direction mode is selected
        if !self.dir_select() {
            if self.controller.right {
                // If right is pressed, turn off bit 0
                output &= !(1 << 0);
            }
            if self.controller.left {
                // If left is pressed, turn off bit 1
                output &= !(1 << 1);
            }
            if self.controller.up {
                // If up is pressed, turn off bit 2
                output &= !(1 << 2);
            }
            if self.controller.down {
                // If down is pressed, turn off bit 3
                output &= !(1 << 3);
            }

        }

        return output;
    }

    #[allow(dead_code)]
    pub fn get_state(&self) -> &GamePadState {
        return &self.controller;
    }
}