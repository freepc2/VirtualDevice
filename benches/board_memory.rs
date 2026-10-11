use std::hint::black_box;
use std::time::{Duration, Instant};

use virtualdevice::{BoardMemory, BoardType, Comizoa, Direction, NetMemory, TwinCAT2, TwinCAT3};

const DATA_POINTS: usize = 50_000;
const BOARD_COUNT: usize = 5;
const WARMUP_ROUNDS: usize = 10;
const REPETITIONS: usize = 100;
const ADDRESS_COUNT: usize = 50;
const ADDRESS_SPACE: usize = ADDRESS_COUNT * ADDRESS_COUNT + 8;

#[derive(Clone, Copy)]
struct TimingSummary {
    fastest: Duration,
    median: Duration,
    average: Duration,
    slowest: Duration,
}

struct BoardResults {
    name: &'static str,
    write: TimingSummary,
    read: TimingSummary,
    mapped_write: TimingSummary,
    mapped_read: TimingSummary,
}

fn report_comparison(board: &str, operation: &str, digital: TimingSummary, mapped: TimingSummary) {
    let digital_ns = digital.average.as_secs_f64() * 1e9 / DATA_POINTS as f64;
    let mapped_ns = mapped.average.as_secs_f64() * 1e9 / DATA_POINTS as f64;
    let (winner, faster, slower) = if digital_ns < mapped_ns {
        (format!("{operation}_digital"), digital_ns, mapped_ns)
    } else if mapped_ns < digital_ns {
        (format!("{operation}_io"), mapped_ns, digital_ns)
    } else {
        ("Tie".to_owned(), digital_ns, mapped_ns)
    };
    let ratio = slower / faster;
    println!(
        "| {board:<10} | {operation:<5} | {digital_ns:>12.3} | {mapped_ns:>12.3} | {winner:<13} | {ratio:>8.2}x | {:>15.2}% | {:>11.3} |",
        (ratio - 1.0) * 100.0,
        slower - faster,
    );
}

#[derive(Clone, Copy)]
struct IoRequest {
    net_id: u32,
    io_number: u32,
    phy_address: u32,
    offset: u32,
    bit_position: u32,
    value: bool,
}

fn make_requests() -> Vec<IoRequest> {
    (0..DATA_POINTS)
        .map(|i| {
            let net_id = (i % BOARD_COUNT) as u32;
            let phy_address = ((i / BOARD_COUNT) % ADDRESS_COUNT) as u32;
            let offset = ((i / (BOARD_COUNT * ADDRESS_COUNT)) % ADDRESS_COUNT) as u32;
            IoRequest {
                net_id,
                io_number: (i / BOARD_COUNT) as u32,
                phy_address,
                offset,
                bit_position: (i % 8) as u32,
                value: i % 2 == 0,
            }
        })
        .collect()
}

fn summarize(mut samples: Vec<Duration>) -> TimingSummary {
    let average_seconds =
        samples.iter().map(Duration::as_secs_f64).sum::<f64>() / samples.len() as f64;
    samples.sort_unstable();
    TimingSummary {
        fastest: samples[0],
        median: samples[samples.len() / 2],
        average: Duration::from_secs_f64(average_seconds),
        slowest: samples[samples.len() - 1],
    }
}

fn measure(mut operation: impl FnMut()) -> TimingSummary {
    for _ in 0..WARMUP_ROUNDS {
        operation();
    }

    let samples = (0..REPETITIONS)
        .map(|_| {
            let start = Instant::now();
            operation();
            start.elapsed()
        })
        .collect();
    summarize(samples)
}

fn format_count(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(digit);
    }
    formatted
}

fn report_io(board: &str, operation: &str, timing: TimingSummary) {
    let ops_per_second = |elapsed: Duration| (DATA_POINTS as f64 / elapsed.as_secs_f64()) as u64;
    let ns_per_io = |elapsed: Duration| elapsed.as_nanos() as f64 / DATA_POINTS as f64;
    println!(
        "| {board:<10} | {operation:<13} | {:>13} | {:>13} | {:>13} | {:>13} | {:>10.2} | {:>11.2} | {:>12.2} | {:>10.2} |",
        format_count(ops_per_second(timing.fastest)),
        format_count(ops_per_second(timing.average)),
        format_count(ops_per_second(timing.median)),
        format_count(ops_per_second(timing.slowest)),
        ns_per_io(timing.fastest),
        ns_per_io(timing.average),
        ns_per_io(timing.median),
        ns_per_io(timing.slowest),
    );
}

