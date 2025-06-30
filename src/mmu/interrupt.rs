#[derive(Copy, Clone, Debug, PartialEq)]
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
        self.iflag | 0b11100000
    }

    pub fn write_if(&mut self, val: u8) {
        self.iflag = val & 0b00011111;
    }

    pub fn request(&mut self, interrupt: Interrupt) {
        let mask = 1 << (interrupt as u8);
        self.iflag |= mask;
    }

    pub fn clear_request(&mut self, interrupt: Interrupt) {
        let reversed_mask = !(1 << (interrupt as u8));
        self.iflag &= reversed_mask;
    }

    pub fn next_request(&self) -> Option<Interrupt> {
        let pending_request = self.ienable & self.iflag;

        for &interrupt in &[
            Interrupt::VBlank,
            Interrupt::LcdStat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ] {
            let mask = 1 << (interrupt as u8);
    
            if pending_request & mask != 0 {
                return Some(interrupt);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_addresses() {
        assert_eq!(Interrupt::VBlank.vector(), 0x40);
        assert_eq!(Interrupt::LcdStat.vector(), 0x48);
        assert_eq!(Interrupt::Timer.vector(),  0x50);
        assert_eq!(Interrupt::Serial.vector(), 0x58);
        assert_eq!(Interrupt::Joypad.vector(), 0x60);
    }

    #[test]
    fn test_read_write_ie() {
        let mut ic = InterruptController::new();

        // IE starts at 0
        assert_eq!(ic.read_ie(), 0);

        // Write some bits (only lower 5 should stick)
        ic.write_ie(0b1111_1111);
        assert_eq!(ic.read_ie(), 0b0001_1111);

        ic.write_ie(0b0000_0101);
        assert_eq!(ic.read_ie(), 0b0000_0101);
    }

    #[test]
    fn test_read_write_if() {
        let mut ic = InterruptController::new();

        // IF starts at 0, but read_if forces high bits 5–7
        assert_eq!(ic.read_if(), 0b1110_0000);

        // Write lower bits only
        ic.write_if(0b1010_1010);
        // stored = 0b0001_01010, read_if ORs with 0b1110_0000
        assert_eq!(ic.read_if(), 0b1110_1010);

        // Clear it back to zero
        ic.write_if(0);
        assert_eq!(ic.read_if(), 0b1110_0000);
    }

    #[test]
    fn test_request_and_clear_request() {
        let mut ic = InterruptController::new();

        // No flags initially
        assert_eq!(ic.read_if() & 0b0001_1111, 0);

        // Request Timer and Serial
        ic.request(Interrupt::Timer);
        ic.request(Interrupt::Serial);
        assert_eq!(
            ic.read_if() & 0b0001_1111,
            (1 << Interrupt::Timer as u8) | (1 << Interrupt::Serial as u8)
        );

        // Clear only Timer
        ic.clear_request(Interrupt::Timer);
        assert_eq!(
            ic.read_if() & 0b0001_1111,
            1 << (Interrupt::Serial as u8)
        );

        // Clear Serial too
        ic.clear_request(Interrupt::Serial);
        assert_eq!(ic.read_if() & 0b0001_1111, 0);
    }

    #[test]
    fn test_next_request_priority_and_masking() {
        let mut ic = InterruptController::new();

        // Enable VBlank, Timer, Joypad
        ic.write_ie((1 << Interrupt::VBlank as u8)
                  | (1 << Interrupt::Timer   as u8)
                  | (1 << Interrupt::Joypad  as u8));

        // Request all five
        for &int in &[Interrupt::VBlank, Interrupt::LcdStat, Interrupt::Timer, Interrupt::Serial, Interrupt::Joypad] {
            ic.request(int);
        }

        // next_request should return VBlank first (highest priority)
        assert_eq!(ic.next_request(), Some(Interrupt::VBlank));
        ic.clear_request(Interrupt::VBlank);

        // Now LcdStat is requested but not enabled → skip to Timer
        assert_eq!(ic.next_request(), Some(Interrupt::Timer));
        ic.clear_request(Interrupt::Timer);

        // Next enabled+requested is Joypad
        assert_eq!(ic.next_request(), Some(Interrupt::Joypad));
        ic.clear_request(Interrupt::Joypad);

        // Only LcdStat and Serial remain requested, but neither is enabled
        assert_eq!(ic.next_request(), None);
    }
}
