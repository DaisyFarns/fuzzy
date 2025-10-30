#![allow(dead_code)]

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ssh_key::LineEnding;
use ssh_key::{Algorithm, HashAlg, PrivateKey, rand_core::OsRng};

use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, iter, path, thread};
use std::{sync::Mutex, time};

// super::constants::KEYS_PER_THREAD;

use crate::constants;
use crate::quality;

#[derive(Debug)]
pub struct FingerprintQuality {
    pub private_key: PrivateKey,
    pub quality: f32,
}

impl FingerprintQuality {
    fn fingerprint(&self) -> String {
        return quality::strip_fingerprint(
            &self
                .private_key
                .public_key()
                .fingerprint(constants::FINGERPRINT_HASH_ALGORITHM)
                .to_string(),
        )
        .to_string();
    }
}

#[derive(Debug)]
pub struct BestFingerprints {
    best_keys: Vec<FingerprintQuality>,
    minimum_quality: f32,
}

impl BestFingerprints {
    fn new() -> BestFingerprints {
        BestFingerprints {
            best_keys: Vec::new(),
            minimum_quality: 0.0,
        }
    }

    fn get_best_keys(&self) -> &Vec<FingerprintQuality> {
        return &self.best_keys;
    }

    fn get_lowest_quality(&self) -> f32 {
        return self.minimum_quality;
    }

    fn add(&mut self, new_key: FingerprintQuality) {
        if new_key.quality < self.minimum_quality {
            return;
        }

        let index = self
            .best_keys
            .binary_search_by(|x| new_key.quality.total_cmp(&x.quality))
            .unwrap_or_else(|x| x);

        self.best_keys.insert(index, new_key);

        if self.best_keys.len() > constants::BEST_KEYS_NUMBER {
            self.best_keys.pop();

            self.minimum_quality = self.best_keys.last().expect("Checked length").quality;
        }
    }

    // Save a checkpoint
    fn save_checkpoint(&self, target_fingerprint: &str) {
        // See if Keys directory exists, if not create it

        if !fs::exists(constants::KEYS_DIRECTORY).unwrap() {
            fs::create_dir(constants::KEYS_DIRECTORY).unwrap();
        }

        let private_keys: Vec<_> = self
            .best_keys
            .iter()
            .map(|x| {
                x.private_key
                    .to_openssh(LineEnding::default())
                    .unwrap()
                    .to_string()
            })
            .collect();

        let checkpoint = json!({
            "target": target_fingerprint,
            "best_private_keys": private_keys
        });

        let checkpoint_filepath =
            path::Path::new(constants::KEYS_DIRECTORY).join(constants::CHECKPOINT_FILENAME);
        let checkpoint_file = fs::File::create(checkpoint_filepath).unwrap();

        // Write checkpoint to file
        serde_json::to_writer_pretty(checkpoint_file, &checkpoint).unwrap();

        // Create spoofed ssh public and private keys

        for (index, key_info) in self.best_keys.iter().enumerate() {
            let private_key = &key_info.private_key;

            let private_key_string = private_key.to_openssh(LineEnding::default()).unwrap();

            let public_key_string = private_key.public_key().to_openssh().unwrap();

            let key_path = path::Path::new(constants::KEYS_DIRECTORY);
            let private_key_file = key_path.join(format!("{}_id_spoof", index));
            let public_key_file = key_path.join(format!("{}_id_spoof.pub", index));

            fs::File::create(private_key_file)
                .unwrap()
                .write(private_key_string.as_bytes())
                .unwrap();

            fs::File::create(public_key_file)
                .unwrap()
                .write(public_key_string.as_bytes())
                .unwrap();
        }
    }

    fn from_checkpoint(data: &CheckPoint) -> BestFingerprints {
        let mut best_keys = Vec::new();

        let attention = quality::gen_attention_vec(&data.target);
        let similarity = quality::get_similarity_map(constants::SIMILARITY_FILEPATH);

        let target_base64_index =
            quality::fingerprint_str_to_b64_index(quality::strip_fingerprint(&data.target));

        for private_key_text in &data.best_private_keys {
            let private_key =
                PrivateKey::from_openssh(private_key_text).expect("Checkpoint file corrupted");
            let fingerprint = private_key
                .public_key()
                .fingerprint(constants::FINGERPRINT_HASH_ALGORITHM)
                .to_string();

            let fingerprint_quality = FingerprintQuality {
                private_key,
                quality: quality::fingerprint_quality(
                    &target_base64_index,
                    quality::strip_fingerprint(&fingerprint),
                    &similarity,
                    &attention,
                ),
            };

            best_keys.push(fingerprint_quality);
        }

        let minimum_quality = best_keys
            .iter()
            .map(|x| x.quality)
            .min_by(|x, y| x.total_cmp(y))
            .expect("Checkpoint file corrupt");

        return BestFingerprints {
            best_keys,
            minimum_quality,
        };
    }
}

