use std::hint::black_box;
use std::time::{Duration, Instant};

use virtualdevice::{
    AccessKind, AnalogValue, AmsAddr, BoardId, BoardMemory, BoardType, Comizoa, Direction,
    IoAddress, TwinCAT2, TwinCAT3,
};

const DATA_POINTS: usize = 400_000;
const REPETITIONS: usize = 50;
const DIGITAL_BYTES_PER_POINT: usize = 2;
const ANALOG_TYPES: [(&str, usize); 8] = [
    ("u8", 1),
    ("i8", 1),
    ("u16", 2),
    ("i16", 2),
    ("u32", 4),
    ("i32", 4),
    ("u64", 8),
    ("i64", 8),
];

#[derive(Clone, Copy)]
struct TimingSummary {
    min: Duration,
    median: Duration,
    max: Duration,
}

fn summarize(mut samples: Vec<Duration>) -> TimingSummary {
    samples.sort_unstable();
    let middle = samples.len() / 2;
    let median = if samples.len() % 2 == 0 {
        (samples[middle - 1] + samples[middle]) / 2
    } else {
        samples[middle]
    };
    TimingSummary {
        min: samples[0],
        median,
        max: samples[samples.len() - 1],
    }
}

fn format_ops(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.2} M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2} K", value / 1_000.0)
    } else {
        format!("{value:.2}")
    }
}

fn format_bytes_per_second(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("{:.2} GB/s", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.2} MB/s", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2} KB/s", value / 1_000.0)
    } else {
        format!("{value:.2} B/s")
    }
}

fn report(
    function: &str,
    data_type: &str,
    summary: TimingSummary,
    calls_per_sample: usize,
    bytes_per_sample: Option<usize>,
) {
    let metrics = |duration: Duration| {
        let seconds = duration.as_secs_f64();
        let ns_per_op = duration.as_nanos() as f64 / calls_per_sample as f64;
        let ops_per_second = calls_per_sample as f64 / seconds;
        (ns_per_op, format_ops(ops_per_second))
    };
    let (min_ns, min_ops) = metrics(summary.min);
    let (median_ns, median_ops) = metrics(summary.median);
    let (max_ns, max_ops) = metrics(summary.max);
    println!(
        "  {function:<15} {data_type:<5} min {:>9.2} ns/op ({:>9} ops/s) | median {:>9.2} ns/op ({:>9} ops/s) | max {:>9.2} ns/op ({:>9} ops/s)",
        min_ns,
        min_ops,
        median_ns,
        median_ops,
        max_ns,
        max_ops,
    );
    if let Some(bytes) = bytes_per_sample {
        let byte_rate = |duration: Duration| bytes as f64 / duration.as_secs_f64();
        println!(
            "  {:<21} min {:>10} | median {:>10} | max {:>10}",
            "byte throughput:",
            format_bytes_per_second(byte_rate(summary.min)),
            format_bytes_per_second(byte_rate(summary.median)),
            format_bytes_per_second(byte_rate(summary.max)),
        );
    }
}

fn measure(mut operation: impl FnMut(usize)) -> TimingSummary {
    summarize(
        (0..REPETITIONS)
            .map(|round| {
                let start = Instant::now();
                operation(round);
                start.elapsed()
            })
            .collect(),
    )
}

fn address<B: BoardType>(direction: Direction, offset: usize, kind: AccessKind) -> IoAddress<B> {
    IoAddress::new(direction, 0, offset as u32, kind)
}

fn bench_digital<B: BoardType>(memory: &mut BoardMemory<B>) {
    let output_addresses: Vec<_> = (0..DATA_POINTS)
        .map(|index| {
            let (offset, bit_position) = if B::BOOL_IS_BYTE {
                (index, 0)
            } else {
                (
                    (index / 16) * DIGITAL_BYTES_PER_POINT,
                    (index % 16) as u8,
                )
            };
            address::<B>(
                Direction::Output,
                offset,
                AccessKind::Digital { bit_position },
            )
        })
        .collect();
    let write = measure(|round| {
        for (index, point) in output_addresses.iter().enumerate() {
            memory
                .write_digital(point, (round + index) % 2 == 0)
                .unwrap();
        }
        black_box(&memory.outputs);
    });
    let read = measure(|_| {
        for point in &output_addresses {
            black_box(memory.read_digital_output(point).unwrap());
        }
    });
    report("write_digital", "bool", write, DATA_POINTS, None);
    report("read_digital", "bool", read, DATA_POINTS, None);

    let write_output = measure(|round| {
        for (index, point) in output_addresses.iter().enumerate() {
            memory
                .write_digital_output(point, (round + index) % 2 == 0)
                .unwrap();
        }
        black_box(&memory.outputs);
    });
    report("write_digital_out", "bool", write_output, DATA_POINTS, None);

    let input_addresses: Vec<_> = output_addresses
        .iter()
        .map(|point| {
            IoAddress::new(
                Direction::Input,
                point.phyaddress,
                point.offset,
                point.access_kind,
            )
        })
        .collect();
    let write_input = measure(|round| {
        for (index, point) in input_addresses.iter().enumerate() {
            memory
                .write_digital_input(point, (round + index) % 2 == 0)
                .unwrap();
        }
        black_box(&memory.inputs);
    });
    let read_input = measure(|_| {
        for point in &input_addresses {
            black_box(memory.read_digital_input(point).unwrap());
        }
    });
    report("write_digital_in", "bool", write_input, DATA_POINTS, None);
    report("read_digital_in", "bool", read_input, DATA_POINTS, None);
}

