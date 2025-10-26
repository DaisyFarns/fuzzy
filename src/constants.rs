use ssh_key::{Algorithm, HashAlg};

pub const KEYS_PER_THREAD: u32 = 10_000;
pub const FINGERPRINT_HASH_ALGORITM: HashAlg = HashAlg::Sha256;
pub const KEY_TYPE: Algorithm = Algorithm::Ed25519;
pub const BASE_64_FINGERPRINT_LENGTH: usize = 43;
