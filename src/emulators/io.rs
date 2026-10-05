/// I/O emulator.
pub struct IoEmulator {
    pub board_id: u8,
}

impl IoEmulator {
    pub fn new(board_id: u8) -> Self {
        Self { board_id }
    }
}
