use std::collections::HashSet;

use virtualdevice::{
    AddressError, ComizoaLayout, IoType, IoValue, LocalServer, MemoryError, NetId, PhyAddress,
    PhysicalIoAddress, ValueKind,
};

const NET_IDS: [NetId; 5] = [1, 2, 3, 4, 5];
const IO_POINTS_PER_KIND_PER_NETWORK: u16 = 100;
const OPERATIONS_PER_KIND: usize = 50_000;
const ONE_SECOND_DATA_PER_IO_TYPE_PER_NETWORK: u16 = 2_500;
const BENCHMARK_IO_TYPES: [IoType; 4] = [
    IoType::AnalogInput,
    IoType::AnalogOutput,
    IoType::DigitalInput,
    IoType::DigitalOutput,
];
type BenchmarkPoint = (NetId, u16, PhysicalIoAddress);

fn address(net_id: NetId, io_number: u16, io_type: IoType) -> PhyAddress {
    if io_type.is_digital() {
        PhyAddress::digital(net_id, 1, io_number, io_type, usize::from(io_number), None)
    } else {
        PhyAddress::analog(
            net_id,
            2,
            io_number,
            io_type,
            usize::from(io_number) * 2,
            ValueKind::U16,
        )
    }
}

#[test]
fn benchmarks_memory_creation_reading_and_overwriting_existing_addresses() {
    let mut server = LocalServer::new();
    let creation_start = std::time::Instant::now();
    for net_id in NET_IDS {
        for io_number in 0..IO_POINTS_PER_KIND_PER_NETWORK {
            server
                .register(address(net_id, io_number, IoType::DigitalOutput))
                .unwrap();
            server
                .register(address(net_id, io_number, IoType::AnalogOutput))
                .unwrap();
        }
    }
    let creation_elapsed = creation_start.elapsed();

    let unique_addresses: HashSet<_> = server
        .networks()
        .flat_map(|network| network.addresses().copied())
        .collect();

    assert_eq!(server.network_count(), 5);
    assert_eq!(unique_addresses.len(), 1_000);

    // Phase 2: read already registered addresses.
    let digital_read_start = std::time::Instant::now();
    for operation in 0..OPERATIONS_PER_KIND {
        let net_id = NET_IDS[(operation / IO_POINTS_PER_KIND_PER_NETWORK as usize) % NET_IDS.len()];
        let io_number = (operation % IO_POINTS_PER_KIND_PER_NETWORK as usize) as u16;
        assert!(matches!(
            std::hint::black_box(server.read_io(net_id, IoType::DigitalOutput, io_number,)),
            Some(IoValue::Bool(_))
        ));
    }
    let digital_read_elapsed = digital_read_start.elapsed();

    let analog_read_start = std::time::Instant::now();
    for operation in 0..OPERATIONS_PER_KIND {
        let net_id = NET_IDS[(operation / IO_POINTS_PER_KIND_PER_NETWORK as usize) % NET_IDS.len()];
        let io_number = (operation % IO_POINTS_PER_KIND_PER_NETWORK as usize) as u16;
        assert!(matches!(
            std::hint::black_box(server.read_io(net_id, IoType::AnalogOutput, io_number,)),
            Some(IoValue::U16(_))
        ));
    }
    let analog_read_elapsed = analog_read_start.elapsed();

    // Phase 3: overwrite values at the same, already registered addresses.
    let digital_write_start = std::time::Instant::now();
    for operation in 0..OPERATIONS_PER_KIND {
        let net_id = NET_IDS[(operation / IO_POINTS_PER_KIND_PER_NETWORK as usize) % NET_IDS.len()];
        let io_number = (operation % IO_POINTS_PER_KIND_PER_NETWORK as usize) as u16;
        server
            .write_io(
                net_id,
                IoType::DigitalOutput,
                io_number,
                IoValue::Bool(operation % 2 == 0),
            )
            .unwrap();
    }
    let digital_write_elapsed = digital_write_start.elapsed();

    let analog_write_start = std::time::Instant::now();
    for operation in 0..OPERATIONS_PER_KIND {
        let net_id = NET_IDS[(operation / IO_POINTS_PER_KIND_PER_NETWORK as usize) % NET_IDS.len()];
        let io_number = (operation % IO_POINTS_PER_KIND_PER_NETWORK as usize) as u16;
        server
            .write_io(
                net_id,
                IoType::AnalogOutput,
                io_number,
                IoValue::U16(operation as u16),
            )
            .unwrap();
    }
    let analog_write_elapsed = analog_write_start.elapsed();

    let ops_per_second =
        |count: usize, elapsed: std::time::Duration| count as f64 / elapsed.as_secs_f64();

    println!("\n=== IONumber-based benchmark: 5 NetIds, 1,000 unique PhyAddress ===");
    println!(
        "1. Memory creation: {} addresses in {:.3} ms ({:.0} addresses/sec)",
        unique_addresses.len(),
        creation_elapsed.as_secs_f64() * 1_000.0,
        ops_per_second(unique_addresses.len(), creation_elapsed),
    );
    println!(
        "2. Read existing addresses: Digital {} ops in {:.3} ms ({:.0} ops/sec)",
        OPERATIONS_PER_KIND,
        digital_read_elapsed.as_secs_f64() * 1_000.0,
        ops_per_second(OPERATIONS_PER_KIND, digital_read_elapsed),
    );
    println!(
        "   Read existing addresses: Analog  {} ops in {:.3} ms ({:.0} ops/sec)",
        OPERATIONS_PER_KIND,
        analog_read_elapsed.as_secs_f64() * 1_000.0,
        ops_per_second(OPERATIONS_PER_KIND, analog_read_elapsed),
    );
    println!(
        "3. Overwrite existing addresses: Digital {} ops in {:.3} ms ({:.0} ops/sec)",
        OPERATIONS_PER_KIND,
        digital_write_elapsed.as_secs_f64() * 1_000.0,
        ops_per_second(OPERATIONS_PER_KIND, digital_write_elapsed),
    );
    println!(
        "   Overwrite existing addresses: Analog  {} ops in {:.3} ms ({:.0} ops/sec)",
        OPERATIONS_PER_KIND,
        analog_write_elapsed.as_secs_f64() * 1_000.0,
        ops_per_second(OPERATIONS_PER_KIND, analog_write_elapsed),
    );

    for net_id in NET_IDS {
        let network = server.network(net_id).unwrap();
        assert_eq!(network.digital_count(), 100);
        assert_eq!(network.analog_count(), 100);

        for number in 0..IO_POINTS_PER_KIND_PER_NETWORK {
            let last_operation = 49_500 + (usize::from(net_id) - 1) * 100 + usize::from(number);
            assert_eq!(
                server.read_digital(net_id, number),
                Some(IoValue::Bool(number % 2 == 0))
            );
            assert_eq!(
                server.read_analog(net_id, number),
                Some(IoValue::U16(last_operation as u16))
            );
        }
    }
}

