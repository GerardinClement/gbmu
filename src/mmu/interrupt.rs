#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interrupt {
    VBlank = 0b00000001,
    LcdStat = 0b00000010,
    Timer = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000,
}

impl Interrupt {
    pub fn vector(self) -> u16 {
        match self {
            Interrupt::VBlank => 0x40,
            Interrupt::LcdStat => 0x48,
            Interrupt::Timer => 0x50,
            Interrupt::Serial => 0x58,
            Interrupt::Joypad => 0x60,
        }
    }
}

#[derive(Clone)]
pub struct InterruptController {
    ienable: u8,
    iflag: u8,
}

impl InterruptController {
    pub fn new() -> Self {
        InterruptController {
            ienable: 0,
            iflag: 0,
        }
    }

    pub fn read_interrupt_enable(&self) -> u8 {
        self.ienable & 0b00011111
    }

    pub fn write_interrupt_enable(&mut self, val: u8) {
        self.ienable = val & 0b00011111;
    }

    pub fn read_interrupt_flag(&self) -> u8 {
        self.iflag | 0b11100000
    }

    pub fn write_interrupt_flag(&mut self, val: u8) {
        self.iflag = val & 0b00011111;
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        self.iflag |= interrupt as u8;
    }

    pub fn clear_request(&mut self, interrupt: Interrupt) {
        self.iflag &= !(interrupt as u8);
    }

    pub fn next_request(&self) -> Option<Interrupt> {
        let pending_request = self.ienable & self.iflag;

        [
            Interrupt::VBlank,
            Interrupt::LcdStat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ]
        .iter()
        .find(|&&interrupt| pending_request & (interrupt as u8) != 0)
        .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_addresses() {
        assert_eq!(Interrupt::VBlank.vector(), 0x40);
        assert_eq!(Interrupt::LcdStat.vector(), 0x48);
        assert_eq!(Interrupt::Timer.vector(), 0x50);
        assert_eq!(Interrupt::Serial.vector(), 0x58);
        assert_eq!(Interrupt::Joypad.vector(), 0x60);
    }

    #[test]
    fn test_read_write_interrupt_enable() {
        let mut ic = InterruptController::new();
        assert_eq!(ic.read_interrupt_enable(), 0);
        ic.write_interrupt_enable(0b1111_1111);
        assert_eq!(ic.read_interrupt_enable(), 0b0001_1111);
        ic.write_interrupt_enable(0b0000_0101);
        assert_eq!(ic.read_interrupt_enable(), 0b0000_0101);
    }

    #[test]
    fn test_read_write_interrupt_flag() {
        let mut ic = InterruptController::new();
        assert_eq!(ic.read_interrupt_flag(), 0b1110_0000);
        ic.write_interrupt_flag(0b1010_1010);
        assert_eq!(ic.read_interrupt_flag(), 0b1110_1010);
        ic.write_interrupt_flag(0);
        assert_eq!(ic.read_interrupt_flag(), 0b1110_0000);
    }

    #[test]
    fn test_request_and_clear_request() {
        let mut ic = InterruptController::new();
        assert_eq!(ic.read_interrupt_flag() & 0b0001_1111, 0);
        ic.request(Interrupt::Timer);
        ic.request(Interrupt::Serial);
        assert_eq!(
            ic.read_interrupt_flag() & 0b0001_1111,
            (Interrupt::Timer as u8) | (Interrupt::Serial as u8)
        );
        ic.clear_request(Interrupt::Timer);
        assert_eq!(
            ic.read_interrupt_flag() & 0b0001_1111,
            Interrupt::Serial as u8
        );
        ic.clear_request(Interrupt::Serial);
        assert_eq!(ic.read_interrupt_flag() & 0b0001_1111, 0);
    }

    #[test]
    fn test_next_request_priority_and_masking() {
        let mut ic = InterruptController::new();
        ic.write_interrupt_enable(
            (Interrupt::VBlank as u8) | (Interrupt::Timer as u8) | (Interrupt::Joypad as u8),
        );
        for &int in &[
            Interrupt::VBlank,
            Interrupt::LcdStat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ] {
            ic.request(int);
        }
        assert_eq!(ic.next_request(), Some(Interrupt::VBlank));
        ic.clear_request(Interrupt::VBlank);
        assert_eq!(ic.next_request(), Some(Interrupt::Timer));
        ic.clear_request(Interrupt::Timer);
        assert_eq!(ic.next_request(), Some(Interrupt::Joypad));
        ic.clear_request(Interrupt::Joypad);
        assert_eq!(ic.next_request(), None);
    }
}