fn bench_analog_type<B: BoardType, T: AnalogValue + Default + 'static>(
    memory: &mut BoardMemory<B>,
    name: &str,
    width: usize,
    write: impl Fn(T, &IoAddress<B>, &mut BoardMemory<B>),
    read: impl Fn(&IoAddress<B>, &BoardMemory<B>) -> T,
) {
    let values: Vec<T> = (0..DATA_POINTS).map(|_| T::default()).collect();
    let addresses: Vec<_> = (0..DATA_POINTS)
        .map(|index| {
            address::<B>(
                Direction::Output,
                index * width,
                AccessKind::Analog {
                    size_bytes: width as u8,
                },
            )
        })
        .collect();
    let write_summary = measure(|_| {
        for (value, point) in values.iter().copied().zip(&addresses) {
            write(value, point, memory);
        }
        black_box(&memory.outputs);
    });
    let read_summary = measure(|_| {
        for point in &addresses {
            black_box(read(point, memory));
        }
    });
    report("write_analog", name, write_summary, DATA_POINTS, None);
    report("read_analog", name, read_summary, DATA_POINTS, None);
}

macro_rules! bench_analog_type {
    ($memory:expr, $ty:ty, $label:literal, $width:expr) => {
        bench_analog_type::<_, $ty>(
            $memory,
            $label,
            $width,
            |value, address, memory| memory.write_analog(address, value).unwrap(),
            |address, memory| memory.read_analog::<$ty>(address).unwrap(),
        )
    };
}

fn bench_bytes<B: BoardType>(memory: &mut BoardMemory<B>) {
    let data = vec![0xA5; DATA_POINTS];
    let write = measure(|_| {
        memory
            .write_bytes(Direction::Output, 0, 0, &data)
            .unwrap();
        black_box(&memory.outputs);
    });
    let read = measure(|_| {
        let sum = memory
            .read_bytes(Direction::Output, 0, 0, DATA_POINTS)
            .unwrap()
            .iter()
            .fold(0u64, |sum, byte| sum.wrapping_add(u64::from(*byte)));
        black_box(sum);
    });
    let read_array = measure(|_| {
        for index in 0..DATA_POINTS {
            black_box(
                memory
                    .read_byte_array::<1>(Direction::Output, 0, index as u32)
                    .unwrap(),
            );
        }
    });
    report("write_bytes", "u8", write, 1, Some(DATA_POINTS));
    report("read_bytes", "u8", read, 1, Some(DATA_POINTS));
    report("read_byte_array", "[u8;1]", read_array, DATA_POINTS, None);
}

fn bench_direct_array_access() {
    let mut array = vec![0u8; DATA_POINTS];
    let values = vec![0x5A; DATA_POINTS];
    let write = measure(|_| {
        for (destination, value) in array.iter_mut().zip(&values) {
            *destination = *value;
        }
        black_box(&array);
    });
    let read = measure(|_| {
        let mut sum = 0u64;
        for value in &array {
            sum = sum.wrapping_add(u64::from(*value));
        }
        black_box(sum);
    });
    report("array_write", "u8", write, DATA_POINTS, None);
    report("array_read", "u8", read, DATA_POINTS, None);
}

fn benchmark_board<B: BoardType>(name: &str, connection: B::Connection) {
    println!("\n[{name}]");
    let digital_size = if B::BOOL_IS_BYTE {
        DATA_POINTS
    } else {
        DATA_POINTS.div_ceil(16) * DIGITAL_BYTES_PER_POINT
    };
    let analog_size = DATA_POINTS * 8;
    let mut memory = BoardMemory::<B>::new(connection, analog_size, digital_size.max(analog_size));

    bench_digital(&mut memory);
    for (type_name, width) in ANALOG_TYPES {
        match type_name {
            "u8" => { bench_analog_type!(&mut memory, u8, "u8", width); }
            "i8" => { bench_analog_type!(&mut memory, i8, "i8", width); }
            "u16" => { bench_analog_type!(&mut memory, u16, "u16", width); }
            "i16" => { bench_analog_type!(&mut memory, i16, "i16", width); }
            "u32" => { bench_analog_type!(&mut memory, u32, "u32", width); }
            "i32" => { bench_analog_type!(&mut memory, i32, "i32", width); }
            "u64" => { bench_analog_type!(&mut memory, u64, "u64", width); }
            "i64" => { bench_analog_type!(&mut memory, i64, "i64", width); }
            _ => unreachable!(),
        }
    }
    bench_bytes(&mut memory);
}

fn main() {
    println!("BoardMemory benchmark: {DATA_POINTS} data points x {REPETITIONS} repetitions");
    println!("Each sample processes {DATA_POINTS} data points; {REPETITIONS} samples are summarized.");
    println!("Point methods and array element accesses count once per point; bulk byte methods count once per call.");
    println!("ns/op and ops/s use that operation count; bulk byte methods also show byte throughput.");
    println!("Array baseline measures direct u8 indexing/iteration, without BoardMemory address checks.");

    benchmark_board::<TwinCAT2>("TwinCAT2", AmsAddr::default());
    benchmark_board::<Comizoa>("Comizoa", BoardId(0));
    benchmark_board::<TwinCAT3>("TwinCAT3", AmsAddr::default());
    println!("\n[Direct array baseline]");
    bench_direct_array_access();
}
