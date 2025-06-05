#![allow(unused_variables)]
#![allow(dead_code)]

const DIV_ADDRESS: u16 = 0xFF04;
const TIMA_ADDRESS: u16 = 0xFF04;
const TMA_ADDRESS: u16 = 0xFF04;
const TAC_ADDRESS: u16 = 0xFF04;

pub struct timer {
	div: u8,
	tima: u8,
	tma: u8,
	tac: u8
}