#[test]
fn benchmarks_one_second_read_and_write_for_each_io_type_and_access_mode() {
    let mut server = LocalServer::new();
    let mut keys_by_type: [Vec<BenchmarkPoint>; 4] = std::array::from_fn(|_| Vec::new());

    for net_id in NET_IDS {
        for io_type in BENCHMARK_IO_TYPES {
            let number_base = if io_type.is_output() {
                ONE_SECOND_DATA_PER_IO_TYPE_PER_NETWORK
            } else {
                0
            };
            for local_number in 0..ONE_SECOND_DATA_PER_IO_TYPE_PER_NETWORK {
                let io_number = number_base + local_number;
                let point = address(net_id, io_number, io_type);
                server.register(point).unwrap();
                keys_by_type[benchmark_type_index(io_type)].push((
                    net_id,
                    io_number,
                    point.physical_key(),
                ));
            }
        }
    }

    let unique_addresses: HashSet<_> = server
        .networks()
        .flat_map(|network| network.addresses().copied())
        .collect();
    assert_eq!(unique_addresses.len(), 50_000);

    let duration = std::time::Duration::from_secs(1);
    for (mode_name, physical) in [("IONumber-based", false), ("Physical-address-based", true)] {
        println!("\n=== {mode_name}: 1 second per I/O type ===");
        for io_type in BENCHMARK_IO_TYPES {
            let points = &keys_by_type[benchmark_type_index(io_type)];
            let (reads, writes, elapsed) =
                benchmark_io_type(&mut server, points, io_type, physical, duration);
            println!(
                "{io_type:?}: {} addresses; Read {reads} ({:.0} ops/sec), Write {writes} ({:.0} ops/sec) in {:.3} s",
                points.len(),
                reads as f64 / elapsed.as_secs_f64(),
                writes as f64 / elapsed.as_secs_f64(),
                elapsed.as_secs_f64(),
            );
        }
    }
}

