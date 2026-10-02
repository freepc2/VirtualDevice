use std::collections::HashMap;
use super::address::PhyAddress;

pub type NetId = u8;
pub type IoNumber = u16;
pub type ModuleId = u8;
pub type Axis = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    U16,
    I16,
    U32,
    I32,
    Float32,
}

/// 디지털 및 아날로그 I/O 포인트를 통합한 열거형
#[derive(Debug, Clone)]
pub enum IoPoint {
    Digital {
        module_id: ModuleId,
        is_output: bool,
        offset: usize,
        bit_mask: Option<u8>,
    },
    Analog {
        module_id: ModuleId,
        is_output: bool,
        offset: usize,
        size_bytes: usize,
        value_kind: ValueKind,
    },
}

/// 물리 주소 직접 접근 맵 (보조 경로)
#[derive(Debug, Clone)]
pub struct ImageRef {
    pub module_id: ModuleId,
    pub offset: usize,
    pub bit_mask: Option<u8>,
}

/// 모듈별 바이트 이미지 저장소
#[derive(Debug, Clone, Default)]
pub struct ModuleMemory {
    pub input_image: Vec<u8>,
    pub output_image: Vec<u8>,
}

/// 단일 네트워크(NetId)의 메모리 관리 영역
#[derive(Debug, Default)]
pub struct NetworkMemory {
    pub net_id: NetId,
    pub dio_points: Vec<Option<IoPoint>>,
    pub aio_points: Vec<Option<IoPoint>>,
    pub modules: HashMap<ModuleId, ModuleMemory>,
    pub physical_map: HashMap<PhyAddress, ImageRef>,
}

/// 레거시 주소 기반 메모리 (기존 벤치마크 및 테스트 호환용)
#[derive(Debug, Default)]
pub struct LegacyDeviceMemory {
    pub storage: HashMap<PhyAddress, u16>,
}

impl LegacyDeviceMemory {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn write(&mut self, phy_addr: PhyAddress, value: u16) {
        self.storage.insert(phy_addr, value);
    }

    pub fn write_silent(&mut self, phy_addr: PhyAddress, value: u16) {
        self.storage.insert(phy_addr, value);
    }

    pub fn read(&self, phy_addr: &PhyAddress) -> Option<u16> {
        self.storage.get(phy_addr).copied()
    }
}
