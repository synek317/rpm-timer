extern crate rpm_timer;

use rpm_timer::RpmTimer;

fn main() {
    let items = &["Hello", "World!", "How", "are", "you?"];

    RpmTimer::default()
        .rps_limit(1.0)
        .max_threads(1)
        .run_slice(items, print);
}

fn print(items: &[&str]) {
    for item in items {
        println!("{}", item);
    }
}