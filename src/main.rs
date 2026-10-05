use virtualdevice::{AccessKind, AmsAddr, BoardMemory, Direction, IoAddress, TwinCAT3};

fn main() {
    let output = IoAddress::<TwinCAT3>::new(
        Direction::Output,
        0,
        0,
        AccessKind::Digital { bit_position: 0 },
    );
    let mut board = BoardMemory::<TwinCAT3>::new(AmsAddr::default(), 1, 1);

    board.write_digital_output(&output, true).expect("write output");
    println!("Digital output value: {}", board.read_digital_output(&output).expect("read output"));
}
