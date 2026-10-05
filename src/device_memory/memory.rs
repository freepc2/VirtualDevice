use std::collections::HashMap;

use super::address::{
    AccessMode, AddressError, IoNumber, IoType, NetId, PhyAddress, PhysicalIoAddress, ValueKind,
};

/// Value stored in virtual memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IoValue {
    Bool(bool),

    U8(u8),
    I8(i8),

    U16(u16),
    I16(i16),

    U32(u32),
    I32(i32),

    U64(u64),
    I64(i64),
}

impl IoValue {
    pub const fn value_kind(self) -> Option<ValueKind> {
        match self {
            Self::Bool(_) => None,

            Self::U8(_) => Some(ValueKind::U8),
            Self::I8(_) => Some(ValueKind::I8),

            Self::U16(_) => Some(ValueKind::U16),
            Self::I16(_) => Some(ValueKind::I16),

            Self::U32(_) => Some(ValueKind::U32),
            Self::I32(_) => Some(ValueKind::I32),

            Self::U64(_) => Some(ValueKind::U64),
            Self::I64(_) => Some(ValueKind::I64),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct IoPoint {
    address: PhyAddress,
    value: IoValue,
}

#[derive(Debug, Clone, Copy)]
struct OccupiedByte {
    mask: u8,
    bit_access: bool,
    io_number: IoNumber,
    bit_owners: [Option<IoNumber>; 8],
}

/// Values use direction-specific arrays shared by logical and physical access.
#[derive(Debug, Default)]
pub struct NetworkMemory {
    net_id: NetId,
    digital_inputs: Vec<Option<IoPoint>>,
    digital_outputs: Vec<Option<IoPoint>>,
    analog_inputs: Vec<Option<IoPoint>>,
    analog_outputs: Vec<Option<IoPoint>>,
    physical_index: HashMap<PhysicalIoAddress, IoNumber>,
    // Used only to check registration overlaps.
    occupied: HashMap<(u32, bool, usize), OccupiedByte>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryError {
    NetworkNotFound(NetId),
    IoNotFound(IoNumber),
    PhysicalAddressNotFound,
    NetworkMismatch {
        expected: NetId,
        actual: NetId,
    },
    DuplicateIoNumber {
        io_number: IoNumber,
        digital: bool,
    },
    PhysicalOverlap {
        io_number: IoNumber,
        existing_io_number: IoNumber,
        offset: usize,
    },
    DirectionMismatch {
        expected: IoType,
        actual: IoType,
    },
    InvalidAddress(AddressError),
    ValueTypeMismatch {
        expected: ValueKind,
        actual: Option<ValueKind>,
    },
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MemoryError {}

impl NetworkMemory {
    pub fn new(net_id: NetId) -> Self {
        Self {
            net_id,
            ..Self::default()
        }
    }
    pub fn net_id(&self) -> NetId {
        self.net_id
    }
    pub fn digital_input_count(&self) -> usize {
        self.digital_inputs.iter().flatten().count()
    }
    pub fn digital_output_count(&self) -> usize {
        self.digital_outputs.iter().flatten().count()
    }
    pub fn analog_input_count(&self) -> usize {
        self.analog_inputs.iter().flatten().count()
    }
    pub fn analog_output_count(&self) -> usize {
        self.analog_outputs.iter().flatten().count()
    }
    pub fn digital_count(&self) -> usize {
        self.digital_input_count() + self.digital_output_count()
    }
    pub fn analog_count(&self) -> usize {
        self.analog_input_count() + self.analog_output_count()
    }
    pub fn addresses(&self) -> impl Iterator<Item = &PhyAddress> {
        self.digital_inputs
            .iter()
            .chain(&self.digital_outputs)
            .chain(&self.analog_inputs)
            .chain(&self.analog_outputs)
            .flatten()
            .map(|point| &point.address)
    }

    fn points(&self, io_type: IoType) -> &[Option<IoPoint>] {
        match io_type {
            IoType::DigitalInput => &self.digital_inputs,
            IoType::DigitalOutput => &self.digital_outputs,
            IoType::AnalogInput => &self.analog_inputs,
            IoType::AnalogOutput => &self.analog_outputs,
        }
    }
    fn points_mut(&mut self, io_type: IoType) -> &mut Vec<Option<IoPoint>> {
        match io_type {
            IoType::DigitalInput => &mut self.digital_inputs,
            IoType::DigitalOutput => &mut self.digital_outputs,
            IoType::AnalogInput => &mut self.analog_inputs,
            IoType::AnalogOutput => &mut self.analog_outputs,
        }
    }
    fn point(&self, io_type: IoType, io_number: IoNumber) -> Option<&IoPoint> {
        self.points(io_type).get(usize::from(io_number))?.as_ref()
    }
    fn duplicate_in_namespace(&self, address: &PhyAddress) -> bool {
        let counterpart = match address.io_type {
            IoType::DigitalInput => IoType::DigitalOutput,
            IoType::DigitalOutput => IoType::DigitalInput,
            IoType::AnalogInput => IoType::AnalogOutput,
            IoType::AnalogOutput => IoType::AnalogInput,
        };
        self.point(address.io_type, address.io_number).is_some()
            || self.point(counterpart, address.io_number).is_some()
    }

    /// Validate before changing memory state.
    pub fn register(&mut self, address: PhyAddress) -> Result<(), MemoryError> {
        address.validate().map_err(MemoryError::InvalidAddress)?;
        if address.net_id != self.net_id {
            return Err(MemoryError::NetworkMismatch {
                expected: self.net_id,
                actual: address.net_id,
            });
        }
        if self.duplicate_in_namespace(&address) {
            return Err(MemoryError::DuplicateIoNumber {
                io_number: address.io_number,
                digital: address.is_digital(),
            });
        }

        let mut occupancy = Vec::with_capacity(address.size_bytes);
        for byte in 0..address.size_bytes {
            let key = (
                address.physical_address,
                address.is_output(),
                address.offset + byte,
            );
            let bit_access = address.access_mode == AccessMode::Bit16;
            let mask = if bit_access {
                let bit = address
                    .bit_position
                    .expect("validated bit access has a bit position");
                if byte == usize::from(bit / 8) {
                    1u8 << (bit % 8)
                } else {
                    0
                }
            } else {
                u8::MAX
            };
            if let Some(existing) = self.occupied.get(&key) {
                if !bit_access || !existing.bit_access || existing.mask & mask != 0 {
                    let existing_io_number = (0..8)
                        .find(|bit| existing.mask & mask & (1 << bit) != 0)
                        .and_then(|bit| existing.bit_owners[bit])
                        .unwrap_or(existing.io_number);
                    return Err(MemoryError::PhysicalOverlap {
                        io_number: address.io_number,
                        existing_io_number,
                        offset: key.2,
                    });
                }
            }
            occupancy.push((
                key,
                OccupiedByte {
                    mask,
                    bit_access,
                    io_number: address.io_number,
                    bit_owners: std::array::from_fn(|bit| {
                        if mask & (1 << bit) != 0 {
                            Some(address.io_number)
                        } else {
                            None
                        }
                    }),
                },
            ));
        }

        let value = if address.is_digital() {
            IoValue::Bool(false)
        } else {
            match address.value_kind {
                ValueKind::U8 => IoValue::U8(0),
                ValueKind::I8 => IoValue::I8(0),
                ValueKind::U16 => IoValue::U16(0),
                ValueKind::I16 => IoValue::I16(0),
                ValueKind::U32 => IoValue::U32(0),
                ValueKind::I32 => IoValue::I32(0),
                ValueKind::U64 => IoValue::U64(0),
                ValueKind::I64 => IoValue::I64(0),
            }
        };
        let points = self.points_mut(address.io_type);
        points.resize_with(points.len().max(usize::from(address.io_number) + 1), || {
            None
        });
        points[usize::from(address.io_number)] = Some(IoPoint { address, value });
        self.physical_index
            .insert(address.physical_key(), address.io_number);
        for (key, new) in occupancy {
            self.occupied
                .entry(key)
                .and_modify(|old| {
                    old.mask |= new.mask;
                    for bit in 0..8 {
                        if new.bit_owners[bit].is_some() {
                            old.bit_owners[bit] = new.bit_owners[bit];
                        }
                    }
                })
                .or_insert(new);
        }
        Ok(())
    }

    /// Stop at the first error; keep prior registrations.
    pub fn register_all<I: IntoIterator<Item = PhyAddress>>(
        &mut self,
        addresses: I,
    ) -> Result<(), MemoryError> {
        for address in addresses {
            self.register(address)?;
        }
        Ok(())
    }
    pub fn resolve_digital(&self, number: IoNumber) -> Option<&PhyAddress> {
        self.resolve_io_kind(number, true)
    }
    pub fn resolve_analog(&self, number: IoNumber) -> Option<&PhyAddress> {
        self.resolve_io_kind(number, false)
    }
    fn resolve_io_kind(&self, number: IoNumber, digital: bool) -> Option<&PhyAddress> {
        let (input, output) = if digital {
            (IoType::DigitalInput, IoType::DigitalOutput)
        } else {
            (IoType::AnalogInput, IoType::AnalogOutput)
        };
        self.point(input, number)
            .or_else(|| self.point(output, number))
            .map(|point| &point.address)
    }
    pub fn read_digital(&self, number: IoNumber) -> Option<IoValue> {
        self.read_io_kind(number, true)
    }
    pub fn read_analog(&self, number: IoNumber) -> Option<IoValue> {
        self.read_io_kind(number, false)
    }
    fn read_io_kind(&self, number: IoNumber, digital: bool) -> Option<IoValue> {
        let (input, output) = if digital {
            (IoType::DigitalInput, IoType::DigitalOutput)
        } else {
            (IoType::AnalogInput, IoType::AnalogOutput)
        };
        self.point(input, number)
            .or_else(|| self.point(output, number))
            .map(|point| point.value)
    }
    pub fn read_io(&self, io_type: IoType, number: IoNumber) -> Option<IoValue> {
        let point = self.point(io_type, number)?;
        (point.address.io_type == io_type).then_some(point.value)
    }
    fn set_value(point: &mut IoPoint, value: IoValue) -> Result<(), MemoryError> {
        let valid = if point.address.is_digital() {
            matches!(value, IoValue::Bool(_))
        } else {
            value.value_kind() == Some(point.address.value_kind)
        };
        if !valid {
            return Err(MemoryError::ValueTypeMismatch {
                expected: point.address.value_kind,
                actual: value.value_kind(),
            });
        }
        point.value = value;
        Ok(())
    }
    fn write_io_kind(
        &mut self,
        number: IoNumber,
        digital: bool,
        value: IoValue,
    ) -> Result<(), MemoryError> {
        let (inputs, outputs) = if digital {
            (&mut self.digital_inputs, &mut self.digital_outputs)
        } else {
            (&mut self.analog_inputs, &mut self.analog_outputs)
        };
        let index = usize::from(number);
        let point = inputs
            .get_mut(index)
            .and_then(Option::as_mut)
            .or_else(|| outputs.get_mut(index).and_then(Option::as_mut))
            .ok_or(MemoryError::IoNotFound(number))?;
        Self::set_value(point, value)
    }
    pub fn write_digital(&mut self, number: IoNumber, value: bool) -> Result<(), MemoryError> {
        self.write_io_kind(number, true, IoValue::Bool(value))
    }
    pub fn write_analog(&mut self, number: IoNumber, value: IoValue) -> Result<(), MemoryError> {
        self.write_io_kind(number, false, value)
    }
    pub fn write_io(
        &mut self,
        io_type: IoType,
        number: IoNumber,
        value: IoValue,
    ) -> Result<(), MemoryError> {
        // Update the requested slot first.
        if let Some(point) = self
            .points_mut(io_type)
            .get_mut(usize::from(number))
            .and_then(Option::as_mut)
        {
            return Self::set_value(point, value);
        }
        // Check the opposite direction only when the slot is missing.
        let opposite = match io_type {
            IoType::DigitalInput => IoType::DigitalOutput,
            IoType::DigitalOutput => IoType::DigitalInput,
            IoType::AnalogInput => IoType::AnalogOutput,
            IoType::AnalogOutput => IoType::AnalogInput,
        };
        if self.point(opposite, number).is_some() {
            Err(MemoryError::DirectionMismatch {
                expected: opposite,
                actual: io_type,
            })
        } else {
            Err(MemoryError::IoNotFound(number))
        }
    }
    pub fn read_physical(&self, key: &PhysicalIoAddress) -> Option<IoValue> {
        self.read_io(key.io_type, *self.physical_index.get(key)?)
    }
    pub fn write_physical(
        &mut self,
        key: &PhysicalIoAddress,
        value: IoValue,
    ) -> Result<(), MemoryError> {
        let number = *self
            .physical_index
            .get(key)
            .ok_or(MemoryError::PhysicalAddressNotFound)?;
        self.write_io(key.io_type, number, value)
    }
    pub fn read_phy(&self, address: &PhyAddress) -> Option<IoValue> {
        self.read_physical(&address.physical_key())
    }
    pub fn write_phy(&mut self, address: &PhyAddress, value: IoValue) -> Result<(), MemoryError> {
        self.write_physical(&address.physical_key(), value)
    }
}
