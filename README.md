# Overview
RpmTimer (_RequestPerMinute Timer_) is a tool for limiting your processing speed to the requested number of items (e.g. requests) per minut.

[![Crates.io](https://img.shields.io/crates/v/rpm-timer.svg)](https://crates.io/crates/rpm-timer)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kbknapp/clap-rs/blob/master/LICENSE-MIT)
[![Build Status](https://travis-ci.org/synek317/rpm-timer.svg?branch=master)](https://travis-ci.org/synek317/rpm-timer)

[Documentation](https://docs.rs/rpm-timer/)

[Repository](https://github.com/synek317/rpm-timer)

# Getting Started

First of all you have to add this dependency to your `Cargo.toml`:

```toml
[dev-dependencies]
rpm-timer = "0.0.1"
```

Next include `rpm_timer` crate in your `lib.rs` or `main.rs` file:

```rust
extern crate test_case_derive;
```

Finally, use `RpmTimer` in the mods where you are going to limit your processing speed:

```rust
use rpm_timer::RpmTimer;
```

# Examples

In order to avoid unnecessary memory alocations, `run` function has two version.

1. `run_slices` accepts slice and pass sub-slices to the processing function:

```rust
extern crate rpm_timer;

use rpm_timer::RpmTimer;

fn send_slice(requests: Vec<String>) {
    RpmTimer::default()
        .rpm_limit(100.0)
        .run_slice(&requests, send_http_requests);
}

fn send_http_requests(requests: &[&String]) {
    // process all requests here
}
```

2. `run_iter` accepts any iterator, collects items and pass every portion in `Vec` to the processing function:processing

```rust
extern crate rpm_timer;

use rpm_timer::RpmTimer;

fn send_slice(reader: BufReader) {
    let lines = reader.lines();

    RpmTimer::default()
        .rpm_limit(100.0)
        .run_iter(lines, send_http_requests);
}

fn send_http_requests(requests: Vec<Result<String, io::Error>>) {
    // process all requests here
}
```

Please check `examples` directory for more detailed, working examples.

# Description

`RpmTimer` works in tick intervals. You can adjust tick length with `tick` method. Every tick it checks if there are any free worker threads (the number of threads can be adjusted with `max_threads`) and how many items should  be processed in order to achieve requested speed. If any items should be processed, `RpmTimer` collects them to the either slice (in non-allocating version) or `Vec` (in allocating version) and fires processing function in the parallel.

Visualization:

Assume 2 worker threads and 500 ms tick time. Also, imagine a lag (e.g. cpu busy with other processes) between 2nd and 3rd second:

`60 RPM` = `1 RPS` = `1` request every `1` second
```
Time                     0   0.5   1   1.5   2   2.5   3   3.5
Main Thread:             |....X....X....X....X.........X....X..
Number of items 'ready'  1   0.5   1   0.5   1         2   0.5
Worker #1                1**********************.......2******.
Worker #2                ..........1**************.............
                                             ^         ^
                                             |         |- 2 items sent to the thread
                                             |- an item is ready but no worker threads available

```

`30 RPM` = `0.5 RPS` = `1` request every `2` seconds
```
Time                     0   0.5   1   1.5   2   2.5   3   3.5
Main Thread:             |....X....X....X..............X....X..
Number of items 'ready'  1  0.25  0.5  0.75  1  0.25  0.5  0.75
Worker #1                1***********..........................
Worker #2                ....................1************.....
```

Legend:
- `.` - sleeping
- `X` - main thread's _tick_
- `*` - busy with # requests

# Contribution

All contributions and comments are more than welcome! Don't be afraid to open an issue or PR whenever you find a bug or have an idea to improve this crate.

# License

MIT License

Copyright (c) 2017 Marcin Sas-Szymański

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