fn benchmark_type_index(io_type: IoType) -> usize {
    match io_type {
        IoType::AnalogInput => 0,
        IoType::AnalogOutput => 1,
        IoType::DigitalInput => 2,
        IoType::DigitalOutput => 3,
    }
}

fn benchmark_io_type(
    server: &mut LocalServer,
    points: &[BenchmarkPoint],
    io_type: IoType,
    physical: bool,
    duration: std::time::Duration,
) -> (usize, usize, std::time::Duration) {
    let start = std::time::Instant::now();
    let mut reads = 0usize;
    let mut writes = 0usize;
    while start.elapsed() < duration {
        for &(net_id, io_number, physical_key) in points {
            let value = if physical {
                server.read_physical(net_id, &physical_key)
            } else {
                server.read_io(net_id, io_type, io_number)
            };
            if io_type.is_digital() {
                assert!(matches!(
                    std::hint::black_box(value),
                    Some(IoValue::Bool(_))
                ));
            } else {
                assert!(matches!(std::hint::black_box(value), Some(IoValue::U16(_))));
            }
            reads += 1;

            let value = if io_type.is_digital() {
                IoValue::Bool(writes % 2 == 0)
            } else {
                IoValue::U16(writes as u16)
            };
            if physical {
                server.write_physical(net_id, &physical_key, value).unwrap();
            } else {
                server.write_io(net_id, io_type, io_number, value).unwrap();
            }
            writes += 1;
        }
    }
    (reads, writes, start.elapsed())
}

#[test]
fn logical_and_physical_access_share_values_without_io_number_in_physical_key() {
    let mut server = LocalServer::new();
    let digital = PhyAddress::digital(1, 10, 500, IoType::DigitalOutput, 12, Some(9));
    let analog = PhyAddress::analog(1, 20, 500, IoType::AnalogInput, 48, ValueKind::I32);
    server.register_all([analog, digital]).unwrap();
    let digital_key = PhysicalIoAddress {
        physical_address: 10,
        offset: 12,
        bit_position: Some(9),
        size_bytes: 2,
        io_type: IoType::DigitalOutput,
    };
    server.write_digital(1, 500, true).unwrap();
    assert_eq!(
        server.read_physical(1, &digital_key),
        Some(IoValue::Bool(true))
    );
    server
        .write_physical(1, &digital.physical_key(), IoValue::Bool(false))
        .unwrap();
    assert_eq!(server.read_digital(1, 500), Some(IoValue::Bool(false)));
    server.write_analog(1, 500, IoValue::I32(-123)).unwrap();
    assert_eq!(
        server.read_physical(1, &analog.physical_key()),
        Some(IoValue::I32(-123))
    );
    server
        .write_physical(1, &analog.physical_key(), IoValue::I32(-456))
        .unwrap();
    assert_eq!(server.read_analog(1, 500), Some(IoValue::I32(-456)));
    let changed_number = PhyAddress {
        io_number: 999,
        ..analog
    };
    assert_eq!(server.read_phy(&changed_number), Some(IoValue::I32(-456)));
    assert_eq!(server.read_physical(2, &digital.physical_key()), None);
    assert!(matches!(
        server.write_physical(2, &digital.physical_key(), IoValue::Bool(true)),
        Err(MemoryError::NetworkNotFound(2))
    ));
    server
        .register(PhyAddress {
            net_id: 2,
            ..digital
        })
        .unwrap();
    server.write_digital(2, 500, true).unwrap();
    assert_eq!(server.read_digital(1, 500), Some(IoValue::Bool(false)));
    assert_eq!(server.read_digital(2, 500), Some(IoValue::Bool(true)));
    println!("[IONumber-based] Digital/Analog values written and read");
    println!("[Physical-address-based] Same values resolved; NetId isolation verified");
}

