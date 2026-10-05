use core::marker::PhantomData;

use super::board::BoardType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction { Input, Output }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmsAddr { pub net_id: [u8; 6], pub port: u16 }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoardId(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessKind { Digital { bit_position: u8 }, Analog { size_bytes: u8 } }

#[derive(Clone, Copy, Debug)]
pub struct IoAddress<B: BoardType> {
    pub direction: Direction,
    pub phyaddress: u32,
    pub offset: u32,
    pub access_kind: AccessKind,
    pub(crate) board: PhantomData<B>,
}

impl<B: BoardType> IoAddress<B> {
    pub const fn new(direction: Direction, phyaddress: u32, offset: u32, access_kind: AccessKind) -> Self {
        Self { direction, phyaddress, offset, access_kind, board: PhantomData }
    }
}
