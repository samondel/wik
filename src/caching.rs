use std::{
    error::Error,
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use digest::{generic_array::GenericArray, Digest};
use dirs::home_dir;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::header::SEC_WEBSOCKET_EXTENSIONS;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha512};

#[derive(Serialize, Deserialize, Debug)]
pub struct CacheEntry {
    url: String,
    file_name: String,
}

#[derive(Debug)]
pub struct CachingSession {
    pub lookup_table: Vec<CacheEntry>,
    pub session_name: String,
}

impl Default for CachingSession {
    fn default() -> Self {
        Self {
            lookup_table: Default::default(),
            session_name: Default::default(),
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
    format!("{:X}", hasher.finalize())
}
/*
pub fn save_dummy_lookup() -> Result<(), Box<dyn Error>> {
    let mut cache_entries = Vec::new();

    let dummy_1 = CacheEntry {
        url: "url1".to_string(),
        file_name: "file_name1".to_string(),
    };
    cache_entries.push(dummy_1);

    let cache_path = get_cache_file_path();

    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&cache_path)?;

    serde_json::to_writer(file, &cache_entries)?;

    Ok(())
}
*/

// pub fn use_cache<T: DeserializeOwned + Serialize>(url: &str) -> Result<T, Box<dyn Error>> {
// 1. Try to find if file is saved somewhere, looking up url to find if a json file for it exists
// 2. If the json file exists, open and deserialize it, return it
// 3. If not, make the GET request, save the json in the response, and return the corresponding object

// let file_path = match get_file_path_for_url(url) {

// }
// // let file_result = File::create(file_path);

// // serde_json::to_writer_pretty(file?, search_response)
// let response = client.get(&request_url).send()?.json::<T>()?;
// }
