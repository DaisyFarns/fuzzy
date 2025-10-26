use std::collections::HashMap;
use std::fs;

use serde_json;

// Strip any 'SHA256:' part from the fingerprint string
pub fn strip_fingerprint(fingerprint: &str) -> &str {
    match fingerprint.find(":") {
        None => return fingerprint,
        Some(colon_index) => {
            // May panic if fingerprint doesn't return ASCII
            return &fingerprint[colon_index + 1..];
        }
    }
}

pub fn gen_attention_map(target_fingerprint_str: &str) -> Vec<f32> {
    let mut attention = Vec::new();

    let attention_function = |x: f32| -> f32 { return (4.0 * x * x).max(0.1) };

    let target_fingerprint_str = strip_fingerprint(target_fingerprint_str);

    // Liner interprilation of the attention_function

    let fingerprint_length = target_fingerprint_str.len();
    let step_size: f32 = 1.0 / fingerprint_length as f32;
    let first_value = step_size / 2.0;

    for i in 0..fingerprint_length {
        attention.push(attention_function(first_value + step_size * i as f32))
    }

    return attention;
}

/// Get the similarity of all pairs of base 64 characters
/// Pairs are ordered on base 64 index, with a similarity
/// value between 0 and 10. Two of the same characters will
/// always map to 10
///
/// To find the similarity between 'A' and 'a', the key would be
/// (0u8, 26u8)
pub fn get_similarity_map(filepath: &str) -> HashMap<(u8, u8), u8> {
    let mut map = HashMap::with_capacity(32 * 63);

    let json_file = fs::File::open(filepath).expect(&format!("Can't open {}", filepath));

    let similarity: serde_json::Value = serde_json::from_reader(json_file).unwrap();
    dbg!(&similarity);

    for i in 0..64 {
        for j in i..64 {
            if let serde_json::Value::Number(pair_similarity) = &similarity[i][j] {

                let pair_similarity = pair_similarity.as_u64().unwrap() as u8;

                if 10 < pair_similarity {
                    panic!(
                        "JSON Malformed, value {} at index {}, {} is not between 0 and 10",
                        pair_similarity, i, j
                    )
                }
                map.insert((i as u8, j as u8), pair_similarity);
            }
        }
    }

    return map;
}
