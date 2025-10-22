# Fuzzy fingerprint generator

CMD tool for generating fuzzy fingerprints

## Timings

### Results

Time it took to preform generate 10 000 keys and their fingerprints:

```
   EC25519:    863ms
ECDSA-P256:  3 476ms
ECDSA-P384: 15 566ms
ECDSA-P521: 19 961ms
```

All these key types took much longer to generate:

```
       DSA:  1 100ms for 1
RSA-SHA256: 22 000ms for 1
RSA-SHA512: 16 700ms for 1
```

### Code

```
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
```

## TODO

Use Mutex for multithreading, but save a copy of the value of the fingerprint
value in each thread and only check mutex when new fingerprint is above this
value. The value is only going to increase :3
