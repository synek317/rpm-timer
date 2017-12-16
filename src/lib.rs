//! # Overview
//! RpmTimer (_RequestPerMinute Timer_) is a tool for limiting your processing speed to the requested number of items (e.g. requests) per minut.
//!
//! It is designed to work with any rate-limited API.
//!
//! [![Crates.io](https://img.shields.io/crates/v/rpm-timer.svg)](https://crates.io/crates/rpm-timer)
//! [![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kbknapp/clap-rs/blob/master/LICENSE-MIT)
//! [![Build Status](https://travis-ci.org/synek317/rpm-timer.svg?branch=master)](https://travis-ci.org/synek317/rpm-timer)
//!
//! [Documentation](https://docs.rs/rpm-timer/)
//!
//! [Repository](https://github.com/synek317/rpm-timer)
//!
//! # Getting Started
//!
//! First of all you have to add this dependency to your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! rpm-timer = "0.0.1"
//! ```
//!
//! Next include `rpm_timer` crate in your `lib.rs` or `main.rs` file:
//!
//! ```
//! extern crate test_case_derive;
//! ```
//!
//! Finally, use `RpmTimer` in the mods where you are going to limit your processing speed:
//!
//! ```
//! use rpm_timer::RpmTimer;
//! ```
//!
//! # Examples
//!
//! In order to avoid unnecessary memory alocations, `run` function has two version.
//!
//! 1. `run_slices` accepts slice and pass sub-slices to the processing function:
//!
//! ```
//! extern crate rpm_timer;
//!
//! use rpm_timer::RpmTimer;
//!
//! fn send_slice(requests: Vec<String>) {
//!     RpmTimer::default()
//!         .rpm_limit(100.0)
//!         .run_slice(&requests, send_http_requests);
//! }
//!
//! fn send_http_requests(requests: &[&String]) {
//!     // process all requests here
//! }
//! ```
//!
//! 2. `run_iter` accepts any iterator, collects items and pass every portion in `Vec` to the processing function:processing
//!
//! ```
//! extern crate rpm_timer;
//!
//! use rpm_timer::RpmTimer;
//!
//! fn send_slice(reader: BufReader) {
//!     let lines = reader.lines();
//!
//!     RpmTimer::default()
//!         .rpm_limit(100.0)
//!         .run_iter(lines, send_http_requests);
//! }
//!
//! fn send_http_requests(requests: Vec<Result<String, io::Error>>) {
//!     // process all requests here
//! }
//! ```
//!
//! Please check `examples` directory for more detailed, working examples.
//!
//! # Description
//!
//! `RpmTimer` works in tick intervals. You can adjust tick length with `tick` method. Every tick it checks if there are any free worker threads (the number of threads can be adjusted with `max_threads`) and how many items should  be processed in order to achieve requested speed. If any items should be processed, `RpmTimer` collects them to the either slice (in non-allocating version) or `Vec` (in allocating version) and fires processing function in the parallel.
//!
//! Visualization:
//!
//! Assume 2 worker threads and 500 ms tick time. Also, imagine a lag (e.g. cpu busy with other processes) between 2nd and 3rd second:60
//!
//! __60 RPM__ = __1 RPS__ = __1__ request every __1__ second
//!
//! ```
//! Time                     0   0.5   1   1.5   2   2.5   3   3.5
//! Main Thread:             |....X....X....X....X.........X....X..
//! Number of items ready    1   0.5   1   0.5   1         2   0.5
//! Worker no. 1             1**********************.......2******.
//! Worker no. 2             ..........1**************.............
//!                                              ^         ^
//!                                              |         |- 2 items sent to the thread
//!                                              |- an item is ready but no worker threads available
//!
//! ```
//!
//! __30 RPM__ = __0.5 RPS__ = __1__ request every __2__ seconds
//!
//! ```
//! Time                     0   0.5   1   1.5   2   2.5   3   3.5
//! Main Thread:             |....X....X....X..............X....X..
//! Number of items ready    1  0.25  0.5  0.75  1  0.25  0.5  0.75
//! Worker no. 1             1***********..........................
//! Worker no. 2             ....................1************.....
//! ```
//!
//! Legend:
//!
//!   - `.` - sleeping
//!   - `X` - main thread's _tick_
//!   - `*` - busy with # requests
//!
//! # Contribution
//!
//! All contributions and comments are more than welcome! Don't be afraid to open an issue or PR whenever you find a bug or have an idea to improve this crate.
//!
//! # License
//!
//! MIT License
//!
//! Copyright (c) 2017 Marcin Sas-Szymański
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.


extern crate scoped_pool;
extern crate num_cpus;

mod helpers;

use std::time::{Duration, Instant};
use std::cmp::min;
use std::thread::sleep;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize};
use scoped_pool::Pool;
use self::helpers::*;

