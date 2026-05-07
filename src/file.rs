use std::path::PathBuf;
use std::{fs, fs::File};
use std::io::ErrorKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayedRom {
    pub last_launched: DateTime<Utc>,
    pub rom_name: String,
    pub rom_path: PathBuf,
    pub launch_count: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GbmuFile {
    pub history: Vec<PlayedRom>,

    #[serde(skip)]
    pub path: PathBuf,
}

impl GbmuFile {
    pub fn get_existing_or_new() -> Self {
        let path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".gbmu/gbmu.json");

        match File::open(&path) {
            Ok(file) => {
                println!("Reading existing file!");
                let mut gbmu: GbmuFile = serde_json::from_reader(file)
                    .unwrap_or_else(|e| {
                        eprintln!("Warning: Could not parse config, starting fresh: {e}");
                        GbmuFile::default()
                    });
                gbmu.path = path;
                gbmu
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("Creating new file!");
                let dir = path.parent().expect("Path has no parent directory");
                fs::create_dir_all(dir).expect("Could not create ~/.gbmu/");
                let mut gbmu = GbmuFile::default();
                gbmu.path = path;
                gbmu.persist(); // write empty JSON so the file exists
                gbmu
            }
            Err(e) => panic!(
                "Something went wrong opening ~/.gbmu/gbmu.json -> {e:?}.\n\
                 If you think this is an error, delete it and restart to create a fresh config."
            ),
        }
    }

    pub fn record_launch(&mut self, rom_name: String, rom_path: PathBuf) {
        if let Some(entry) = self.history.iter_mut().find(|r| r.rom_path == rom_path) {
            entry.last_launched = Utc::now();
            entry.launch_count += 1;
        } else {
            self.history.push(PlayedRom {
                last_launched: Utc::now(),
                rom_name,
                rom_path,
                launch_count: 1,
            });
        }
        self.persist();
    }

    // pub fn recent(&self) -> Vec<&PlayedRom> {
    //     let mut sorted: Vec<&PlayedRom> = self.history.iter().collect();
    //     sorted.sort_by(|a, b| b.last_launched.cmp(&a.last_launched));
    //     sorted
    // }

    fn persist(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.path, json) {
                    eprintln!("Warning: Could not write to {:?}: {e}", self.path);
                }
            }
            Err(e) => eprintln!("Warning: Could not serialize config: {e}"),
        }
    }
}