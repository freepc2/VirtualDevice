use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

struct IoEntry {
    byte: NonNull<u8>,
    mask: u8,
}

pub trait BoardType {
    const BIT_IS_BYTE: bool;
}

pub struct TwinCAT2;
pub struct TwinCAT3;
pub struct Comizoa;

impl BoardType for TwinCAT2 {
    const BIT_IS_BYTE: bool = false;
}

impl BoardType for TwinCAT3 {
    const BIT_IS_BYTE: bool = true;
}

impl BoardType for Comizoa {
    const BIT_IS_BYTE: bool = false;
}

pub enum Direction {
    Input,
    Output,
}

pub struct BoardMemory<B: BoardType> {
    inputs: Vec<UnsafeCell<u8>>,
    outputs: Vec<UnsafeCell<u8>>,
    input_io: Vec<Option<IoEntry>>,
    output_io: Vec<Option<IoEntry>>,
    pub board: PhantomData<B>,
}

impl<B: BoardType> BoardMemory<B> {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        Self {
            inputs: (0..input_size).map(|_| UnsafeCell::new(0)).collect(),
            outputs: (0..output_size).map(|_| UnsafeCell::new(0)).collect(),
            input_io: Vec::new(),
            output_io: Vec::new(),
            board: PhantomData,
        }
    }

    #[inline(always)]
    pub fn read_digital(
        &self,
        direction: Direction,
        offset: u32,
        bitposition: u32,
    ) -> Result<bool, &'static str> {
        if bitposition > 7 {
            return Err("Invalid bit position (0 ~ 7)");
        }

        let buffer = match direction {
            Direction::Input => &self.inputs,
            Direction::Output => &self.outputs,
        };

        // 컴파일 타임에 결정되는 보드 특성에 따라 인덱스를 계산합니다.
        let index: u32 = if B::BIT_IS_BYTE {
            offset.checked_add(bitposition).ok_or("Address overflow")?
        } else {
            offset
        };

        if index as usize >= buffer.len() {
            return Err("Out of bounds");
        }
        let byte = unsafe { *buffer.get_unchecked(index as usize).get() };

        Ok(if B::BIT_IS_BYTE {
            byte != 0
        } else {
            let mask: u8 = 1 << bitposition;
            (byte & mask) != 0
        })
    }

    #[inline(always)]
    pub fn write_digital(
        &mut self,
        direction: Direction,
        offset: u32,
        bitposition: u32,
        value: bool,
    ) -> Result<(), &'static str> {
        if bitposition > 7 {
            return Err("Invalid bit position (0 ~ 7)");
        }

        let buffer = match direction {
            Direction::Input => &self.inputs,
            Direction::Output => &self.outputs,
        };

        // 컴파일 타임에 결정되는 보드 특성에 따라 인덱스를 계산합니다.
        let index: u32 = if B::BIT_IS_BYTE {
            offset.checked_add(bitposition).ok_or("Address overflow")?
        } else {
            offset
        };

        if index as usize >= buffer.len() {
            return Err("Out of bounds");
        }
        let byte = unsafe { buffer.get_unchecked(index as usize).get() };

        // SAFETY: the index was checked and writes require exclusive board access.
        unsafe {
            if B::BIT_IS_BYTE {
                *byte = u8::from(value);
            } else {
                let mask = 1_u8 << bitposition;
                let value_mask = 0_u8.wrapping_sub(u8::from(value));
                *byte = (*byte & !mask) | (value_mask & mask);
            }
        }
        Ok(())
    }

    /// Register or replace an IO number. Buffer allocations remain fixed for this board.
    pub fn register_io(
        &mut self,
        io_number: u32,
        direction: Direction,
        offset: u32,
        bitposition: u32,
    ) -> Result<(), &'static str> {
        if bitposition > 7 {
            return Err("Invalid bit position (0 ~ 7)");
        }
        let index = if B::BIT_IS_BYTE {
            offset.checked_add(bitposition).ok_or("Address overflow")?
        } else {
            offset
        };
        let (buffer, entries) = match direction {
            Direction::Input => (&self.inputs, &mut self.input_io),
            Direction::Output => (&self.outputs, &mut self.output_io),
        };
        let cell = buffer.get(index as usize).ok_or("Out of bounds")?;
        let number = io_number as usize;
        let required = number.checked_add(1).ok_or("Invalid IO number")?;
        if required > entries.len() {
            entries
                .try_reserve(required - entries.len())
                .map_err(|_| "IO map allocation failed")?;
            entries.resize_with(required, || None);
        }
        // UnsafeCell permits legacy accesses and registered pointers to share a byte.
        entries[number] = Some(IoEntry {
            byte: NonNull::new(cell.get()).unwrap(),
            mask: 1_u8 << bitposition,
        });
        Ok(())
    }

    #[inline(always)]
    pub fn read_io(&self, io_number: u32, direction: Direction) -> Result<bool, &'static str> {
        let entries = match direction {
            Direction::Input => &self.input_io,
            Direction::Output => &self.output_io,
        };
        let entry = entries
            .get(io_number as usize)
            .and_then(Option::as_ref)
            .ok_or("Unregistered IO number")?;
        // SAFETY: buffers are private, never resized, and owned by this board.
        let byte = unsafe { entry.byte.as_ptr().read() };
        Ok(if B::BIT_IS_BYTE {
            byte != 0
        } else {
            byte & entry.mask != 0
        })
    }

    #[inline(always)]
    pub fn write_io(
        &mut self,
        io_number: u32,
        direction: Direction,
        value: bool,
    ) -> Result<(), &'static str> {
        let entries = match direction {
            Direction::Input => &self.input_io,
            Direction::Output => &self.output_io,
        };
        let entry = entries
            .get(io_number as usize)
            .and_then(Option::as_ref)
            .ok_or("Unregistered IO number")?;
        // SAFETY: fixed buffers stay alive and writes require exclusive board access.
        unsafe {
            let pointer = entry.byte.as_ptr();
            if B::BIT_IS_BYTE {
                pointer.write(u8::from(value));
            } else {
                let value_mask = 0_u8.wrapping_sub(u8::from(value));
                pointer.write((pointer.read() & !entry.mask) | (value_mask & entry.mask));
            }
        }
        Ok(())
    }
}

