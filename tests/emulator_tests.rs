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
fn bench_write_throughput_1s() {
    let mut mem = DeviceMemory::new();
    let duration = std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();
    let mut count: u64 = 0;

    while start.elapsed() < duration {
        let is_dio = count % 2 == 0;
        let board_id = ((count / 65536) % 256) as u8;
        let offset = (count % 65536) as u16;
        let base_addr = if is_dio { 0x1000 } else { 0x2000 };

        let addr = if is_dio {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                Some((count % 16) as u8),
                DataType::Bit,
            )
        } else {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                None,
                DataType::Word,
            )
        };
        mem.write_silent(addr, (count % 65536) as u16);
        count += 1;
    }

    let total_items = mem.storage.len();
    let dio_count = mem.storage.keys().filter(|a| a.bit.is_some() || a.data_type == DataType::Bit).count();
    let aio_count = total_items - dio_count;
    let approx_memory_bytes = total_items * (std::mem::size_of::<PhyAddress>() + std::mem::size_of::<u16>());

    println!("\n========================================");
    println!("🚀 1-Second New Address Generation & Write Benchmark:");
    println!("- Total Unique Writes in 1s: {} ops", count);
    println!("- Stored Items in Memory: {}", total_items);
    println!("- DIO Created: {}", dio_count);
    println!("- AIO Created: {}", aio_count);
    println!("- Approx Memory Usage: {} bytes ({:.2} MB)", approx_memory_bytes, approx_memory_bytes as f64 / (1024.0 * 1024.0));
    println!("========================================\n");

    assert_eq!(count, total_items as u64);
}

#[test]
fn bench_read_throughput_1s() {
    let mut mem = DeviceMemory::new();
    let pre_count = 500_000;
    
    for i in 0..pre_count {
        let is_dio = i % 2 == 0;
        let board_id = ((i / 65536) % 256) as u8;
        let offset = (i % 65536) as u16;
        let base_addr = if is_dio { 0x1000 } else { 0x2000 };

        let addr = if is_dio {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                Some((i % 16) as u8),
                DataType::Bit,
            )
        } else {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                None,
                DataType::Word,
            )
        };
        mem.write_silent(addr, i as u16);
    }

    let duration = std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();
    let mut count: u64 = 0;

    while start.elapsed() < duration {
        let idx = (count % pre_count) as usize;
        let is_dio = idx % 2 == 0;
        let board_id = ((idx / 65536) % 256) as u8;
        let offset = (idx % 65536) as u16;
        let base_addr = if is_dio { 0x1000 } else { 0x2000 };

        let addr = if is_dio {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                Some((idx % 16) as u8),
                DataType::Bit,
            )
        } else {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                None,
                DataType::Word,
            )
        };
        let _val = mem.read(&addr);
        count += 1;
    }

    let total_items = mem.storage.len();
    let dio_count = mem.storage.keys().filter(|a| a.bit.is_some() || a.data_type == DataType::Bit).count();
    let aio_count = total_items - dio_count;
    let approx_memory_bytes = total_items * (std::mem::size_of::<PhyAddress>() + std::mem::size_of::<u16>());

    println!("\n========================================");
    println!("🚀 1-Second Read Throughput (Existing 500K Memory):");
    println!("- Total Reads in 1s: {} ops", count);
    println!("- Pre-populated Items: {}", total_items);
    println!("- DIO Count: {}", dio_count);
    println!("- AIO Count: {}", aio_count);
    println!("- Approx Memory Usage: {} bytes ({:.2} MB)", approx_memory_bytes, approx_memory_bytes as f64 / (1024.0 * 1024.0));
    println!("========================================\n");

    assert!(count > 0);
}

#[test]
fn bench_update_existing_throughput_1s() {
    let mut mem = DeviceMemory::new();
    let pre_count = 500_000;

    for i in 0..pre_count {
        let is_dio = i % 2 == 0;
        let board_id = ((i / 65536) % 256) as u8;
        let offset = (i % 65536) as u16;
        let base_addr = if is_dio { 0x1000 } else { 0x2000 };

        let addr = if is_dio {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                Some((i % 16) as u8),
                DataType::Bit,
            )
        } else {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                None,
                DataType::Word,
            )
        };
        mem.write_silent(addr, i as u16);
    }

    let duration = std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();
    let mut count: u64 = 0;

    while start.elapsed() < duration {
        let idx = (count % pre_count) as usize;
        let is_dio = idx % 2 == 0;
        let board_id = ((idx / 65536) % 256) as u8;
        let offset = (idx % 65536) as u16;
        let base_addr = if is_dio { 0x1000 } else { 0x2000 };

        let addr = if is_dio {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                Some((idx % 16) as u8),
                DataType::Bit,
            )
        } else {
            PhyAddress::from_twincat(
                board_id,
                base_addr,
                offset,
                None,
                DataType::Word,
            )
        };
        mem.write_silent(addr, (count % 65536) as u16);
        count += 1;
    }

    let total_items = mem.storage.len();
    let dio_count = mem.storage.keys().filter(|a| a.bit.is_some() || a.data_type == DataType::Bit).count();
    let aio_count = total_items - dio_count;
    let approx_memory_bytes = total_items * (std::mem::size_of::<PhyAddress>() + std::mem::size_of::<u16>());

    println!("\n========================================");
    println!("🚀 1-Second Update/Overwrite Throughput (Existing 500K Memory):");
    println!("- Total Updates/Writes in 1s: {} ops", count);
    println!("- Pre-populated Items: {}", total_items);
    println!("- DIO Count: {}", dio_count);
    println!("- AIO Count: {}", aio_count);
    println!("- Approx Memory Usage: {} bytes ({:.2} MB)", approx_memory_bytes, approx_memory_bytes as f64 / (1024.0 * 1024.0));
    println!("========================================\n");

    assert_eq!(total_items, pre_count as usize);
}
