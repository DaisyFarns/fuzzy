use ssh_key::{Algorithm, HashAlg};
use std::time;

pub const KEYS_PER_THREAD: u32 = 10_000;
pub const FINGERPRINT_HASH_ALGORITM: HashAlg = HashAlg::Sha256;
pub const KEY_TYPE: Algorithm = Algorithm::Ed25519;
pub const BASE_64_FINGERPRINT_LENGTH: usize = 43;
pub const SIMILARITY_FILEPATH: &str = "base64similarities.json";
pub const SLEEP_DURATION: time::Duration = time::Duration::from_secs(10);
pub const BEST_KEYS_NUMBER: usize = 30; // The number of best keys to keep