pub fn worker_function(
    target_fingerprint: &str,
    best_results: Arc<Mutex<BestFingerprints>>,
    similarity: &HashMap<(u8, u8), f32>,
    attention_vec: &Vec<f32>,
    total_count: Arc<AtomicU64>,
) {
    let target_base64_index =
        quality::fingerprint_str_to_b64_index(quality::strip_fingerprint(target_fingerprint));

    let mut local_minimum_quality = 0.0;

    for _ in 0..constants::KEYS_PER_THREAD {
        let private_key = PrivateKey::random(&mut OsRng, constants::KEY_TYPE).unwrap();
        let public_key = private_key.public_key();

        let fingerprint = public_key
            .fingerprint(constants::FINGERPRINT_HASH_ALGORITHM)
            .to_string();

        let fingerprint_quality = FingerprintQuality {
            private_key,
            quality: quality::fingerprint_quality(
                &target_base64_index,
                quality::strip_fingerprint(&fingerprint),
                similarity,
                attention_vec,
            ),
        };

        // Check that the last seen lowest value
        if fingerprint_quality.quality > local_minimum_quality {
            let mut best_results_inner = best_results.lock().unwrap();

            local_minimum_quality = best_results_inner.minimum_quality;

            // May or may not add
            best_results_inner.add(fingerprint_quality);
        }

        total_count.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn display_status_thread(
    best_keys: Arc<Mutex<BestFingerprints>>,
    target: &str,
    start_time: time::Instant,
    total_keys: Arc<AtomicU64>,
) {
    loop {
        thread::sleep(constants::SLEEP_DURATION);

        let inner = best_keys.lock().unwrap();
        let best_result = inner.best_keys.first().expect("No fingerprints yet");
        inner.save_checkpoint(target);

        let key_gen_rate =
            total_keys.load(Ordering::Relaxed) as f64 / start_time.elapsed().as_secs_f64();

        println!("");
        println!(
            "Time: {} ({:.2}k keys / s)",
            start_time.elapsed().as_secs(),
            key_gen_rate / 1000.0
        );
        println!("Best Quality: {}", best_result.quality);
        println!("Best Fingerprint:   {}", best_result.fingerprint());
        println!("Target Fingerprint: {}", target);
    }
}

pub fn start_new_fingerprint(target_fingerprint: &str) {
    generate_keys(target_fingerprint, BestFingerprints::new());
}

#[derive(Serialize, Deserialize)]
struct CheckPoint {
    target: String,
    best_private_keys: Vec<String>,
}

pub fn continue_from_checkpoint() -> io::Result<()> {
    let checkpoint_filepath =
        path::Path::new(constants::KEYS_DIRECTORY).join(constants::CHECKPOINT_FILENAME);
    let checkpoint_file = fs::File::open(checkpoint_filepath)?;
    let checkpoint: CheckPoint = serde_json::from_reader(checkpoint_file)
        .unwrap_or_else(|e| panic!("Failed to parse JSON checkpoint: {}", e));

    println!("Continuing fingerprint {}", checkpoint.target);

    generate_keys(
        &checkpoint.target,
        BestFingerprints::from_checkpoint(&checkpoint),
    );

    return io::Result::Ok(());
}

fn generate_keys(target_fingerprint: &str, best_keys: BestFingerprints) {
    let best_keys = Arc::new(Mutex::new(best_keys));

    let attention = quality::gen_attention_vec(target_fingerprint);
    let similarity = quality::get_similarity_map(constants::SIMILARITY_FILEPATH);

    let total_keys_generated = Arc::new(AtomicU64::new(0));

    let endless_iter = iter::repeat((Arc::clone(&best_keys), Arc::clone(&total_keys_generated)));

    let endless_iter =
        endless_iter
            .par_bridge()
            .into_par_iter()
            .map(|(thread_best_keys, thread_total_keys)| {
                worker_function(
                    &target_fingerprint,
                    thread_best_keys,
                    &similarity,
                    &attention,
                    thread_total_keys,
                );

                return false;
            });

    let start_time = time::Instant::now();

    let target_fingerprint_cloned = target_fingerprint.to_string();
    let total_keys_cloned = Arc::clone(&total_keys_generated);
    let record_thread = thread::spawn(move || {
        display_status_thread(
            best_keys,
            &target_fingerprint_cloned,
            start_time,
            total_keys_cloned,
        );
    });

    // Will never end
    endless_iter.find_any(|x| *x);
    record_thread.join().unwrap();
    return;
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
        Arc::new(AtomicU64::new(0)),
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
