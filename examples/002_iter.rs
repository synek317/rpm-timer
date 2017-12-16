extern crate chrono;
extern crate rand;
extern crate rpm_timer;

use chrono::Local;
use std::thread;
use std::time::{Instant, Duration};
use std::fs::{File, remove_file};
use std::io::{self, BufReader, BufRead, Write};
use rand::{Rng, thread_rng};
use rpm_timer::RpmTimer;

const RPM: f64 = 1200.0;
const COUNT: usize = 1000;
const MAX_SEND_TIME_MS: u64 = 10000;
const REQUESTS_FILEPATH: &'static str = "examples/requests.txt";

// Slightly modified 001_slice example that reads requests from file

fn main() {
    generate_requests();

    let reader = BufReader::new(File::open(REQUESTS_FILEPATH).expect("file"));
    let requests = reader.lines();
    let expected_time = COUNT as f64 / RPM * 60.0;
    let start_time = Instant::now();

    log(format!("START - sending {} requests with the speed {} RPM. It will take about {} - {}s.", COUNT,  RPM, expected_time, expected_time + MAX_SEND_TIME_MS as f64 / 1000.0));

    RpmTimer::default()
        .rpm_limit(RPM)
        .max_threads(None)
        .run_iter(requests, send_http_requests);

    log(format!("END - sending {} requests with the speed {} RPM. It took {}s (expected: {} - {}).", COUNT, RPM, elapsed_seconds(start_time), expected_time, expected_time + MAX_SEND_TIME_MS as f64 / 1000.0));

    remove_file(REQUESTS_FILEPATH).expect("remove");
}

fn send_http_requests(requests: Vec<Result<String, io::Error>>) {
    let sleep_time_ms = thread_rng().gen_range(40, MAX_SEND_TIME_MS);

    log(format!("Start - sending '{:?}'. It will take {} s.", requests, sleep_time_ms as f64 / 1000.0));

    // Imagine processing real requests here
    thread::sleep(Duration::from_millis(sleep_time_ms));

    log(format!("End - sending '{:?}'", requests));
}

fn generate_requests() {
    let mut file = File::create(REQUESTS_FILEPATH).expect("file");

    for i in 0..COUNT {
        writeln!(file, "http://fake.api?id={}", i).expect("write");
    }
}

fn log(msg: String) {
    println!("[{}] {}", Local::now().to_rfc2822(), msg);
}

fn elapsed_seconds(time: Instant) -> f64 {
    let elapsed = time.elapsed();

    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1000_000_000f64
}
