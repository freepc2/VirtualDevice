use virtualdevice::{BoardMemory, BoardType, Comizoa, Direction, NetMemory, TwinCAT2, TwinCAT3};

fn net_with_five_boards<B: BoardType>() -> NetMemory<B> {
    let mut net = NetMemory::new();
    for _ in 0..5 {
        net.boards.push(BoardMemory::new(8, 8));
    }
    net
}

#[test]
fn net_ids_zero_through_four_select_boards() {
    let mut net = net_with_five_boards::<TwinCAT2>();

    for net_id in 0..5 {
        net.write_digital(net_id, Direction::Output, 0, 0, true)
            .unwrap();
        assert!(net.read_digital(net_id, Direction::Output, 0, 0).unwrap());
    }

    assert_eq!(net.boards.len(), 5);
}

#[test]
fn invalid_net_ids_return_errors_without_panicking() {
    let mut net = net_with_five_boards::<TwinCAT2>();

    assert!(net.read_digital(5, Direction::Output, 0, 0).is_err());
    assert!(net.read_digital(6, Direction::Output, 0, 0).is_err());
    assert!(net.write_digital(6, Direction::Output, 0, 0, true).is_err());
}

#[test]
fn digital_io_routes_to_the_requested_board_and_direction() {
    let mut net = net_with_five_boards::<TwinCAT2>();

    net.write_digital(2, Direction::Output, 1, 3, true).unwrap();

    assert!(net.read_digital(2, Direction::Output, 1, 3).unwrap());
    assert!(!net.read_digital(1, Direction::Output, 1, 3).unwrap());
    assert!(!net.read_digital(2, Direction::Input, 1, 3).unwrap());
}

#[test]
fn boards_read_and_write_digital_values() {
    let mut twincat = net_with_five_boards::<TwinCAT3>();
    twincat
        .write_digital(1, Direction::Output, 0, 3, true)
        .unwrap();
    assert!(twincat.read_digital(1, Direction::Output, 0, 3).unwrap());

    let mut comizoa = net_with_five_boards::<Comizoa>();
    comizoa
        .write_digital(4, Direction::Input, 0, 7, true)
        .unwrap();
    assert!(comizoa.read_digital(4, Direction::Input, 0, 7).unwrap());
}

#[test]
fn out_of_range_offsets_and_bit_positions_return_errors() {
    let mut net = net_with_five_boards::<TwinCAT2>();

    assert!(net.read_digital(1, Direction::Output, 8, 0).is_err());
    assert!(net.write_digital(1, Direction::Output, 0, 8, true).is_err());
}

#[test]
fn registered_io_survives_board_moves_and_shares_legacy_memory() {
    let mut board = BoardMemory::<TwinCAT2>::new(8, 8);
    board.register_io(10, Direction::Output, 2, 3).unwrap();
    board.register_io(11, Direction::Output, 2, 4).unwrap();
    board.register_io(10, Direction::Input, 2, 3).unwrap();
    let mut net = NetMemory::new();
    net.boards.push(board);
    // Moving the board list must preserve registered byte addresses.
    for _ in 0..32 {
        net.boards.push(BoardMemory::new(8, 8));
    }
    net.write_io(0, 10, Direction::Output, true).unwrap();
    net.write_io(0, 11, Direction::Output, true).unwrap();
    net.write_io(0, 10, Direction::Output, false).unwrap();
    assert!(net.read_digital(0, Direction::Output, 2, 4).unwrap());
    assert!(!net.read_io(0, 10, Direction::Input).unwrap());
    net.write_digital(0, Direction::Output, 2, 3, true).unwrap();
    assert!(net.read_io(0, 10, Direction::Output).unwrap());
    assert!(net.read_io(1, 10, Direction::Output).is_err());
}

#[test]
fn registered_byte_io_validates_and_can_replace_a_mapping() {
    let mut net = net_with_five_boards::<TwinCAT3>();
    assert!(net.register_io(5, 0, Direction::Output, 0, 0).is_err());
    assert!(net.register_io(0, 0, Direction::Output, 0, 8).is_err());
    assert!(net.register_io(0, 0, Direction::Output, 7, 1).is_err());
    assert!(
        net.register_io(0, 0, Direction::Output, u32::MAX, 1)
            .is_err()
    );
    assert!(net.write_io(0, 0, Direction::Output, true).is_err());
    net.register_io(0, 0, Direction::Output, 1, 2).unwrap();
    net.write_io(0, 0, Direction::Output, true).unwrap();
    assert!(net.read_digital(0, Direction::Output, 3, 0).unwrap());
    net.register_io(0, 0, Direction::Output, 4, 0).unwrap();
    assert!(!net.read_io(0, 0, Direction::Output).unwrap());
    assert!(net.read_digital(0, Direction::Output, 3, 0).unwrap());
}
