use std::collections::HashMap;
use super::address::PhyAddress;

/// 디바이스 메모리 맵 (Device Memory Map)
///
/// `PhyAddress`를 키(Key)로 하고 16비트 레지스터 값(`u16`)을 값(Value)으로 가지는
/// 인메모리(In-Memory) 가상 디바이스 저장소입니다.
#[derive(Debug, Default)]
pub struct DeviceMemory {
    pub storage: HashMap<PhyAddress, u16>,
}

impl DeviceMemory {
    /// 새로운 빈 디바이스 메모리 인스턴스를 생성합니다.
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    /// 특정 물리 주소(`PhyAddress`)에 16비트 값을 기록합니다. (로그 출력 포함)
    pub fn write(&mut self, phy_addr: PhyAddress, value: u16) {
        println!(
            "[Memory Write] Board: {}, Addr: 0x{:04X}, Offset: {}, Bit: {:?}, Value: {}",
            phy_addr.board_id, phy_addr.address, phy_addr.offset, phy_addr.bit, value
        );
        self.storage.insert(phy_addr, value);
    }

    /// 로그 출력 없이 순수하게 메모리에 값을 기록합니다. (고속 벤치마크/대량 연산용)
    pub fn write_silent(&mut self, phy_addr: PhyAddress, value: u16) {
        self.storage.insert(phy_addr, value);
    }

    /// 특정 물리 주소(`PhyAddress`)의 값을 읽어옵니다. 값이 존재하지 않으면 `None`을 반환합니다.
    pub fn read(&self, phy_addr: &PhyAddress) -> Option<u16> {
        self.storage.get(phy_addr).copied()
    }
}
