pub struct Apu { }

impl Apu {}

impl Default for Apu {
    fn default() -> Self {
        Apu {}
    }
}

impl Apu {
    fn read(self, addr: u16) -> u8 { 0xFF }
    fn write(self, addr:u16 , value :u8) { }
}
