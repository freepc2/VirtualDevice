use virtualdevice::{DataType, DeviceMemory, PhyAddress};

#[test]
fn test_comizoa_auto_generation() {
    let addr = PhyAddress::from_comizoa(1, 0x2000, 25);
    // channel 25 -> offset = 25 / 16 = 1, bit = 25 % 16 = 9
    assert_eq!(addr.address, 0x2000);
    assert_eq!(addr.offset, 1);
    assert_eq!(addr.bit, Some(9));
    assert_eq!(addr.data_type, DataType::Bit);
}

#[test]
fn bench_memory_throughput_10k() {
    let mut mem = DeviceMemory::new();
    let count = 10_000;

    let start_write = std::time::Instant::now();
    for i in 0..count {
        let addr = PhyAddress::from_twincat(
            1,
            0x1000 + (i as u32),
            (i % 16) as u16,
            Some((i % 16) as u8),
            DataType::Bit,
        );
        mem.write_silent(addr, i as u16);
    }
    let duration_write = start_write.elapsed();

    let start_read = std::time::Instant::now();
    for i in 0..count {
        let addr = PhyAddress::from_twincat(
            1,
            0x1000 + (i as u32),
            (i % 16) as u16,
            Some((i % 16) as u8),
            DataType::Bit,
        );
        let _val = mem.read(&addr);
    }
    let duration_read = start_read.elapsed();

    println!("\n========================================");
    println!("🚀 Integration Benchmark (10K Items):");
    println!("- Write 10K: {:?} ({:.2} ops/sec)", duration_write, count as f64 / duration_write.as_secs_f64());
    println!("- Read  10K: {:?} ({:.2} ops/sec)", duration_read, count as f64 / duration_read.as_secs_f64());
    println!("========================================\n");

    assert_eq!(mem.storage.len(), count);
}
