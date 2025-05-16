mod registers;
mod flags_registers;

use crate::flags_registers::FlagsRegister;

fn main() {
    let flags = FlagsRegister {
        zero: true,
        subtract: false,
        half_carry: true,
        carry: false,
    };

    println!("{}", flags.zero);
    // Utilisation...
}
