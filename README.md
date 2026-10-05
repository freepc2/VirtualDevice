# Virtual Device Emulator (Multi-Language: Rust & C++)

본 프로젝트는 산업용 가상 디바이스(TwinCAT, Comizoa 등 PLC/컨트롤러) 에뮬레이터를 **Rust**와 **Modern C++ (C++20)** 두 가지 언어로 각각 구현하고, 성능(처리량, 메모리 사용량)과 언어적 인체공학(Ergonomics)을 비교·학습하기 위한 폴리글랏(Polyglot) 학습 프로젝트입니다.

---

## 📂 프로젝트 폴더 구조 (GitHub 컨벤션 준수)

GitHub 및 오픈소스 멀티랭귀지 비교 저장소의 표준 관례에 따라 언어별로 디렉토리를 완전히 분리하여 빌드 시스템과 툴링 충돌을 방지하였습니다.

```text
VirtualDevice/
├── docs/                      # 아키텍처 설계 및 명세서
│   └── architecture.md
├── rust/                      # 🦀 러스트(Rust) 구현 패키지 (Cargo)
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── device_memory/     # 메모리 맵 및 물리 주소 (PhyAddress)
│   │   └── emulators/         # 모터 및 I/O 에뮬레이터
│   └── tests/
│       └── emulator_tests.rs  # 단위 테스트 및 1초 처리량 벤치마크
├── cpp/                       # ⚡ 모던 C++20 구현 패키지 (CMake)
│   ├── CMakeLists.txt
│   ├── include/
│   │   └── virtualdevice/     # 헤더 전용 및 인라인 모듈
│   ├── src/
│   │   └── main.cpp
│   └── tests/
│       └── emulator_tests.cpp # C++ 단위 테스트 및 벤치마크
├── benchmarks/                # 크로스 언어 성능 비교 가이드 및 스크립트
│   └── README.md
├── .gitignore
└── README.md                  # 본 문서
```

---

## 🚀 시작하기 (Build & Run)

### 1. 🦀 Rust 구현부 실행 및 테스트
Rust 패키지는 `rust/` 디렉토리 내부에 위치합니다.

- **단위 테스트 및 벤치마크 실행**:
  ```bash
  cargo test --manifest-path rust/Cargo.toml --release
  ```
- **메인 실행파일 실행**:
  ```bash
  cargo run --manifest-path rust/Cargo.toml --release
  ```

### 2. ⚡ C++ 구현부 빌드 및 테스트
C++ 패키지는 `cpp/` 디렉토리 내부에 위치하며 `CMake` (C++20)를 사용합니다.

- **빌드 (CMake)**:
  ```bash
  cd cpp
  cmake -B build
  cmake --build build --config Release
  ```
- **테스트 및 벤치마크 실행**:
  ```bash
  cd cpp/build
  ctest --output-on-failure -C Release
  # 또는 직접 실행
  ./Release/emulator_tests.exe  # (Windows 기준)
  ```

---

## 📊 성능 비교 항목 (Performance Benchmarks)

두 구현체 모두 동일한 워크로드로 다음 항목들을 측정하고 비교합니다:
1. **초당 신규 주소 생성 및 쓰기 처리량 (Write Throughput / Sec)**: `PhyAddress` 키 생성 후 인메모리 해시맵(`HashMap` vs `unordered_map`)에 적재하는 연산 속도.
2. **초당 읽기 처리량 (Read Throughput / Sec)**: 500,000개의 메모리가 사전 적재된 상태에서 무작위 조회 수행 속도.
3. **메모리 오버헤드 (Memory Footprint)**: 키와 값 보관에 소요되는 대략적인 메모리 점유 크기.
