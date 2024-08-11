use rand::{distributions::Alphanumeric, Rng};

pub fn generate_random_string(r : usize) -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(r)
        .map(char::from)
       .collect()
}