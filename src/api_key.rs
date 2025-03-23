use rand::{distributions::Alphanumeric, Rng};
use std::fs;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::hashing::hash_api_key;

// Define the structure for storing API keys
#[derive(Serialize, Deserialize, Debug, Default)]
struct ApiKeyStore {
    api_keys: HashMap<String, HashedApiKey>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct HashedApiKey {
    hashed_api_key: String,
}

lazy_static! {
    static ref API_KEYS: Mutex<ApiKeyStore> = Mutex::new(ApiKeyStore::default());
}

// Function to generate a cryptographically secure API key
pub fn generate_api_key() -> String {
    let api_key: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    api_key
}

// Function to store the API key in store.json
pub fn store_api_key(api_key: String) {
    let hashed_api_key = hash_api_key(&api_key);
    let mut api_keys = API_KEYS.lock().unwrap();
    api_keys.api_keys.insert(api_key.clone(), HashedApiKey { hashed_api_key });

    // Serialize the API key store to JSON and write it to the file with indentation
    let api_keys_json = serde_json::to_string_pretty(&api_keys.api_keys).expect("Unable to serialize API keys");
    fs::write("store.json", api_keys_json).expect("Unable to write to file");
}

// Function to verify the API key
pub fn verify_api_key(api_key: &String) -> bool {
    let hashed_api_key = hash_api_key(api_key);
    let api_keys = API_KEYS.lock().unwrap();
    if let Some(stored_key) = api_keys.api_keys.get(api_key) {
        stored_key.hashed_api_key == hashed_api_key
    } else {
        false
    }
}

// Function to load API keys from store.json
pub fn load_api_keys() {
    match fs::read_to_string("store.json") {
        Ok(data) => {
            if data.is_empty() {
                println!("Info: store.json is empty, using default API key store.");
            } else {
                match serde_json::from_str::<HashMap<String, HashedApiKey>>(&data) {
                    Ok(api_key_store) => {
                        let mut api_keys = API_KEYS.lock().unwrap();
                        api_keys.api_keys = api_key_store;
                    }
                    Err(e) => {
                        eprintln!("Error: Unable to deserialize API keys from store.json: {}", e);
                        // If deserialization fails, create a default ApiKeyStore
                        let mut api_keys = API_KEYS.lock().unwrap();
                        api_keys.api_keys = HashMap::new();
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Error: store.json not found.");
        }
    }
}
