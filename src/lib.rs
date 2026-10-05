//! Virtual device memory core.
//!
//! Provides typed board, digital, analog, and byte access.

pub mod device_memory;
pub mod emulators;

pub use device_memory::{
    AccessKind, AccessMode, AddressError, AnalogValue, AmsAddr, Axis, BitPosition, BoardId,
    BoardMemory, BoardType, Comizoa, ComizoaLayout, DeviceType, Direction, IoAddress, IoNumber,
    IoType, IoValue, LocalServer, MemoryError, NetId, NetworkMemory, OffsetAddress, PhyAddress,
    PhysicalAddress, PhysicalIoAddress, SizeBytes, TwinCAT2, TwinCAT3, ValueKind,
};

pub use emulators::{IoEmulator, MotorEmulator};
