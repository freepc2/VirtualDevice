//! Virtual device memory core.
//!
//! Provides typed board, digital, analog, and byte access.

pub mod device_memory;

pub use device_memory::{
    AccessKind, AnalogValue, AmsAddr, BoardId, BoardMemory, BoardType, Comizoa, Direction,
    IoAddress, TwinCAT2, TwinCAT3,
};
