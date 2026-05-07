use std::path::PathBuf;
use std::{fs::File, fs};
use std::io::{BufRead, BufReader};
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
    path: PathBuf, //Path of the file, stored once to avoid having to home_dir() everytime
    history: Vec<PlayedRoms>,
}

impl Default for GmbuFile {
	fn default() -> Self {
		Self { history: vec![],  path: PathBuf::new()}
	}
}

impl GmbuFile {

    fn new(path: PathBuf) -> Self {

        println!("Creating new file!");

        let dir = path.parent().expect("Path has no parent directory");
        fs::create_dir_all(dir).expect("Could not create folder for storing the data!");
        let _ = File::create(&path).expect("Could not create ~/.gbmu/gbmu!");
        Self {
            path: path,
            ..Default::default()
        }
    }

    fn read_existing(f: File, path: PathBuf) -> Self {
        println!("Reading existing file!");

        let reader = BufReader::new(f);
        
        let history = reader.lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 3 {
                return None
            }
            let last_launched = parts[0].parse::<DateTime<Utc>>().ok()?;
            Some(PlayedRoms {
                last_launched,
                rom_name: parts[1].to_string(),
                rom_path: parts[2].to_string(),
            })
        })
        .collect();


        Self { history, path }
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
                    return GmbuFile::new(path)
                },
                _ => {
                    panic!("Something went wrong opening ~/.gbmu/gbmu -> {error:?}.\nIf you think this is an error, delete it and restart the program to create a fresh config.");
                }
            }
        };
        GmbuFile::read_existing(file, path)
    }
}
    