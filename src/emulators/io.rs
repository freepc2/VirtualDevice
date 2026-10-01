/// I/O 에뮬레이터 모듈
pub struct IoEmulator {
    pub board_id: u8,
}

impl IoEmulator {
    pub fn new(board_id: u8) -> Self {
        Self { board_id }
    }
}
