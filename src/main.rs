mod constants;
mod keygen;
mod quality;
mod tests;

use clap::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("Conflicting arguments: enter a fingerprint or continuous option")]
    ConflictingArguments,

    #[error("Missing or corrupted checkpoint file")]
    BadCheckpoint,

    #[error("Invalid {} fingerprint", constants::FINGERPRINT_HASH_ALGORITHM)]
    InvalidFingerprint,
}

fn is_base64(characters: &str) -> bool {
    let alphabet: Vec<_> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .chars()
        .collect();

    for char in characters.chars() {
        if !alphabet.contains(&char) {
            return false;
        }
    }
    return true;
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Base 64 string of the SHA256 fingerprint
    fingerprint: Option<String>,

    /// Continue from checkpoint
    #[arg(short, long)]
    checkpoint: bool,
}

fn parse_cli() -> Result<(), Error> {
    let cli = Cli::parse();

    if cli.fingerprint.is_some() && cli.checkpoint {
        eprintln!("{}", Error::ConflictingArguments);
        return Result::Err(Error::ConflictingArguments);
    }

    // Starting new run for fingerprint
    if cli.fingerprint.is_some() {
        let mut fingerprint = cli.fingerprint.expect("Checked");

        // Check value

        if fingerprint.len() != constants::BASE_64_FINGERPRINT_LENGTH {
            fingerprint = quality::strip_fingerprint(&fingerprint).to_string();
        }

        if fingerprint.len() != constants::BASE_64_FINGERPRINT_LENGTH || !is_base64(&fingerprint) {
            eprintln!(
                "Fingerprint length must be {} characters of base64 without padding",
                constants::BASE_64_FINGERPRINT_LENGTH
            );
            return Err(Error::InvalidFingerprint);
        }

        keygen::start_new_fingerprint(&fingerprint);
    }

    if cli.checkpoint {
        let checkpoint_result = keygen::continue_from_checkpoint();

        if let Err(checkpoint_error) = checkpoint_result {
            eprintln!("{}", checkpoint_error);
            eprintln!("{}", Error::BadCheckpoint);
            return Err(Error::BadCheckpoint);
        }
    }

    return Result::Ok(());
}

fn main() -> Result<(), Error> {
    // keygen::start_new_fingerprint("+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU")
    parse_cli()?;

    return Ok(());
}

// TODO Clap interface for checkpoint files
// TODO Adjust attention function to focus more on the very beginning of the
// Fingerprint
// TODO pretty little flame graph
