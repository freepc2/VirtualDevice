use core::ops::Range;

use super::board_address::{AccessKind, Direction, IoAddress};
use super::{AnalogValue, BoardType};

/// Input and output memory for one board.
pub struct BoardMemory<B: BoardType> {
    pub connection: B::Connection,
    pub inputs: Vec<u8>,
    pub outputs: Vec<u8>,
    pub input_start_address: u32,
    pub output_start_address: u32,
}

impl<B: BoardType> BoardMemory<B> {
    pub fn new(connection: B::Connection, input_size: usize, output_size: usize) -> Self {
        Self::new_with_start_addresses(connection, 0, input_size, 0, output_size)
    }

    pub fn new_with_start_addresses(
        connection: B::Connection,
        input_start_address: u32,
        input_size: usize,
        output_start_address: u32,
        output_size: usize,
    ) -> Self {
        Self {
            connection,
            inputs: vec![0; input_size],
            outputs: vec![0; output_size],
            input_start_address,
            output_start_address,
        }
    }

    #[inline]
    fn buffer(&self, dir: Direction) -> &[u8] {
        match dir {
            Direction::Input => &self.inputs,
            Direction::Output => &self.outputs,
        }
    }

    #[inline]
    fn buffer_mut(&mut self, dir: Direction) -> &mut [u8] {
        match dir {
            Direction::Input => &mut self.inputs,
            Direction::Output => &mut self.outputs,
        }
    }

    #[inline]
    fn start_address(&self, dir: Direction) -> u32 {
        match dir {
            Direction::Input => self.input_start_address,
            Direction::Output => self.output_start_address,
        }
    }

    #[inline]
    fn base_index(&self, addr: &IoAddress<B>) -> Result<usize, &'static str> {
        let absolute = u64::from(addr.phyaddress)
            .checked_add(u64::from(addr.offset))
            .ok_or("Out of bounds")?;
        let relative = absolute
            .checked_sub(u64::from(self.start_address(addr.direction)))
            .ok_or("Out of bounds")?;
        usize::try_from(relative).map_err(|_| "Out of bounds")
    }

    #[inline]
    fn digital_location(&self, addr: &IoAddress<B>) -> Result<(usize, u8), &'static str> {
        let AccessKind::Digital { bit_position } = addr.access_kind else {
            return Err("Access kind is not Digital");
        };
        if bit_position > if B::BOOL_IS_BYTE { 7 } else { 15 } {
            return Err("Invalid bit position for this board");
        }
        let base = self.base_index(addr)?;
        if B::BOOL_IS_BYTE {
            let index = base
                .checked_add(usize::from(bit_position))
                .ok_or("Out of bounds")?;
            Ok((index, 0))
        } else {
            let index = base
                .checked_add(usize::from(bit_position / 8))
                .ok_or("Out of bounds")?;
            Ok((index, bit_position % 8))
        }
    }

    #[inline]
    fn analog_range(&self, addr: &IoAddress<B>, width: usize) -> Result<Range<usize>, &'static str> {
        let AccessKind::Analog { size_bytes } = addr.access_kind else {
            return Err("Access kind is not Analog");
        };
        if usize::from(size_bytes) != width {
            return Err("size_bytes does not match the requested data type");
        }
        let start = self.base_index(addr)?;
        let end = start.checked_add(width).ok_or("Out of bounds")?;
        Ok(start..end)
    }

    /// Write a digital value.
    #[inline]
    pub fn write_digital(
        &mut self,
        addr: &IoAddress<B>,
        value: bool,
    ) -> Result<(), &'static str> {
        let (index, bit_position) = self.digital_location(addr)?;
        let byte = self
            .buffer_mut(addr.direction)
            .get_mut(index)
            .ok_or("Out of bounds")?;

        if B::BOOL_IS_BYTE {
            *byte = u8::from(value);
        } else if value {
            *byte |= 1 << bit_position;
        } else {
            *byte &= !(1 << bit_position);
        }
        Ok(())
    }

    /// Read a digital value.
    #[inline]
    pub fn read_digital(&self, addr: &IoAddress<B>) -> Result<bool, &'static str> {
        let (index, bit_position) = self.digital_location(addr)?;
        let byte = *self
            .buffer(addr.direction)
            .get(index)
            .ok_or("Out of bounds")?;
        Ok(if B::BOOL_IS_BYTE {
            byte != 0
        } else {
            byte & (1 << bit_position) != 0
        })
    }

    /// Write an analog value.
    pub fn write_analog<T: AnalogValue>(
        &mut self,
        addr: &IoAddress<B>,
        value: T,
    ) -> Result<(), &'static str> {
        let range = self.analog_range(addr, T::SIZE)?;
        let dst = self
            .buffer_mut(addr.direction)
            .get_mut(range)
            .ok_or("Out of bounds")?;
        value.write_le(dst);
        Ok(())
    }

    /// Read an analog value.
    pub fn read_analog<T: AnalogValue>(
        &self,
        addr: &IoAddress<B>,
    ) -> Result<T, &'static str> {
        let range = self.analog_range(addr, T::SIZE)?;
        let src = self
            .buffer(addr.direction)
            .get(range)
            .ok_or("Out of bounds")?;
        Ok(T::read_le(src))
    }

    #[inline]
    fn byte_range(
        &self,
        dir: Direction,
        phyaddress: u32,
        offset: u32,
        size: usize,
    ) -> Result<Range<usize>, &'static str> {
        let absolute = u64::from(phyaddress)
            .checked_add(u64::from(offset))
            .ok_or("Out of bounds")?;
        let start = usize::try_from(
            absolute
                .checked_sub(u64::from(self.start_address(dir)))
                .ok_or("Out of bounds")?,
        )
        .map_err(|_| "Out of bounds")?;
        let end = start.checked_add(size).ok_or("Out of bounds")?;
        Ok(start..end)
    }

    /// Read a byte range.
    #[inline]
    pub fn read_bytes(
        &self,
        dir: Direction,
        phyaddress: u32,
        offset: u32,
        size: usize,
    ) -> Result<&[u8], &'static str> {
        let range = self.byte_range(dir, phyaddress, offset, size)?;
        self.buffer(dir).get(range).ok_or("Out of bounds")
    }

    /// Write a byte range.
    #[inline]
    pub fn write_bytes(
        &mut self,
        dir: Direction,
        phyaddress: u32,
        offset: u32,
        data: &[u8],
    ) -> Result<(), &'static str> {
        let range = self.byte_range(dir, phyaddress, offset, data.len())?;
        let dst = self
            .buffer_mut(dir)
            .get_mut(range)
            .ok_or("Out of bounds")?;
        dst.copy_from_slice(data);
        Ok(())
    }

    /// Read a fixed-size byte array.
    #[inline]
    pub fn read_byte_array<const N: usize>(
        &self,
        dir: Direction,
        phyaddress: u32,
        offset: u32,
    ) -> Result<[u8; N], &'static str> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.read_bytes(dir, phyaddress, offset, N)?);
        Ok(out)
    }
}
