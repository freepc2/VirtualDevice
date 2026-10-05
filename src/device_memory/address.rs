/// Network identifier.
pub type NetId = u8;

/// Logical I/O number within a network and kind.
pub type IoNumber = u16;

/// Physical module address.
pub type PhysicalAddress = u32;

/// Byte offset within a module.
pub type OffsetAddress = usize;

/// Bit index within a packed word.
pub type BitPosition = u8;

/// Access size in bytes.
pub type SizeBytes = usize;

/// Axis number.
pub type Axis = u8;

/// Controller type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    TwinCAT2,
    TwinCAT3,
    Comizoa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoType {
    DigitalInput,
    DigitalOutput,
    AnalogInput,
    AnalogOutput,
}

impl IoType {
    pub const fn is_digital(self) -> bool {
        matches!(self, Self::DigitalInput | Self::DigitalOutput)
    }

    pub const fn is_analog(self) -> bool {
        matches!(self, Self::AnalogInput | Self::AnalogOutput)
    }

    pub const fn is_output(self) -> bool {
        matches!(self, Self::DigitalOutput | Self::AnalogOutput)
    }
}

/// Memory access width and layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessMode {
    /// Packed 16-bit digital access.
    Bit16,

    /// Direct byte access.
    Byte,

    /// Direct word access.
    Word,

    /// Direct double-word access.
    DWord,

    /// Direct quad-word access.
    QWord,
}

/// Integer type used to interpret raw module data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    U8,
    I8,

    U16,
    I16,

    U32,
    I32,

    U64,
    I64,
}

impl ValueKind {
    pub const fn size_bytes(self) -> SizeBytes {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 => 4,
            Self::U64 | Self::I64 => 8,
        }
    }

    pub const fn is_signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64)
    }

    pub const fn access_mode(self) -> AccessMode {
        match self.size_bytes() {
            1 => AccessMode::Byte,
            2 => AccessMode::Word,
            4 => AccessMode::DWord,
            8 => AccessMode::QWord,
            _ => unreachable!(),
        }
    }
}

/// Physical address and layout for one I/O point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhyAddress {
    /// Network identifier.
    pub net_id: NetId,

    /// Logical channel number.
    pub io_number: IoNumber,

    /// Module start address.
    pub physical_address: PhysicalAddress,

    /// Byte offset within the module.
    pub offset: OffsetAddress,

    /// Bit index for packed digital access.
    pub bit_position: Option<BitPosition>,

    /// Access size in bytes.
    pub size_bytes: SizeBytes,

    /// Memory access mode.
    pub access_mode: AccessMode,

    /// Value interpretation.
    pub value_kind: ValueKind,

    /// I/O kind and direction.
    pub io_type: IoType,
}

/// Physical lookup key without a logical channel number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalIoAddress {
    pub physical_address: PhysicalAddress,
    pub offset: OffsetAddress,
    pub bit_position: Option<BitPosition>,
    pub size_bytes: SizeBytes,
    pub io_type: IoType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    InvalidDigitalLayout,
    InvalidAnalogLayout,
    ChannelOutOfRange { channel: u16, count: u16 },
    OffsetOverflow,
}

/// Comizoa channel counts and analog type.
#[derive(Debug, Clone, Copy)]
pub struct ComizoaLayout {
    pub digital_channels: u16,
    pub analog_channels: u16,
    pub analog_kind: ValueKind,
}

impl PhyAddress {
    /// Create a digital address at an explicit offset.
    pub fn digital(
        net_id: NetId,
        physical_address: PhysicalAddress,
        io_number: IoNumber,
        io_type: IoType,
        offset: OffsetAddress,
        bit_position: Option<BitPosition>,
    ) -> Self {
        Self {
            net_id,
            physical_address,
            io_number,
            io_type,
            offset,
            bit_position,
            size_bytes: if bit_position.is_some() { 2 } else { 1 },
            access_mode: if bit_position.is_some() {
                AccessMode::Bit16
            } else {
                AccessMode::Byte
            },
            value_kind: ValueKind::U8,
        }
    }

    /// Create an analog address at an explicit offset.
    pub fn analog(
        net_id: NetId,
        physical_address: PhysicalAddress,
        io_number: IoNumber,
        io_type: IoType,
        offset: OffsetAddress,
        value_kind: ValueKind,
    ) -> Self {
        Self {
            net_id,
            physical_address,
            io_number,
            io_type,
            offset,
            value_kind,
            bit_position: None,
            size_bytes: value_kind.size_bytes(),
            access_mode: value_kind.access_mode(),
        }
    }

    /// Calculate a Comizoa address from its channel layout.
    pub fn from_comizoa(
        net_id: NetId,
        physical_address: PhysicalAddress,
        io_number: IoNumber,
        io_type: IoType,
        channel: u16,
        layout: ComizoaLayout,
    ) -> Result<Self, AddressError> {
        let count = if io_type.is_digital() {
            layout.digital_channels
        } else {
            layout.analog_channels
        };
        if channel >= count {
            return Err(AddressError::ChannelOutOfRange { channel, count });
        }
        if io_type.is_digital() {
            Ok(Self::digital(
                net_id,
                physical_address,
                io_number,
                io_type,
                usize::from(channel / 16) * 2,
                Some((channel % 16) as u8),
            ))
        } else {
            let analog_base = usize::from(layout.digital_channels).div_ceil(16) * 2;
            let offset = analog_base
                .checked_add(usize::from(channel) * layout.analog_kind.size_bytes())
                .ok_or(AddressError::OffsetOverflow)?;
            Ok(Self::analog(
                net_id,
                physical_address,
                io_number,
                io_type,
                offset,
                layout.analog_kind,
            ))
        }
    }

    pub fn validate(&self) -> Result<(), AddressError> {
        if self.io_type.is_digital() {
            let valid = self.value_kind == ValueKind::U8
                && match self.access_mode {
                    AccessMode::Bit16 => {
                        self.size_bytes == 2 && self.bit_position.is_some_and(|bit| bit < 16)
                    }
                    AccessMode::Byte => self.size_bytes == 1 && self.bit_position.is_none(),
                    _ => false,
                };
            if !valid {
                return Err(AddressError::InvalidDigitalLayout);
            }
        } else if self.bit_position.is_some()
            || self.size_bytes != self.value_kind.size_bytes()
            || self.access_mode != self.value_kind.access_mode()
        {
            return Err(AddressError::InvalidAnalogLayout);
        }
        self.offset
            .checked_add(self.size_bytes)
            .ok_or(AddressError::OffsetOverflow)?;
        Ok(())
    }

    pub const fn physical_key(&self) -> PhysicalIoAddress {
        PhysicalIoAddress {
            physical_address: self.physical_address,
            offset: self.offset,
            bit_position: self.bit_position,
            size_bytes: self.size_bytes,
            io_type: self.io_type,
        }
    }
    pub const fn is_digital(&self) -> bool {
        self.io_type.is_digital()
    }
    pub const fn is_analog(&self) -> bool {
        self.io_type.is_analog()
    }
    pub const fn is_output(&self) -> bool {
        self.io_type.is_output()
    }
    pub const fn is_bit_access(&self) -> bool {
        matches!(self.access_mode, AccessMode::Bit16)
    }
    pub const fn is_byte_access(&self) -> bool {
        matches!(self.access_mode, AccessMode::Byte)
    }
    pub fn bit_mask(&self) -> Option<u16> {
        self.bit_position
            .and_then(|bit| 1u16.checked_shl(u32::from(bit)))
    }
}
