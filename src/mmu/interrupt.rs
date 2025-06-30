pub enum Interrupt {
    VBlank = 0, // 0x40
    LcdStat = 1, // 0x48
    Timer = 2, // 0x50 
    Serial = 3, // 0x58
    Joypad = 4, // 0x60
}

impl Interrupt {
    pub fn vector(self) -> u16 {
        0x40 + ((self as u16) * 8) // 0x40 = 64 so 64 + (index * 8) gives the right vector
    }
}

pub struct InterruptController {
    ienable: u8,
    iflag: u8,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController { ienable: 0, iflag: 0 }
    }

    pub fn read_ie(&self) -> u8 {
        self.ienable & 0b00011111
    }

    pub fn write_ie(&mut self, val: u8) {
        self.ienable = val & 0b00011111;
    }

    pub fn read_if(&self) -> u8 {
        self.ienable | 0b11100000
    }

    pub fn write_if(&mut self, val: u8) {
        self.iflag = val & 0b00011111;
    }

    pub fn request(&mut self, int: Interrupt) {
        let mask = 1 << (int as u8);
        self.iflag |= mask;
    }

    pub fn clear_flag(&mut self, int: Interrupt) {
        let reversed_mask = !(1 << (int as u8));
        self.iflag &= reversed_mask;
    }

//     pub fn next(&self) -> Option<Interrupt> {}
}