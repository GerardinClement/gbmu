#[derive(Default)]
pub struct Timers {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    previous_and_result: bool,
}

const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;

impl Timers {
    pub fn tick(&mut self) -> bool {
        self.div = self.div.wrapping_add(1);
        let enabled = (self.tac & 0b100) > 0;
        let mask = 0b1
            << match self.tac & 0b11 {
                0b00 => 9,
                0b01 => 3,
                0b10 => 5,
                0b11 => 7,
                _ => unreachable!(),
            };

        let kept_bit = (self.div & mask) > 0;
        let and_result = kept_bit && enabled;

        let mut overflowed = false;
        if self.previous_and_result && !and_result {
            let result = self.tima.wrapping_add(1);
            if result == 0 {
                self.tima = self.tma;
                overflowed = true
            } else {
                self.tima = result;
            }
        }
        self.previous_and_result = and_result;
        overflowed
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            DIV_ADDR => self.div = 0,
            TIMA_ADDR => self.tima = value,
            TMA_ADDR => self.tma = value,
            TAC_ADDR => self.tac = value,
            _ => unreachable!(),
        }
    }
    pub fn read_byte(&self, addr: u16) -> u8 {
        let a_box = Box::new(18);
        match addr {
            DIV_ADDR => (self.div >> 8) as u8,
            TIMA_ADDR => self.tima,
            TMA_ADDR => self.tma,
            TAC_ADDR => self.tac,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let timer = Timers::default();
        assert_eq!(timer.div, 0);
        assert_eq!(timer.tima, 0);
        assert_eq!(timer.tma, 0);
        assert_eq!(timer.tac, 0);
    }

    #[test]
    fn test_div_is_ticking_once_by_tick() {
        let mut timer = Timers::default();

        for _ in 0..20 {
            timer.tick();
        }

        assert_eq!(timer.div, 20);
    }

    fn test_read_div_exposed_register_ticks_once_every_overflow() {
        let mut timer = Timers::default();

        let tick_value = timer.read_byte(DIV_ADDR);
        assert_eq!(tick_value, 0);
        for _ in 0..66000 {
            timer.tick();
        }
        let tick_value = timer.read_byte(DIV_ADDR);
        assert_eq!(tick_value, 1);
    }

    #[test]
    fn test_div_reset_when_written_to() {
        let mut timer = Timers::default();

        timer.tick();
        assert_ne!(timer.div, 0);
        timer.write_byte(DIV_ADDR, 27);
        assert_eq!(timer.div, 0);
    }

    #[test]
    fn test_other_than_div_register_has_read_and_write_possible() {
        let vec = vec![TIMA_ADDR, TMA_ADDR, TAC_ADDR];
        let mut timers = Timers::default();
        let value = 42;

        for addr in &vec {
            timers.write_byte(*addr, value);
        }
        assert_eq!(value, timers.tima);
        assert_eq!(value, timers.tma);
        assert_eq!(value, timers.tac);

        for addr in vec {
            assert_eq!(value, timers.read_byte(addr));
        }
    }

    #[test]
    fn test_tac_disable_tima_ticking() {
        let mut timers = Timers::default();

        for _ in 0..66000 {
            timers.tick();
        }
        assert_eq!(0, timers.tima);

        timers.write_byte(TAC_ADDR, 0b101);
        for _ in 0..66000 {
            timers.tick();
        }
        assert_ne!(0, timers.tima);
    }

    #[test]
    fn test_tima_tick_at_good_pace_bit_3() {
        let mut timers = Timers::default();

        timers.write_byte(TAC_ADDR, 0b101);
        for _ in 0..15 {
            timers.tick();
            assert_eq!(0, timers.tima);
        }
        timers.tick();
        assert_eq!(1, timers.tima);
    }

    #[test]
    fn test_tima_tick_at_good_pace_bit_5() {
        let mut timers = Timers::default();

        timers.write_byte(TAC_ADDR, 0b110);
        for _ in 0..63 {
            timers.tick();
            assert_eq!(0, timers.tima);
        }
        timers.tick();
        assert_eq!(1, timers.tima);
    }

    #[test]
    fn test_tima_tick_at_good_pace_bit_7() {
        let mut timers = Timers::default();

        timers.write_byte(TAC_ADDR, 0b111);
        for _ in 0..255 {
            timers.tick();
            assert_eq!(0, timers.tima);
        }
        timers.tick();
        assert_eq!(1, timers.tima);
    }

    #[test]
    fn test_tima_tick_at_good_pace_bit_9() {
        let mut timers = Timers::default();

        timers.write_byte(TAC_ADDR, 0b100);
        for _ in 0..1023 {
            timers.tick();
            assert_eq!(0, timers.tima);
        }
        timers.tick();
        assert_eq!(1, timers.tima);
    }

    #[test]
    fn test_timer_tick_overflowing_returns_true() {
        let mut timers = Timers {
            tima: 0xFF,
            ..Default::default()
        };
        timers.write_byte(TAC_ADDR, 0b101);
        for a in 0..15 {
            assert_eq!(false, timers.tick(), "overflow comming for : {a}");
        }
        assert_eq!(true, timers.tick());
    }

    #[test]
    fn test_timer_overflowing_reset_tima_to_tma() {
        let test_value = 0x53;
        let mut timers = Timers {
            tima: 0xFF,
            tma: 0x53,
            ..Default::default()
        };
        timers.write_byte(TAC_ADDR, 0b101);
        for a in 0..=15 {
            timers.tick();
        }
        assert_eq!(timers.tima, timers.tma);
    }
}
