use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use md5;
use hex;
use rand::{Rng, thread_rng};

pub fn rand_string(length: usize) -> String {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // rand str
    let random_string: String = thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .map(|c| c as char)
        .collect();

    let nanos = since_the_epoch.subsec_nanos();
    let md5_result = format!("{:x}::{}", md5::compute(format!("{}", nanos)), random_string).to_lowercase();
    let split_str = &md5_result[..length];
    String::from(split_str)
}

pub fn rand_number(min: usize, max: usize) -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}