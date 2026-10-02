use std::collections::HashMap;
use super::memory::{NetId, Axis, NetworkMemory};

/// 모터 축 상태
#[derive(Debug, Default, Clone)]
pub struct MotorState {
    pub target_position: f64,
    pub actual_position: f64,
    pub velocity: f64,
    pub servo_on: bool,
    pub homed: bool,
    pub moving: bool,
    pub alarm_code: u16,
}

/// 장비의 단일 메모리 시스템 (Device Memory)
#[derive(Debug, Default)]
pub struct DeviceMemory {
    pub networks: HashMap<NetId, NetworkMemory>,
    pub motors: HashMap<(NetId, Axis), MotorState>,
}

impl DeviceMemory {
    pub fn new() -> Self {
        Self {
            networks: HashMap::new(),
            motors: HashMap::new(),
        }
    }
}

/// 장비 ID별 최상위 서버 (Local Server)
#[derive(Debug, Default)]
pub struct LocalServer {
    pub device_id: String,
    pub memory: DeviceMemory,
}

impl LocalServer {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            memory: DeviceMemory::new(),
        }
    }
}