/// Use this struct to limit the speed of any items processing.
///
/// Adjust processing speed using struct's methods.
///
/// Example usage:
///
/// ```
/// extern crate rpm_timer;
///
/// use rpm_timer::RpmTimer;
///
/// fn main() {
///     let items = &["Hello", "World!", "How", "are", "you?"];
///
///     RpmTimer::default()
///         .rps_limit(1.0)
///         .max_threads(1)
///         .run_slice(items, print);
/// }
///
/// fn print(items: &[&str]) {
///     for item in items {
///         println!("{}", item);
///     }
/// }
/// ```
pub struct RpmTimer {
    tick: Duration,
    rps_limit: f64,
    max_threads: Option<usize> //None == number of cpus
}

impl RpmTimer {
    /// Main thread will try to spawn working threads every _tick_.
    ///
    /// Tip: yhe higher RPM requested, the lower tick duration should be.
    ///
    /// Default: 100 ms
    pub fn tick(mut self, value: Duration) -> Self {
        self.tick = value;
        self
    }

    /// Target requests per minute number. It overrides the value previously set by `rps_limit`, if any.
    ///
    /// Default: 60
    pub fn rpm_limit(self, value: f64) -> Self {
        self.rps_limit(value / 60f64)
    }

    /// Target requests per second number. It overrides the value previously set by `rpm_limit`, if any.
    ///
    /// Default: 1
    pub fn rps_limit(mut self, value: f64) -> Self {
        self.rps_limit = value;
        self
    }

    /// Maximum number of working threads in the pool.
    ///
    /// Pass `None` to limit the number to the number of cpu cores (uses `num_cpus` under the hood).
    ///
    /// Default: None
    pub fn max_threads<T: Into<Option<usize>>>(mut self, value: T) -> Self {
        self.max_threads = value.into();
        self
    }

    /// Non-allocating method that spawns thread and pass sub-slices to the workers.
    ///
    /// This is the preffered way unless you only have an iterator.
    ///
    /// It waits for all spawned threads to finish.
    pub fn run_slice<T, F>(self, items: &[T], action: F)
        where F: Fn(&[T]) + Sync,
              T: Send + Sync
    {
        let mut last_dispatched_item_index = 0;

        self.run(action, |items_to_dispatch| {
            let first_item_index_to_process = last_dispatched_item_index;

            last_dispatched_item_index = min(last_dispatched_item_index + items_to_dispatch, items.len());

            (&items[first_item_index_to_process..last_dispatched_item_index], last_dispatched_item_index == items.len())
        });
    }

    /// Allocating method that spawns thread and pass vectors with collected items to the workers.
    ///
    /// This is the most generic solution but you should only use it when `run_slice` is not possible..
    ///
    /// It waits for all spawned threads to finish.
    pub fn run_iter<T, I, F>(self, mut items: I, action: F)
        where F: Fn(Vec<T>) + Sync,
              I: Iterator<Item=T>,
              T: Send
    {
        self.run(action, |items_to_dispatch| {
            let items_to_process = items.by_ref().take(items_to_dispatch).collect::<Vec<_>>();
            let len = items_to_process.len();

            (items_to_process, len != items_to_dispatch)
        });
    }

    fn run<TItems, FAction, FTake>(self, action: FAction, mut take: FTake)
        where FAction: Fn(TItems) + Sync,
              FTake: FnMut(usize) -> (TItems, bool),
              TItems: Send
    {
        let pool_size          = self.max_threads.unwrap_or_else(|| num_cpus::get());
        let pool               = Pool::new(pool_size);
        let working_threads    = Arc::new(AtomicUsize::new(0));
        let mut last_tick_time = Instant::now();
        let mut items_ready    = 1f64;
        let mut finished       = false;

        pool.scoped(|scope|
            while !finished {
                let tick_start_time = Instant::now();

                if working_threads.get() < pool_size {
                    let seconds_since_last_tick = last_tick_time.elapsed_seconds();

                    last_tick_time  = tick_start_time;
                    items_ready    += self.rps_limit * seconds_since_last_tick;

                    let items_to_take = items_ready.floor() as usize;

                    if items_to_take > 0 {
                        let (taken_items, is_finished) = take(items_to_take);
                        let working_threads_clone      = working_threads.clone();
                        let a = &action;

                        finished     = is_finished;
                        items_ready -= items_to_take as f64;

                        working_threads.increase();

                        scope.execute(move || {
                            a(taken_items);
                            working_threads_clone.decrease();
                        });
                    }
                }

                sleep(self.tick - tick_start_time.elapsed());
            }
        );
    }
}

impl Default for RpmTimer {
    fn default() -> Self {
        Self {
            tick:        Duration::from_millis(100),
            rps_limit:   1f64,
            max_threads: None
        }
    }
}
