mod analog;
mod board;
mod board_address;
mod board_memory;

pub use board_address::{AccessKind, AmsAddr, BoardId, Direction, IoAddress};
pub use analog::AnalogValue;
pub use board::{BoardType, Comizoa, TwinCAT2, TwinCAT3};
pub use board_memory::BoardMemory;
