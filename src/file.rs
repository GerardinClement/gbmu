use std::fs::File;
use std::io::ErrorKind;

use chrono::{DateTime, Utc};

// We could store the fd then we wouldn't have to open it everytime we wanna read/write but that'd be lame asf
// Instead we just gonna update everytime there's a change inside one of these struct

pub struct PlayedRoms {
    last_launched: DateTime<Utc>,
    rom_name: String,
    rom_path: String
}

pub struct GmbuFile {
    history: Vec<PlayedRoms>
}

impl Default for GmbuFile {
	fn default() -> Self {
		Self { history: vec![] }
	}
}

impl GmbuFile {

    fn new() -> Self {
        println!("Creating new file!");
        Default::default()
    }

    fn read_existing(f: File) -> Self {
        println!("Reading existing file!");
        Default::default() 
    }

    pub fn get_existing_or_new() -> Self {
        let path = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".gbmu/gbmu");

        // -> If no file create a new one 
        // -> if permissions are fucked or open fails for whatever reason, go nahui we shutdown and tell the user to delete everything
        // -> if everything good we parse it
        
        let f_res = File::open(&path);
        let file = match f_res { 
            Ok(file) => file,
            Err(error) => match error.kind() {
                ErrorKind::NotFound =>  {
                    return GmbuFile::new()
                },
                _ => {
                    panic!("Something went wrong opening ~/.gbmu/gbmu -> {error:?}.\nIf you think this is an error, delete it and restart the program to create a fresh config.");
                }
            }
        };
        GmbuFile::read_existing(file)
    }
}
    