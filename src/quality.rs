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

    let attention_function = |x: f32| -> f32 { return (-1.5 * x + 1.0).max(1.5 * x - 0.5) };

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
