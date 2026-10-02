# 가상 디바이스 메모리 및 I/O 아키텍처 상세 설계서

본 문서는 장비 에뮬레이터 시스템의 메모리 관리, I/O 매핑, MDB 로딩 및 요청 처리 구조에 대한 상세 설계 사양을 정의합니다. (현행 `HashMap<PhyAddress, u16>` 구조에서 바이트 이미지 기반 구조로의 고도화를 위한 설계입니다.)

---

## 1. 전체 구조 (Overall Architecture)

장비별 클라이언트 API는 다양하지만, `LocalServer`가 사용하는 값의 원본은 하나입니다. 서버는 장비 ID를 확인하고, NetId와 번호를 해당 네트워크의 바이트 이미지 위치로 바꿉니다.

```
[ Comizoa / TwinCAT 클라이언트 ] 
       │
       ▼
[ 프로토콜·장비별 어댑터 ]
       │
       ▼
[ 장비 ID 검증 / NetId 결정 ]
       │
       ▼
[ LocalServer (DIO / AIO / Motor) ]
```

### 구성 요소별 역할
- **서버 (Device ID)**: 
  - `LocalServer` 인스턴스 하나가 장비 ID 하나를 담당합니다. 다른 ID의 요청에는 성공이 아닌 오류 응답을 보냅니다.
- **I/O (NetId)**: 
  - 한 장비 안에 여러 NetId가 있을 수 있습니다. NetId마다 DIO/AIO 번호 배열과 모듈 이미지를 가집니다.
- **Motor (축)**: 
  - Motor는 바이트 I/O 값으로 모델링하지 않고 별도 상태로 관리합니다. `NetId + Axis`로 축을 찾습니다.

---

## 2. 식별자와 조회 키 (Identifiers and Query Keys)

| 대상 | 조회 키 | 의미 |
| :--- | :--- | :--- |
| **LocalServer** | `DeviceId` | 요청 입구에서 검사합니다. 서버 내부 I/O 배열 키에는 반복해서 넣지 않습니다. |
| **DIO** | `(NetId, DIO, IONumber)` | 해당 네트워크의 Digital 신호 위치를 찾습니다. |
| **AIO** | `(NetId, AIO, IONumber)` | 해당 네트워크의 Analog 신호 위치를 찾습니다. |
| **물리 주소 접근** | `(NetId, PhyAddress)` | 필요한 호출에만 제공하며 위와 같은 이미지 위치를 가리킵니다. |
| **Motor** | `(NetId, Axis)` | 축별 상태와 명령을 찾습니다. |

### 주요 규칙
- **독립된 번호 공간**: DIO와 AIO는 각각 독립된 번호 공간입니다. NetId가 다르면 같은 IONumber를 다시 쓸 수 있습니다. DI/DO, AI/AO는 번호 공간을 추가로 나누지 않고 등록된 방향 속성으로 검증합니다.
- **NetId 누락 시**: 기존 호출에 NetId가 없다면 연결 정보 또는 설정에서 NetId가 유일하게 정해질 때만 요청을 처리합니다. 여러 NetId가 가능하면 임의의 네트워크를 선택하지 않고 모호한 요청 오류(Ambiguous Request Error)를 반환합니다.

---

## 3. 실제 메모리 배치 (Actual Memory Layout)

번호 배열에는 값이 아닌 **위치 정보(포인터/참조)**만 저장됩니다. 실제 값의 단일 원본(Single Source of Truth)은 NetId·모듈·입출력별 `Vec<u8>` 바이트 이미지입니다.

```rust
LocalServer {
    device_id: String,
    networks: HashMap<NetId, NetworkMemory>,
}

NetworkMemory {
    net_id: NetId,
    dio_points: Vec<Option<DioPoint>>,     // max_ionumber + 1 크기
    dio_modules: Vec<Vec<u8>>,             // 모듈별 입력/출력 바이트 이미지
    aio_points: Vec<Option<AioPoint>>,     // max_ionumber + 1 크기
    aio_modules: Vec<Vec<u8>>,             // 모듈별 입력/출력 바이트 이미지
    motors: HashMap<Axis, MotorState>,
    physical_map: HashMap<PhyAddress, ImageRef>,
}
```

- **번호 배열**: 각 NetId의 DIO와 AIO에 대해 `max_ionumber + 1` 크기로 생성합니다. 빈 번호는 `None`입니다. 번호가 등성등성하면 메타데이터 용량이 증가할 수 있으므로 상한을 확인합니다.
- **NetId 조회**: 요청마다 NetId를 한 번 찾아야 합니다. NetId가 희소(Sparse)하면 바깥 단계에 작은 HashMap을 사용할 수 있습니다.

---

## 4. 시작 시 MDB 로딩 (MDB Loading at Startup)

에뮬레이터 구동 시 MDB(Memory Database) 데이터를 로딩하는 5단계 절차입니다.

