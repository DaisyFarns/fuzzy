use ssh_key::{Algorithm, HashAlg, PrivateKey, PublicKey, rand_core::OsRng};

use std::{sync::Mutex, time};

// super::constants::KEYS_PER_THREAD;

use crate::constants;
use crate::quality;

pub struct BestFingerprint {
    public_key: PublicKey,
    private_key: PrivateKey,
    fingerprint: ssh_key::Fingerprint,
    quailty: f32,
}

pub fn worker_function(
    target_fingerprint: &str,
    best_result: Mutex<BestFingerprint>,
) {
    for _ in 0..constants::KEYS_PER_THREAD {
        let private_key =
            PrivateKey::random(&mut OsRng, constants::KEY_TYPE).unwrap();
        let public_key = private_key.public_key();

        let fingerprint =
            public_key.fingerprint(constants::FINGERPRINT_HASH_ALGORITM);

        // Process fingerprint, check quailty
    }
}

#[allow(dead_code)]
pub fn test_fingerprint_generation() {
    let mut fingerprints = Vec::new();

    let t = time::Instant::now();

    for _ in 0..10_000 {
        let skey = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let pub_key = skey.public_key();
        fingerprints.push(pub_key.fingerprint(HashAlg::Sha256));
    }

    println!("Done in {}ms", t.elapsed().as_millis());
}

pub fn test_worker() {
    let skey = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
    let binding = skey.clone();
    let pub_key = binding.public_key();

    let fingerprint = pub_key.fingerprint(HashAlg::Sha256);

    let best = BestFingerprint {
        private_key: skey,
        public_key: pub_key.clone(),
        fingerprint: fingerprint,
        quailty: 0.0,
    };

    worker_function(
        "+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU",
        Mutex::new(best),
    );
}

pub fn test_quaility() {
    let skey = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
    let binding = skey.clone();
    let pub_key = binding.public_key();

    let fingerprint = pub_key.fingerprint(HashAlg::Sha256);

    println!("Current {}", fingerprint.to_string());

    dbg!(quality::gen_attention_map(&fingerprint.to_string()));
}
