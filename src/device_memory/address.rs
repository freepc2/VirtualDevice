/// 데이터 타입 정의 (TwinCAT / Comizoa 공통 지원)
///
/// 산업용 장비 메모리 및 I/O에서 다루는 데이터의 규격을 나타냅니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    /// 디지털 비트 (1 bit)
    Bit,
    /// 바이트 단위 (8 bit)
    Byte,
    /// 워드 단위 (16 bit)
    Word,
    /// 더블 워드 단위 (32 bit)
    DWord,
    /// 32비트 부동소수점 (Float)
    Float32,
    /// 사용자 정의 크기
    Custom(u16),
}

/// 물리 주소 (Physical Address) - 코어 도메인용
///
/// TwinCAT, Comizoa 등 산업용 컨트롤러 및 I/O 장비의 메모리/포인트 주소를
/// 고유하게 식별하기 위한 통합 물리 주소 구조체입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhyAddress {
    /// 장비 보드 ID 또는 PLC 스테이션 번호 (0 ~ 255)
    pub board_id: u8,

    /// 모터 축 번호 또는 채널/슬롯 번호 (0 ~ 65,535)
    pub channel: u16,

    /// 베이스 메모리 주소 (예: 0x1000, 0x2000 등 숫자형 주소)
    pub address: u32,

    /// 워드/바이트 상대 오프셋 (Base Address 기준 Offset)
    pub offset: u16,

    /// 디지털 I/O 등의 비트 위치 (0 ~ 15). 아날로그 데이터의 경우 None
    pub bit: Option<u8>,

    /// 데이터 규격 타입 (DataType)
    pub data_type: DataType,
}

impl PhyAddress {
    /// Comizoa 제어기 기반 주소 자동 생성 팩토리 (Factory Method)
    ///
    /// Comizoa 보드의 특성에 따라 베이스 주소(`address`)와 채널 번호(`channel`)를 입력받아,
    /// 내부 채널 규칙에 맞춰 `offset`과 `bit`를 자동으로 계산하여 `PhyAddress`를 생성합니다.
    pub fn from_comizoa(board_id: u8, address: u32, channel: u16) -> Self {
        let calc_offset = channel / 16;
        let calc_bit = (channel % 16) as u8;

        Self {
            board_id,
            channel,
            address,
            offset: calc_offset,
            bit: Some(calc_bit),
            data_type: DataType::Bit,
        }
    }

    /// TwinCAT (ADS / PLC) 기반 주소 매핑 팩토리 (Factory Method)
    ///
    /// TwinCAT 프로젝트의 심볼릭 주소나 명시적 프로세스 이미지 매핑 정보로부터
    /// `board_id`, `address`, `offset`, `bit`, `data_type`을 직접 지정하여 생성합니다.
    pub fn from_twincat(
        board_id: u8,
        address: u32,
        offset: u16,
        bit: Option<u8>,
        data_type: DataType,
    ) -> Self {
        Self {
            board_id,
            channel: 0,
            address,
            offset,
            bit,
            data_type,
        }
    }
}
