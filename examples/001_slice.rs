extern crate chrono;
extern crate rand;
extern crate rpm_timer;

use chrono::Local;
use std::thread;
use std::time::{Instant, Duration};
use rand::{Rng, thread_rng};
use rpm_timer::RpmTimer;

const RPM: f64 = 120.0;
const COUNT: usize = 1000;
const MAX_SEND_TIME_MS: u64 = 10000;

// Imitate sending requests to some HTTP API that only allows maximum of RPM requests per minute
// Each request takes between 40 and MAX_SEND_TIME_MS ms to process

fn main() {
    let requests = (0..100).map(|i| format!("http://fake.api?id={}", i)).collect::<Vec<_>>();
    let expected_time = COUNT as f64 / RPM * 60.0;
    let start_time = Instant::now();

    log(format!("START - sending {} requests with the speed {} RPM. It will take about {} - {}s.", COUNT,  RPM, expected_time, expected_time + MAX_SEND_TIME_MS as f64 / 1000.0));

    RpmTimer::default()
        .rpm_limit(RPM)
        .max_threads(None)
        .run_slice(&requests, &send_http_requests);

    log(format!("END - sending {} requests with the speed {} RPM. It took {}s (expected: {} - {}).", COUNT, RPM, elapsed_seconds(start_time), expected_time, expected_time + MAX_SEND_TIME_MS as f64 / 1000.0));
}

fn send_http_requests(requests: &[String]) {
    let sleep_time_ms = thread_rng().gen_range(40, MAX_SEND_TIME_MS);

    log(format!("Start - sending '{:?}'. It will take {} s.", requests, sleep_time_ms as f64 / 1000.0));

    // Imagine processing real requests here
    thread::sleep(Duration::from_millis(sleep_time_ms));

    log(format!("End - sending '{:?}'", requests));
}

fn log(msg: String) {
    println!("[{}] {}", Local::now().to_rfc2822(), msg);
}

fn elapsed_seconds(time: Instant) -> f64 {
    let elapsed = time.elapsed();

    elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1000_000_000f64
}
