use virtualdevice::{
    AccessKind, AmsAddr, BoardId, BoardMemory, Comizoa, Direction, IoAddress, TwinCAT2, TwinCAT3,
};

#[test]
fn bit_addressed_boards_preserve_other_bits_in_the_same_byte() {
    let bit_zero = IoAddress::<TwinCAT2>::new(
        Direction::Output,
        0,
        0,
        AccessKind::Digital { bit_position: 0 },
    );
    let bit_three = IoAddress::<TwinCAT2>::new(
        Direction::Output,
        0,
        0,
        AccessKind::Digital { bit_position: 3 },
    );
    let mut memory = BoardMemory::<TwinCAT2>::new(AmsAddr::default(), 2, 2);

    memory.write_digital_output(&bit_zero, true).unwrap();
    memory.write_digital_output(&bit_three, true).unwrap();
    memory.write_digital_output(&bit_zero, false).unwrap();

    assert!(!memory.read_digital_output(&bit_zero).unwrap());
    assert!(memory.read_digital_output(&bit_three).unwrap());
    assert_eq!(memory.outputs[0], 0b0000_1000);
}

#[test]
fn packed_sixteen_bit_addresses_access_both_bytes_of_a_word() {
    let bit_eight = IoAddress::<TwinCAT2>::new(
        Direction::Output,
        100,
        0,
        AccessKind::Digital { bit_position: 8 },
    );
    let mut memory = BoardMemory::<TwinCAT2>::new_with_start_addresses(
        AmsAddr::default(),
        100,
        2,
        100,
        2,
    );

    memory.write_digital_output(&bit_eight, true).unwrap();
    assert!(memory.read_digital_output(&bit_eight).unwrap());
    assert_eq!(memory.outputs, [0, 1]);
}

#[test]
fn board_memory_addresses_are_relative_to_configured_image_start() {
    let output = IoAddress::<TwinCAT3>::new(
        Direction::Output,
        1000,
        5,
        AccessKind::Analog { size_bytes: 2 },
    );
    let mut memory = BoardMemory::<TwinCAT3>::new_with_start_addresses(
        AmsAddr::default(),
        500,
        4,
        1000,
        8,
    );

    memory.write_analog(&output, 0x1234_u16).unwrap();
    assert_eq!(memory.outputs[5..7], [0x34, 0x12]);
    assert_eq!(memory.read_analog::<u16>(&output).unwrap(), 0x1234);
}

#[test]
fn byte_addressed_boards_store_each_digital_value_in_its_own_byte() {
    let first = IoAddress::<TwinCAT3>::new(
        Direction::Output,
        0,
        0,
        AccessKind::Digital { bit_position: 0 },
    );
    let second = IoAddress::<TwinCAT3>::new(
        Direction::Output,
        0,
        1,
        AccessKind::Digital { bit_position: 0 },
    );
    let mut memory = BoardMemory::<TwinCAT3>::new(AmsAddr::default(), 2, 2);

    memory.write_digital_output(&first, true).unwrap();
    memory.write_digital_output(&second, true).unwrap();
    memory.write_digital_output(&first, false).unwrap();

    assert!(!memory.read_digital_output(&first).unwrap());
    assert!(memory.read_digital_output(&second).unwrap());
    assert_eq!(memory.outputs, [0, 1]);
}

#[test]
fn analog_values_round_trip_as_little_endian_for_input_and_output_buffers() {
    let output = IoAddress::<Comizoa>::new(
        Direction::Output,
        0,
        2,
        AccessKind::Analog { size_bytes: 4 },
    );
    let input = IoAddress::<Comizoa>::new(
        Direction::Input,
        0,
        0,
        AccessKind::Analog { size_bytes: 2 },
    );
    let mut memory = BoardMemory::<Comizoa>::new(BoardId(7), 4, 8);

    memory.write_analog(&output, 0x1234_5678_u32).unwrap();
    memory.write_analog(&input, -1234_i16).unwrap();

    assert_eq!(memory.outputs[2..6], [0x78, 0x56, 0x34, 0x12]);
    assert_eq!(memory.read_analog::<u32>(&output).unwrap(), 0x1234_5678);
    assert_eq!(memory.read_analog::<i16>(&input).unwrap(), -1234);
}

#[test]
fn invalid_access_kinds_sizes_bits_and_ranges_return_errors() {
    let digital = IoAddress::<TwinCAT2>::new(
        Direction::Output,
        0,
        0,
        AccessKind::Digital { bit_position: 8 },
    );
    let analog = IoAddress::<TwinCAT2>::new(
        Direction::Output,
        0,
        0,
        AccessKind::Analog { size_bytes: 2 },
    );
    let wrong_kind = IoAddress::<TwinCAT2>::new(
        Direction::Output,
        0,
        0,
        AccessKind::Digital { bit_position: 0 },
    );
    let out_of_bounds = IoAddress::<TwinCAT2>::new(
        Direction::Input,
        1,
        1,
        AccessKind::Analog { size_bytes: 2 },
    );
    let mut memory = BoardMemory::<TwinCAT2>::new(AmsAddr::default(), 2, 2);

    assert!(memory.write_digital_output(&digital, true).is_err());
    assert!(memory.write_analog(&analog, 1_u32).is_err());
    assert!(memory.write_analog(&wrong_kind, 1_u16).is_err());
    assert!(memory.read_analog::<u16>(&out_of_bounds).is_err());
    assert!(memory.read_bytes(Direction::Output, 0, 2, 1).is_err());
    assert!(memory.write_bytes(Direction::Input, 0, 3, &[1, 2]).is_err());
}

#[test]
fn raw_byte_and_fixed_array_access_target_the_selected_buffer() {
    let mut memory = BoardMemory::<TwinCAT2>::new(AmsAddr::default(), 4, 4);

    memory
        .write_bytes(Direction::Input, 1, 1, &[10, 20])
        .unwrap();
    memory
        .write_bytes(Direction::Output, 0, 2, &[30, 40])
        .unwrap();

    assert_eq!(memory.inputs, [0, 0, 10, 20]);
    assert_eq!(memory.read_byte_array::<2>(Direction::Output, 0, 2).unwrap(), [30, 40]);
    assert_eq!(memory.read_bytes(Direction::Input, 0, 0, 4).unwrap(), &[0, 0, 10, 20]);
}

#[test]
fn motor_emulator_reads_u16_speed_from_the_registered_address() {
    use virtualdevice::emulators::MotorEmulator;
    use virtualdevice::{IoType, IoValue, LocalServer, PhyAddress, ValueKind};

    let address = PhyAddress::analog(1, 1, 12, IoType::AnalogInput, 0, ValueKind::U16);
    let mut server = LocalServer::new();
    server.register(address).unwrap();
    server.write_analog(1, 12, IoValue::U16(1500)).unwrap();

    let mut motor = MotorEmulator::new(3);
    motor.tick(&server, &address);
    assert_eq!(motor.axis_id, 3);
    assert_eq!(motor.speed, 1500);
}
