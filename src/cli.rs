use clap::{Arg, ArgAction, command};

pub struct EmulatorArguments {
    pub rom_path: Option<String>,
    pub boot_rom: bool,
}

impl EmulatorArguments {
    pub fn get() -> Self {
        let matches = command!()
            .arg(
                Arg::new( "rom_path")
                    .help("The path of the rom you want to launch.")
            )
            .arg(
                Arg::new("boot_rom")
                    .short('b')
                    .long("boot_rom")
                    .action(ArgAction::SetTrue)
                    .required(false)
                    .help("If set, nintendo basic boot rom will boot first.")
            )
            .get_matches();


        // path is specified in cli command
        let rom_path = if let Some(path) = matches.get_one::<String>("rom_path") {
            Some(String::from(path))
        } else {
            None
        };

        // boot_with_nintendo_room
        let boot_rom = matches.get_flag("boot_rom");

        Self {
            rom_path,
            boot_rom,
        }
    }
}
