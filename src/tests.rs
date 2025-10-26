#![cfg(test)]

use crate::constants;
use crate::quality;
use crate::quality::gen_attention_vec;
use crate::quality::get_similarity_map;

#[test]
fn validate_string_to_base_64_index() {
    let fingerprint = "+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU";
    let expected: [u8; constants::BASE_64_FINGERPRINT_LENGTH] = [
        62, 3, 34, 24, 55, 48, 47, 47, 21, 58, 19, 46, 9, 9, 33, 27, 41, 25, 34, 44, 5, 63, 51, 11,
        3, 0, 52, 51, 15, 12, 18, 47, 7, 29, 36, 43, 56, 20, 47, 2, 14, 42, 20,
    ];

    assert_eq!(quality::fingerprint_str_to_b64_index(fingerprint), expected);
}

#[test]
fn max_similarity() {
    let map = get_similarity_map("base64similarities.json");

    let mut max = 0.0_f32;
    for item in map.values() {
        if *item > max {
            max = *item;
        }
    }

    assert_eq!(max, 1.0)
}

fn gen_quaility_simalarity_and_attention() -> (quality::SimilarityMap, Vec<f32>) {
    let fingerprint = "+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU";

    let attention = gen_attention_vec(fingerprint);

    let similarity_map = quality::get_similarity_map("base64similarities.json");

    return (similarity_map, attention);
}

#[test]
fn identical_fingerprint_quaility() {
    let fingerprint = "+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU";

    let fingerprint_b64_index = quality::fingerprint_str_to_b64_index(fingerprint);

    let (similarity_map, attention) = gen_quaility_simalarity_and_attention();

    assert!(
        quality::fingerprint_quality(
            &fingerprint_b64_index,
            fingerprint,
            &similarity_map,
            &attention
        ) > 0.99
    )
}

#[test]
fn example_quaility() {
    let target_fingerprint = "ZcamIQM3R6IY5zZjMeEDWk2A360sQTe1uFBh2Ef9AyZ";
    let similar_fingerprint = "ZcamIQM3R6siTmPCmB8IPtTNNkDX8yh95FBh2Ef9AyZ";
    let random_fingerprint = "0qVX7iU8jCN/pPwTqL6iO5b83bmjXMyKdJe2UXXALmJ";

    let (similarity_map, attention) = gen_quaility_simalarity_and_attention();

    let target_b64_index = quality::fingerprint_str_to_b64_index(target_fingerprint);

    assert!(
        quality::fingerprint_quality(
            &target_b64_index,
            similar_fingerprint,
            &similarity_map,
            &attention
        ) > 0.5
    );

    assert!(
        quality::fingerprint_quality(
            &target_b64_index,
            random_fingerprint,
            &similarity_map,
            &attention
        ) < 0.05
    )
}
