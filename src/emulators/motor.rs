use crate::device_memory::{IoValue, LocalServer, PhyAddress};

/// Motor emulator state.
pub struct MotorEmulator {
    pub axis_id: u16,
    pub speed: u16,
}

impl MotorEmulator {
    pub fn new(axis_id: u16) -> Self {
        Self { axis_id, speed: 0 }
    }

    /// Update the motor from its target address.
    pub fn tick(&mut self, memory: &LocalServer, target_addr: &PhyAddress) {
        if let Some(IoValue::U16(val)) = memory.read_phy(target_addr) {
            self.speed = val;
            // Add motor behavior here.
        }
    }
}
