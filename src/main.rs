use virtualdevice::{IoType, LocalServer, NetId, PhyAddress};

fn main() {
    println!("=== Virtual Device Emulator (New Architecture) ===");

    let mut server = LocalServer::new();
    let net_id: NetId = 1;
    let module_id = 1;
    let io_number = 10;

    let output = PhyAddress::digital(net_id, module_id, io_number, IoType::DigitalOutput, 0, None);
    server.register(output).expect("failed to register output");

    server
        .write_digital(net_id, io_number, true)
        .expect("failed to write digital output");

    println!(
        "LocalServer initialized with {} network(s).",
        server.network_count()
    );
    println!(
        "Network {} registered module {} and digital output {} (value: {:?}).",
        net_id,
        module_id,
        io_number,
        server.read_digital(net_id, io_number),
    );
}