1. **장비 필터**: 서버의 장비 ID에 해당하는 I/O 행을 읽고 NetId별로 묶습니다.
2. **설정 검증**: 필수 필드, 중복(`NetId, DIO/AIO, IONumber`), 방향, 크기, 모듈과 오프셋의 범위를 검사합니다.
3. **모듈 이미지 할당**: NetId·모듈·입출력마다 필요한 길이 `max(offset + 접근 길이)`를 계산해 바이트 이미지(`Vec<u8>`)를 할당합니다.
4. **번호 위치 등록**: DIO/AIO의 각 `points[IONumber]`에 이미지 위치와 비트 마스크 또는 Analog 형식을 기록합니다.
5. **보조 경로 등록**: 필요한 경우 `(NetId, PhyAddress)`를 같은 이미지 위치에 연결합니다. Motor 설정은 공급원을 확인한 뒤 축을 별도로 등록합니다.

---

## 5. DIO: 바이트 또는 비트 (DIO: Byte or Bit)

Digital 저장 방식은 장비 이름만으로 전역 결정하지 않고 MDB의 모듈 매핑으로 S/W 시작 시 확정됩니다.

- **TwinCAT 3 대상 매핑**: 바이트 오프셋 (`image[offset] != 0` / `image[offset] = value`)
- **Comizoa / TwinCAT 2 대상 매핑**: 바이트 오프셋 + 비트 마스크 (`image[offset] & mask != 0` / 비트 변경 연산)

---

## 6. AIO: 길이와 자료형 (AIO: Length and Data Type)

AIO 위치에는 `module, input/output, byte_offset, size_bytes, value_kind`를 기록합니다. 크기가 같아도 정수와 부동소수점으로 다르게 해석합니다.

- **16비트 값 (`size=2`)**: 16비트 정수 (부호 유무 및 바이트 순서 등록)
- **32비트 정수 (`size=4`)**: `u32` 또는 `i32`
- **32비트 Float (`f32`)**: 4바이트의 비트 패턴을 부동소수점으로 변환 (`from_le_bytes` / `from_be_bytes`)

---

## 7. Motor: I/O 메모리와 분리 (Motor: Separation of I/O Memory)

Motor는 `(NetId, Axis)`로 관리합니다. 축 상태에는 지령 위치, 실제 위치, 속도, Servo 상태, Home 상태, 이동 상태, 알람을 둡니다.

- **명령 흐름**: `Servo / Home / Move / Jog / Stop 명령` ──> `MotorControl (NetId, Axis)` ──> `축 상태 스냅샷 (진행 / 완료 / 오류)`
- **주의**: 이동 명령을 접수했다는 응답은 이동 완료를 뜻하지 않습니다. UI는 상태 스냅샷의 위치·완료·알람을 확인합니다.

---

## 8. 요청 처리 예시와 오류 (Request Processing Examples and Errors)

### 처리 흐름 예시 (DIO 읽기)
`DeviceId 검사` ──> `NetId 찾기` ──> `DIO[IONumber] 위치 조회` ──> `모듈 입력/출력 이미지에서 바이트 또는 비트 읽기` ──> `값 반환`

### 주요 오류 상황 및 대응
- **다른 DeviceId**: 메모리에 접근하지 않고 대상 장비 불일치 오류 반환
- **미등록 NetId 또는 IONumber**: `UnknownNetId` 또는 `UnknownIONumber`로 구분
- **NetId가 빠진 기존 호출**: 유일한 NetId를 확정할 수 없으면 `AmbiguousNetId`로 처리
- **입력 쓰기 / 형식 불일치**: 값을 변경하지 않고 접근 권한 또는 형식 오류 반환
- **실장비 통신 실패**: 값 0과 구별되는 오류를 반환하고 캐시 갱신 여부를 정책대로 처리

---

## 9. 성능, 메모리, 동시성

- **빠른 경로 (Fast Path)**: NetId를 한 번 선택한 뒤 `points[IONumber]`로 위치를 찾고 바이트 이미지에 접근합니다. 반복 내부 작업은 확인된 위치를 재사용할 수 있습니다.
- **용량**: 바이트 이미지는 모듈에 필요한 길이만큼만 할당합니다. 번호 배열은 NetId별 최대 IONumber에 좌우되므로 MDB의 번호 분포와 메모리 상한을 확인합니다.
- **동시성**: 여러 스레드가 같은 바이트를 동시에 바꾸면 갱신이 유실될 수 있습니다. 메모리 권한을 한 실행 흐름에 모으거나 모듈 단위 잠금 등 동시성 규칙이 필요합니다.

---

## 10. 구현 순서와 미확정 사항

### 구현 순서
1. MDB의 NetId, IONumber, 모듈, 입출력 방향, Digital 저장 단위, Analog 크기·형식·바이트 순서를 확정합니다.
2. NetId별 번호 배열과 모듈별 입력·출력 이미지를 만들고 중복·범위를 검증합니다.
3. 장비 ID 검사 및 NetId 기반 DIO/AIO 읽기·쓰기를 연결합니다.
4. 필요한 경우 `(NetId, PhyAddress)` 조회 경로를 같은 이미지 위치에 연결합니다.
5. Motor 설정 공급원을 확인하고 `NetId + Axis` 제어·상태를 구현합니다.
6. 실제 MDB와 클라이언트 요청으로 지연 및 메모리 사용량을 측정합니다.