pub struct NetMemory<B: BoardType> {
    pub boards: Vec<BoardMemory<B>>,
}

impl<B: BoardType> NetMemory<B> {
    pub fn register_io(
        &mut self,
        netid: u32,
        io_number: u32,
        direction: Direction,
        offset: u32,
        bitposition: u32,
    ) -> Result<(), &'static str> {
        self.boards
            .get_mut(netid as usize)
            .ok_or("Invalid netId")?
            .register_io(io_number, direction, offset, bitposition)
    }

    #[inline(always)]
    pub fn read_io(
        &self,
        netid: u32,
        io_number: u32,
        direction: Direction,
    ) -> Result<bool, &'static str> {
        self.boards
            .get(netid as usize)
            .ok_or("Invalid netId")?
            .read_io(io_number, direction)
    }

    #[inline(always)]
    pub fn write_io(
        &mut self,
        netid: u32,
        io_number: u32,
        direction: Direction,
        value: bool,
    ) -> Result<(), &'static str> {
        self.boards
            .get_mut(netid as usize)
            .ok_or("Invalid netId")?
            .write_io(io_number, direction, value)
    }

    pub fn new() -> Self {
        Self { boards: Vec::new() }
    }

    #[inline(always)]
    pub fn read_digital(
        &self,
        netid: u32,
        direction: Direction,
        offset: u32,
        bitposition: u32,
    ) -> Result<bool, &'static str> {
        let board = self.boards.get(netid as usize).ok_or("Invalid netId")?;

        board.read_digital(direction, offset, bitposition)
    }

    #[inline(always)]
    pub fn write_digital(
        &mut self,
        netid: u32,
        direction: Direction,
        offset: u32,
        bitposition: u32,
        value: bool,
    ) -> Result<(), &'static str> {
        let board = self.boards.get_mut(netid as usize).ok_or("Invalid netId")?;

        board.write_digital(direction, offset, bitposition, value)
    }
}
