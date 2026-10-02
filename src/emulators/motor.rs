use crate::device_memory::{LegacyDeviceMemory as DeviceMemory, PhyAddress};

/// 모터 에뮬레이터 구조체
pub struct MotorEmulator {
    pub axis_id: u16,
    pub speed: u16,
}

impl MotorEmulator {
    pub fn new(axis_id: u16) -> Self {
        Self { axis_id, speed: 0 }
    }

    /// 모터 에뮬레이터 틱(Tick) 처리 로직
    pub fn tick(&mut self, memory: &mut DeviceMemory, target_addr: &PhyAddress) {
        if let Some(val) = memory.read(target_addr) {
            self.speed = val;
            // 예: 모터 연산 로직 수행
        }
    }
}
