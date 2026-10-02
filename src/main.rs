use virtualdevice::{LocalServer, NetworkMemory, IoPoint, ModuleMemory};

fn main() {
    println!("=== Virtual Device Emulator (New Architecture) ===");

    let mut server = LocalServer::new("DEV-001");
    let net_id = 1;

    let mut net_mem = NetworkMemory {
        net_id,
        ..Default::default()
    };

    let module_id = 1;
    net_mem.modules.insert(module_id, ModuleMemory {
        input_image: vec![0; 10],
        output_image: vec![0; 10],
    });

    net_mem.dio_points.resize(11, None);
    net_mem.dio_points[10] = Some(IoPoint::Digital {
        module_id,
        is_output: true,
        offset: 0,
        bit_mask: Some(0x01),
    });

    server.memory.networks.insert(net_id, net_mem);

    println!("LocalServer initialized for Device: {}", server.device_id);
    println!("Network {} registered with module memory and DIO point.", net_id);
}
