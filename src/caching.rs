use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, BufReader, Write},
    path::PathBuf,
};

use dirs::home_dir;
use rand::{distributions::Alphanumeric, Rng};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};

pub type Url = String;
pub type FileName = String;

#[derive(Debug)]
pub struct CachingSession {
    pub lookup_table: HashMap<Url, FileName>,
    pub session_name: String,
}

impl Default for CachingSession {
    fn default() -> Self {
        Self {
            lookup_table: HashMap::new(),
            session_name: String::from("session_name"),
        }
    }
}

impl CachingSession {
    const WIK_DIR: &str = ".cache/wik/caches/";
    pub fn new() -> Self {
        let mut session = Self::default();
        session.session_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        session
    }

    pub fn has_url(&self, url: &Url) -> bool {
        self.lookup_table.contains_key(url)
    }

    fn cache_dir() -> PathBuf {
        home_dir().unwrap().join(Self::WIK_DIR)
    }

    fn session_cache_dir(&self) -> PathBuf {
        Self::cache_dir().join(format!("{}/", self.session_name))
    }

    pub fn get_cache_file_path(&self, file_name: &str) -> PathBuf {
        //
        self.session_cache_dir().join(file_name)
    }

    pub fn write_to_cache<T: Serialize>(
        &mut self,
        url: &Url,
        serializable_object: T,
    ) -> Result<(), Box<dyn Error>> {
        let file_name = create_hash(url);

        let file_path = self.get_cache_file_path(&file_name);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(file_path)?;

        let json_data = serde_json::to_string(&serializable_object)?;
        file.write_all(json_data.as_bytes())?;

        self.lookup_table.insert(url.clone(), file_name);

        Ok(())
    }

    pub fn get_from_cache<T: DeserializeOwned>(&self, url: &Url) -> Option<T> {
        match self.lookup_table.get(url) {
            Some(file_name) => {
                // get from the file system
                let file_result = File::options()
                    .read(true)
                    .write(false)
                    .open(self.get_cache_file_path(file_name));
                if let Err(_) = file_result {
                    return None;
                } else {
                    let reader = BufReader::new(file_result.unwrap());
                    match serde_json::from_reader::<_, T>(reader) {
                        Ok(deserialized_object) => Some(deserialized_object),
                        Err(_) => None,
                    }
                }
            }
            None => None,
        }
    }
    pub fn clear_caches() -> io::Result<()> {
        match fs::remove_dir_all(Self::cache_dir()) {
            Ok(_) => match fs::create_dir(Self::cache_dir()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    println!("Error creating files!");
                    Err(e)
                }
            },
            Err(e) => {
                println!("Error removing files!");
                Err(e)
            }
        }
    }
}

fn create_hash(msg: &str) -> String {
    let mut hasher = Sha256::default();
    hasher.update(msg);
    format!("{:x}", hasher.finalize()).as_str()[0..10].to_string()
}
