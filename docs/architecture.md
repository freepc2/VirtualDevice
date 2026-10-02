# 가상 디바이스 에뮬레이터 (Virtual Device Emulator) 설계 및 개발 로드맵

본 문서는 러스트(Rust) 기반으로 구현되는 가상 디바이스 에뮬레이터 시스템의 아키텍처 설계와 러스트 초보 개발자를 위한 단계별 구현 로드맵을 정의합니다.

---

## 1. 프로젝트 개요 및 설계 목표

- **개발 언어**: Rust
- **핵심 목표**:
  - 단일 PC 환경에서 복잡도를 낮추고 데이터 정합성을 유지하기 위해 싱글 스레드 기반의 비동기 이벤트 루프(`Tokio current_thread`) 채택
  - 서버와 클라이언트(에뮬레이터)가 동일한 코어 엔진을 공유하며, NATS 메시징 브로커를 통해 양방향으로 유연하게 통신
  - 단독 구동(Standalone) 및 다중 장비 통합 연동(Cluster Mode) 지원
  - 파일 I/O 및 네트워크 통신을 순수 도메인 로직(Core)으로부터 분리하여 테스트 용이성 및 유지보수성 극대화

---

## 2. 시스템 아키텍처 및 모듈 구조

전체 프로젝트는 역할과 책임에 따라 모듈 및 크레이트(Cargo Workspace)로 분리합니다.

```
D:\Dev\VirtualDevice\
├── docs/                      # 설계 및 문서화 폴더
│   └── architecture.md        # 본 문서
├── src/                       # 소스코드 폴더 (단일 프로젝트 또는 Cargo Workspace)
└── README.md
```

### 모듈 구성안
1. **Core (`core`)**:
   - **역할**: 순수 도메인 로직, 메모리 맵(Address, Offset, Mapper), 에뮬레이션 계산 규칙(A2D, P2A, Random Noise 등). 상세한 메모리 배치 및 I/O 매핑 설계는 [Device Memory Architecture Design](./device_memory_design.md)을 참조하세요.
   - **특징**: 파일 I/O(`std::fs`, `tokio::fs`) 및 네트워크(`NATS`) 의존성 없음 (순수 러스트 구조체 및 연산).
2. **Config Loader (`config-loader`)**:
   - **역할**: YAML / CSV 설정 파일을 읽어 Core가 요구하는 구조체로 파싱 및 검증.
3. **Network (`network`)**:
   - **역할**: NATS 브로커 연동, 메시지 송수신, 우선순위 큐(`BinaryHeap`) 관리.
4. **Emulators (`emulators` / binaries)**:
   - **역할**: `core`, `config-loader`, `network`를 조합하여 실행되는 개별 바이너리 (`motor-emulator`, `robot-emulator`, `io-emulator` 등).

---

## 3. 핵심 설계 특징

- **이벤트 루프 & 우선순위 큐**: 
  - 다중 소스에서 들어오는 변경 요청 및 명령을 `std::collections::BinaryHeap` 기반의 우선순위 큐에 적재하고, 싱글 스레드 비동기 이벤트 루프에서 순차적으로 처리.
- **논블로킹 I/O 및 로깅**:
  - 네트워크 통신(NATS)과 런타임 파일 I/O는 `tokio` 비동기로 처리.
  - 로그는 `tracing`과 `tracing-appender`를 결합하여 논블로킹(Non-blocking) 방식으로 처리하여 메인 루프 성능 저하 방지.
- **동작 모드**:
  - **Standalone 모드**: 중앙 서버 없이 에뮬레이터 단독 실행 (로컬 테스트 및 개별 검증).
  - **Cluster 모드**: NATS 및 중앙 제어를 연결하여 여러 장비 간 상태 실시간 동기화 및 설정 단일화(`Single Source of Truth`).

---

## 4. 러스트 초보자를 위한 단계별 개발 로드맵 (5단계)

한 번에 모든 것을 복잡하게 구축하기보다, 가장 안쪽의 핵심부터 단일 프로젝트(Single Crate)로 차근차근 빌드업합니다.

### [1단계] 메모리 맵과 장치 데이터 구조 만들기
- **목표**: 러스트의 `struct`, `enum`, `HashMap`을 익히며 장치의 Address, Offset, 메모리 값을 관리하는 코어 데이터 구조 구현.
- **외부 의존성**: 없음 (순수 러스트 문법 숙달).

### [2단계] 설정 파일(YAML/CSV) 읽어오기
- **목표**: `serde` 및 `serde_yaml`/`csv` 크레이트를 활용해 설정 파일을 읽어 1단계의 데이터 구조에 초기값 세팅.
- **외부 의존성**: `serde`.

### [3단계] 기본 에뮬레이션 및 계산 로직 추가
- **목표**: 에뮬레이터가 틱(Tick)마다 동작하며 값 계산 (예: A2D 변환, Random Noise 추가), 메모리 갱신 루프 구현.

### [4단계] 비동기 루프와 채널(Channel) 도입
- **목표**: `tokio` 런타임을 도입하고, 싱글 스레드 비동기 이벤트 루프 위에서 타이머와 메시지 전달(`mpsc` channel) 구현.
- **외부 의존성**: `tokio`.

### [5단계] NATS 통신 및 우선순위 큐 결합
- **목표**: `async-nats`와 `BinaryHeap`을 결합하여 네트워크 명령 수신 및 우선순위별 순차 처리 구조로 완성.
- **외부 의존성**: `async-nats`.