#[test]
fn duplicate_numbers_are_rejected_across_input_and_output_in_either_order() {
    for digital in [true, false] {
        for input_first in [true, false] {
            let mut server = LocalServer::new();
            let make = |input: bool| {
                let io_type = match (digital, input) {
                    (true, true) => IoType::DigitalInput,
                    (true, false) => IoType::DigitalOutput,
                    (false, true) => IoType::AnalogInput,
                    (false, false) => IoType::AnalogOutput,
                };
                address(1, 7, io_type)
            };
            let first = make(input_first);
            server.register(first).unwrap();
            assert_eq!(
                server.register(make(!input_first)),
                Err(MemoryError::DuplicateIoNumber {
                    io_number: 7,
                    digital
                })
            );
            assert_eq!(server.network(1).unwrap().addresses().count(), 1);
            assert!(server.read_io(1, first.io_type, 7).is_some());
            assert_eq!(server.read_io(1, make(!input_first).io_type, 7), None);
        }
    }
}

#[test]
fn wrong_direction_type_missing_location_and_number_do_not_modify_values() {
    let mut server = LocalServer::new();
    let analog = address(1, 7, IoType::AnalogInput);
    server.register(analog).unwrap();
    server.write_analog(1, 7, IoValue::U16(42)).unwrap();
    assert!(matches!(
        server.write_io(1, IoType::AnalogOutput, 7, IoValue::U16(100)),
        Err(MemoryError::DirectionMismatch { .. })
    ));
    assert!(matches!(
        server.write_physical(1, &analog.physical_key(), IoValue::I16(100)),
        Err(MemoryError::ValueTypeMismatch { .. })
    ));
    assert_eq!(
        server.write_analog(1, 8, IoValue::U16(1)),
        Err(MemoryError::IoNotFound(8))
    );
    let mut unknown = analog.physical_key();
    unknown.offset += 1;
    assert_eq!(server.read_physical(1, &unknown), None);
    assert_eq!(
        server.write_physical(1, &unknown, IoValue::U16(1)),
        Err(MemoryError::PhysicalAddressNotFound)
    );
    assert_eq!(server.read_analog(1, 7), Some(IoValue::U16(42)));
}

#[test]
fn explicit_offsets_allow_mixed_analog_sizes_and_reject_physical_overlap() {
    let mut server = LocalServer::new();
    let wide = PhyAddress::analog(1, 1, 100, IoType::AnalogOutput, 8, ValueKind::U64);
    let narrow = PhyAddress::analog(1, 1, 200, IoType::AnalogOutput, 16, ValueKind::U32);
    server.register_all([narrow, wide]).unwrap();
    server.write_analog(1, 100, IoValue::U64(u64::MAX)).unwrap();
    server.write_analog(1, 200, IoValue::U32(42)).unwrap();
    assert_eq!(server.read_phy(&wide), Some(IoValue::U64(u64::MAX)));
    let overlapping = PhyAddress::analog(1, 1, 300, IoType::AnalogOutput, 12, ValueKind::U32);
    assert!(matches!(
        server.register(overlapping),
        Err(MemoryError::PhysicalOverlap { .. })
    ));
    assert_eq!(server.read_analog(1, 300), None);
    let digital = PhyAddress::digital(1, 1, 300, IoType::DigitalOutput, 8, None);
    assert!(matches!(
        server.register(digital),
        Err(MemoryError::PhysicalOverlap { .. })
    ));
    assert_eq!(server.read_digital(1, 300), None);
    // The same offset in the input direction refers to a different external image.
    server
        .register(PhyAddress {
            io_type: IoType::DigitalInput,
            ..digital
        })
        .unwrap();
}

#[test]
fn comizoa_uses_channel_layout_in_any_registration_order_and_shares_bit_words() {
    let layout = ComizoaLayout {
        digital_channels: 32,
        analog_channels: 8,
        analog_kind: ValueKind::U16,
    };
    let digital = PhyAddress::from_comizoa(1, 1, 900, IoType::DigitalOutput, 25, layout).unwrap();
    let adjacent = PhyAddress::from_comizoa(1, 1, 100, IoType::DigitalOutput, 24, layout).unwrap();
    let analog = PhyAddress::from_comizoa(1, 1, 900, IoType::AnalogOutput, 1, layout).unwrap();
    assert_eq!((digital.offset, digital.bit_position), (2, Some(9)));
    assert_eq!(analog.offset, 6);
    for analog_first in [true, false] {
        let mut server = LocalServer::new();
        let order = if analog_first {
            [analog, digital, adjacent]
        } else {
            [digital, adjacent, analog]
        };
        server.register_all(order).unwrap();
        server.write_digital(1, 900, true).unwrap();
        server.write_digital(1, 100, false).unwrap();
        server.write_analog(1, 900, IoValue::U16(1234)).unwrap();
        assert_eq!(server.read_phy(&digital), Some(IoValue::Bool(true)));
        assert_eq!(server.read_phy(&adjacent), Some(IoValue::Bool(false)));
        assert_eq!(server.read_phy(&analog), Some(IoValue::U16(1234)));
        assert!(matches!(
            server.register(PhyAddress {
                io_number: 901,
                ..digital
            }),
            Err(MemoryError::PhysicalOverlap { .. })
        ));
    }
    assert!(matches!(
        PhyAddress::from_comizoa(1, 1, 0, IoType::DigitalOutput, 32, layout),
        Err(AddressError::ChannelOutOfRange { .. })
    ));
}

