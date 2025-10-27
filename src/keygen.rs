#![allow(dead_code)]

use ssh_key::{Algorithm, HashAlg, PrivateKey, PublicKey, rand_core::OsRng};

use std::collections::HashMap;
use std::sync::Arc;
use std::{sync::Mutex, time};

// super::constants::KEYS_PER_THREAD;

use crate::constants;
use crate::quality;

#[derive(Debug)]
pub struct FingerprintQuality {
    pub private_key: PrivateKey,
    pub quailty: f32,
}

impl FingerprintQuality {
    fn fingerprint(&self) -> String {
        return self
            .private_key
            .public_key()
            .fingerprint(constants::FINGERPRINT_HASH_ALGORITM)
            .to_string();
    }
}

#[derive(Debug)]
pub struct BestFingerprints {
    best_keys: Vec<FingerprintQuality>,
    minimum_quaility: f32,
}

impl BestFingerprints {
    fn new() -> BestFingerprints {
        BestFingerprints {
            best_keys: Vec::new(),
            minimum_quaility: 0.0,
        }
    }

    fn get_best_keys(&self) -> &Vec<FingerprintQuality> {
        return &self.best_keys;
    }

    fn get_lowest_quality(&self) -> f32 {
        return self.minimum_quaility;
    }

    fn add(&mut self, new_key: FingerprintQuality) {
        if new_key.quailty < self.minimum_quaility {
            return;
        }

        let index = self
            .best_keys
            .binary_search_by(|x| new_key.quailty.total_cmp(&x.quailty))
            .unwrap_or_else(|x| x);

        self.best_keys.insert(index, new_key);
    }
}

pub fn worker_function(
    target_fingerprint: &str,
    best_results: Arc<Mutex<BestFingerprints>>,
    similarity: &HashMap<(u8, u8), f32>,
    attention_vec: &Vec<f32>,
) {
    let target_base64_index =
        quality::fingerprint_str_to_b64_index(quality::strip_fingerprint(target_fingerprint));

    let mut local_minimum_quaility = 0.0;

    for _ in 0..constants::KEYS_PER_THREAD {
        let private_key = PrivateKey::random(&mut OsRng, constants::KEY_TYPE).unwrap();
        let public_key = private_key.public_key();

        let fingerprint = public_key
            .fingerprint(constants::FINGERPRINT_HASH_ALGORITM)
            .to_string();

        let fingerprint_quality = FingerprintQuality {
            private_key,
            quailty: quality::fingerprint_quality(
                &target_base64_index,
                quality::strip_fingerprint(&fingerprint),
                similarity,
                attention_vec,
            ),
        };

        // Check that the last seen lowest value
        if fingerprint_quality.quailty > local_minimum_quaility {
            let mut best_results_inner = best_results.lock().unwrap();

            local_minimum_quaility = best_results_inner.minimum_quaility;

            // May or may not add
            best_results_inner.add(fingerprint_quality);
        }
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
    let pub_key = skey.public_key();

    let target_fingerprint = pub_key.fingerprint(HashAlg::Sha256).to_string();

    println!("Target: {}", target_fingerprint.to_string());

    let best_fingerprints = Arc::new(Mutex::new(BestFingerprints::new()));

    let attention_vec = quality::gen_attention_vec(&target_fingerprint);
    let similarity = quality::get_similarity_map(constants::SIMILARITY_FILEPATH);

    worker_function(
        &target_fingerprint,
        Arc::clone(&best_fingerprints),
        &similarity,
        &attention_vec,
    );

    let inner_best_keys = best_fingerprints.lock().unwrap();

    println!(
        "Result: {}",
        inner_best_keys.best_keys.first().unwrap().fingerprint()
    )
}

pub fn test_quaility() {
    let skey = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
    let binding = skey.clone();
    let pub_key = binding.public_key();

    let fingerprint = pub_key.fingerprint(HashAlg::Sha256);

    println!("Current {}", fingerprint.to_string());

    dbg!(quality::gen_attention_vec(&fingerprint.to_string()));
}

pub fn test_similarity_map() {
    let sim_map = quality::get_similarity_map("base64similarities.json");
    dbg!(&sim_map);

    assert!(sim_map.get(&(3_u8, 3_u8)) == Some(&0.0));
    assert!(*sim_map.get(&(34_u8, 35_u8)).unwrap() > 0.0);

    println!("Assertions complete");
}