fn make_net<B: BoardType>() -> NetMemory<B> {
    let mut net = NetMemory::new();
    for _ in 0..BOARD_COUNT {
        net.boards
            .push(BoardMemory::<B>::new(ADDRESS_SPACE, ADDRESS_SPACE));
    }
    net
}

fn benchmark_net<B: BoardType>(name: &'static str, requests: &[IoRequest]) -> BoardResults {
    let mut net = make_net::<B>();

    // Both APIs operate on the same bytes. Registration is outside all timers.
    for request in requests {
        net.register_io(
            request.net_id,
            request.io_number,
            Direction::Output,
            request.phy_address + request.offset,
            request.bit_position,
        )
        .expect("valid benchmark registration");
    }

    let write = measure(|| {
        for request in requests {
            let address = request.phy_address + request.offset;
            net.write_digital(
                request.net_id,
                Direction::Output,
                address,
                request.bit_position,
                request.value,
            )
            .expect("valid benchmark write");
        }
        black_box(&net);
    });

    let read = measure(|| {
        for request in requests {
            let address = request.phy_address + request.offset;
            black_box(
                net.read_digital(
                    request.net_id,
                    Direction::Output,
                    address,
                    request.bit_position,
                )
                .expect("valid benchmark read"),
            );
        }
    });

    let mapped_write = measure(|| {
        for request in requests {
            net.write_io(
                request.net_id,
                request.io_number,
                Direction::Output,
                request.value,
            )
            .expect("valid benchmark IO write");
        }
        black_box(&net);
    });

    let mapped_read = measure(|| {
        for request in requests {
            black_box(
                net.read_io(request.net_id, request.io_number, Direction::Output)
                    .expect("valid benchmark IO read"),
            );
        }
    });

    // Validate the mappings without including these checks in the measurement.
    for request in requests {
        assert_eq!(
            net.read_io(request.net_id, request.io_number, Direction::Output)
                .unwrap(),
            net.read_digital(
                request.net_id,
                Direction::Output,
                request.phy_address + request.offset,
                request.bit_position,
            )
            .unwrap(),
        );
    }

    report_io(name, "write_digital", write);
    report_io(name, "write_io", mapped_write);
    report_io(name, "read_digital", read);
    report_io(name, "read_io", mapped_read);
    BoardResults {
        name,
        write,
        read,
        mapped_write,
        mapped_read,
    }
}

fn main() {
    let requests = make_requests();
    println!("NetMemory in-memory benchmark");
    println!(
        "Boards: {BOARD_COUNT} | Requests/sample: {} | Warm-up: {WARMUP_ROUNDS} | Measured samples: {REPETITIONS} | Requests/operation: {}",
        format_count(DATA_POINTS as u64),
        format_count((DATA_POINTS * REPETITIONS) as u64),
    );
    println!("Address inputs: 50 phyAddress values, 50 offsets, bitPosition 0-7");
    println!();
    println!("NetId: 0-4 | IO numbers/board: 0-9,999 | Direction: Output");
    println!(
        "Address calculation: phyAddress + offset; byte-addressed boards also add bitPosition"
    );
    println!("IO registration and mapping validation are excluded from timing");
    println!("[NetMemory API comparison: in-memory throughput]");
    println!(
        "| Board      | Operation     | Peak IO/s   | Mean IO/s   | Median IO/s | Lowest IO/s | Min ns/IO | Mean ns/IO | Median ns/IO | Max ns/IO |"
    );
    println!(
        "|------------|---------------|-------------|-------------|-------------|-------------|-----------|------------|--------------|-----------|"
    );

    let results = [
        benchmark_net::<TwinCAT2>("TwinCAT2", &requests),
        benchmark_net::<TwinCAT3>("TwinCAT3", &requests),
        benchmark_net::<Comizoa>("Comizoa", &requests),
    ];

    println!();
    println!("[Winners based on mean time per IO; smaller ns/IO is faster]");
    println!("Speedup = slower time / faster time; throughput gain is relative to the slower API");
    println!("Measured differences may include timing noise; registration cost is excluded");
    println!(
        "| Board      | Type  | Digital ns/IO | Mapped ns/IO | Faster API    | Speedup   | Throughput gain  | Saved ns/IO |"
    );
    println!(
        "|------------|-------|---------------|--------------|---------------|-----------|------------------|-------------|"
    );
    for result in results {
        report_comparison(result.name, "write", result.write, result.mapped_write);
        report_comparison(result.name, "read", result.read, result.mapped_read);
    }
}
