use ssh_key::{Algorithm, HashAlg, PrivateKey, rand_core::OsRng};

use std::time;

fn main() {
    println!("Hi!");

    let mut fingerprints = Vec::new();

    let t = time::Instant::now();

    for _ in 0..10_000 {
        let skey = PrivateKey::random(
            &mut OsRng,
            Algorithm::Ecdsa {
                curve: ssh_key::EcdsaCurve::NistP521,
            },
        )
        .unwrap();
        let pub_key = skey.public_key();
        fingerprints.push(pub_key.fingerprint(HashAlg::Sha256));
    }

    println!("Done in {}ms", t.elapsed().as_millis());
}
