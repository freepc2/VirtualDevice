use super::board_address::{AmsAddr, BoardId};

/// Board connection and digital layout rules.
pub trait BoardType: Sized + 'static {
    /// Board connection data.
    type Connection: Copy;

    /// Whether each digital value occupies one byte.
    const BOOL_IS_BYTE: bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TwinCAT2;

impl BoardType for TwinCAT2 {
    type Connection = AmsAddr;
    const BOOL_IS_BYTE: bool = false;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TwinCAT3;

impl BoardType for TwinCAT3 {
    type Connection = AmsAddr;
    const BOOL_IS_BYTE: bool = true;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Comizoa;

impl BoardType for Comizoa {
    type Connection = BoardId;
    const BOOL_IS_BYTE: bool = false;
}