#[test]
fn invalid_addresses_fail_before_registration_and_sparse_numbers_are_supported() {
    let mut server = LocalServer::new();
    let digital = PhyAddress::digital(1, 1, u16::MAX, IoType::DigitalOutput, 12, Some(15));
    for invalid in [
        PhyAddress {
            bit_position: Some(16),
            ..digital
        },
        PhyAddress {
            size_bytes: 1,
            ..digital
        },
        PhyAddress {
            offset: usize::MAX,
            ..digital
        },
    ] {
        assert!(matches!(
            server.register(invalid),
            Err(MemoryError::InvalidAddress(_))
        ));
        assert_eq!(server.network_count(), 0);
    }
    server.register(digital).unwrap();
    server.write_digital(1, u16::MAX, true).unwrap();
    assert_eq!(server.read_phy(&digital), Some(IoValue::Bool(true)));
    assert_eq!(server.read_digital(1, 0), None);
    assert_eq!(server.network(1).unwrap().digital_count(), 1);
}

#[test]
fn every_integer_kind_round_trips_through_both_access_paths() {
    let cases = [
        (ValueKind::U8, IoValue::U8(u8::MAX)),
        (ValueKind::I8, IoValue::I8(i8::MIN)),
        (ValueKind::U16, IoValue::U16(u16::MAX)),
        (ValueKind::I16, IoValue::I16(i16::MIN)),
        (ValueKind::U32, IoValue::U32(u32::MAX)),
        (ValueKind::I32, IoValue::I32(i32::MIN)),
        (ValueKind::U64, IoValue::U64(u64::MAX)),
        (ValueKind::I64, IoValue::I64(i64::MIN)),
    ];
    let mut server = LocalServer::new();
    for (number, (kind, value)) in cases.into_iter().enumerate() {
        let address =
            PhyAddress::analog(1, 1, number as u16, IoType::AnalogOutput, number * 8, kind);
        server.register(address).unwrap();
        server
            .write_physical(1, &address.physical_key(), value)
            .unwrap();
        assert_eq!(server.read_analog(1, number as u16), Some(value));
        server.write_analog(1, number as u16, value).unwrap();
        assert_eq!(
            server.read_physical(1, &address.physical_key()),
            Some(value)
        );
    }
}

#[test]
fn shared_word_overlap_error_identifies_the_actual_bit_owner() {
    let mut server = LocalServer::new();
    let bit0 = PhyAddress::digital(1, 1, 10, IoType::DigitalOutput, 0, Some(0));
    let bit1 = PhyAddress::digital(1, 1, 11, IoType::DigitalOutput, 0, Some(1));
    server.register_all([bit0, bit1]).unwrap();
    assert_eq!(
        server.register(PhyAddress {
            io_number: 12,
            ..bit0
        }),
        Err(MemoryError::PhysicalOverlap {
            io_number: 12,
            existing_io_number: 10,
            offset: 0
        })
    );
    assert_eq!(server.network(1).unwrap().digital_count(), 2);
}

#[test]
fn network_registration_rejects_wrong_net_id_without_creating_a_slot() {
    let mut network = virtualdevice::NetworkMemory::new(1);
    let point = address(2, 7, IoType::DigitalOutput);
    assert_eq!(
        network.register(point),
        Err(MemoryError::NetworkMismatch {
            expected: 1,
            actual: 2
        })
    );
    assert_eq!(network.addresses().count(), 0);
}
