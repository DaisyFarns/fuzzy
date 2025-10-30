#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;

use serde_json;

use crate::constants;

pub type SimilarityMap = HashMap<(u8, u8), f32>;

/// Strip any 'SHA256:' part from the fingerprint string
pub fn strip_fingerprint(fingerprint: &str) -> &str {
    match fingerprint.find(":") {
        None => return fingerprint,
        Some(colon_index) => {
            // May panic if fingerprint doesn't return ASCII
            return &fingerprint[colon_index + 1..];
        }
    }
}

pub fn gen_attention_vec(target_fingerprint_str: &str) -> Vec<f32> {
    let mut attention = Vec::new();

    // Generate an exponential curve

    let lambda = 16.2;
    let exponential_curve = |x: f32| std::f32::consts::E.powf(-1.0 * lambda * x);

    let attention_function = |x: f32| -> f32 {
        exponential_curve(x)
            .max(0.5 * exponential_curve(-x + 1.0))
            .max(0.01)
    };

    let target_fingerprint_str = strip_fingerprint(target_fingerprint_str);

    // Liner interpolation of the attention_function

    let fingerprint_length = target_fingerprint_str.len();
    let step_size: f32 = 1.0 / fingerprint_length as f32;
    let first_value = step_size / 2.0;

    for i in 0..fingerprint_length {
        attention.push(attention_function(first_value + step_size * i as f32))
    }

    let attention_sum: f32 = attention.iter().sum();

    dbg!(&attention);

    return attention.iter().map(|x| x / attention_sum).collect();
}

/// Get the similarity of all pairs of base 64 characters
/// Pairs are ordered on base 64 index, with a similarity
/// value between 0 and 10. Two of the same characters will
/// always map to 10. What is
///
/// To find the similarity between 'A' and 'a', the key would be
/// (0u8, 26u8)
pub fn get_similarity_map(filepath: &str) -> SimilarityMap {
    let mut map = HashMap::with_capacity(32 * 63);

    let json_file = fs::File::open(filepath).expect(&format!("Can't open {}", filepath));

    let similarity: serde_json::Value = serde_json::from_reader(json_file).unwrap();
    for i in 0..64 {
        for j in i..64 {
            if let serde_json::Value::Number(pair_similarity) = &similarity[i][j] {
                let pair_similarity = pair_similarity.as_u64().unwrap();

                if 10 < pair_similarity {
                    panic!(
                        "JSON Malformed, value {} at index {}, {} is not between 0 and 10",
                        pair_similarity, i, j
                    )
                }
                map.insert((i as u8, j as u8), (pair_similarity as f32) / 10.0);
            }
        }
    }

    return map;
}

pub fn fingerprint_str_to_b64_index(
    stripped_fingerprint_str: &str,
) -> [u8; constants::BASE_64_FINGERPRINT_LENGTH] {
    let mut fingerprint = [0_u8; 43];

    for (index, character_byte) in stripped_fingerprint_str.bytes().enumerate() {
        match character_byte {
            // A-Z
            65..91 => fingerprint[index] = character_byte - 65,

            // a-z
            97..123 => fingerprint[index] = character_byte - 71,

            // 0-9
            48..58 => fingerprint[index] = character_byte + 4,

            // +
            43 => fingerprint[index] = 62,

            // /
            47 => fingerprint[index] = 63,

            _ => panic!("Invalid base 64 char in fingerprint: {}", character_byte),
        }
    }

    return fingerprint;
}

pub fn fingerprint_quality(
    target_fingerprint_b64_index: &[u8; constants::BASE_64_FINGERPRINT_LENGTH],
    candidate_fingerprint: &str,
    similarity: &SimilarityMap,
    attention_map: &Vec<f32>,
) -> f32 {
    let candidate_fingerprint_b64_indexes =
        fingerprint_str_to_b64_index(strip_fingerprint(candidate_fingerprint));

    let mut candidate_similarty: [f32; 43] = [0_f32; constants::BASE_64_FINGERPRINT_LENGTH];

    for (index, pair) in target_fingerprint_b64_index
        .iter()
        .zip(candidate_fingerprint_b64_indexes)
        .enumerate()
    {
        let sorted_pair: (u8, u8);

        if pair.1 < *pair.0 {
            sorted_pair = (pair.1, *pair.0)
        } else {
            sorted_pair = (*pair.0, pair.1);
        }

        candidate_similarty[index] = *similarity.get(&sorted_pair).unwrap();
    }

    let mut quality = 0.0;

    for (index, char_simalarity) in candidate_similarty.iter().enumerate() {
        quality += char_simalarity * attention_map[index]
    }

    return quality;
}
