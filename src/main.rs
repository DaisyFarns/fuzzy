mod constants;
mod keygen;
mod quality;
mod tests;

fn main() {
    // keygen::start_new_fingerprint("+DiY3wvvV6TuJJhbpZisF/zLDA0zPMSvHdkr4UvCOqU")
    keygen::continue_from_checkpoint();
}

// TODO Adjust attention function to be much better
