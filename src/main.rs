use virtualdevice::{DataType, DeviceMemory, MotorEmulator, PhyAddress};

fn main() {
    println!("=== Virtual Device Emulator - Main Entry Point ===");

    let mut memory = DeviceMemory::new();
    let mut motor = MotorEmulator::new(1);

    // TwinCAT 방식 주소 예시
    let twincat_addr = PhyAddress::from_twincat(1, 0x1000, 0, Some(0), DataType::Word);
    memory.write(twincat_addr, 1500); // 모터 속도 설정

    // 모터 에뮬레이터 틱 실행
    motor.tick(&mut memory, &twincat_addr);
    println!("Motor Axis {} Speed set to: {}", motor.axis_id, motor.speed);
}
