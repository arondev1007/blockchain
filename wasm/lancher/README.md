# 🔐 Blockchain Smart Contract VM Launcher

WebAssembly(WASM) 기반의 스마트 컨트랙트 실행기(VM)

## 📌 프로젝트 소개
컴파일된 WebAssembly(WASM) 바이너리를 바이트 코드 형태로 입력받아
스마트 컨트랙트를 실행하는 가상 머신(VM) 런처입니다.

OPCode 단위의 가스 소모량을 제어할 수 있는 미들웨어 구조를 통해
실행 비용을 정밀하게 관리할 수 있으며,
타입 시스템과 런타임 실행 구조를 직접 설계하여
실행 시 타입 안정성과 예외 처리 흐름을 체계화하였습니다.

Host(WASM 실행기)와 Guest(스마트 컨트랙트) 간의
안전한 메모리 읽기/쓰기 인터페이스를 제공하여
신뢰할 수 있는 데이터 공유를 지원합니다.

또한 Imported Function 메커니즘을 활용하여,
가스 소모가 큰 연산을 Host 영역에서 수행하고
그 결과를 Guest로 전달함으로써
성능과 실행 비용을 최적화합니다.

## 🛠 기술 스택
- Language: Rust
- Runtime: WebAssembly (WASM)
- VM Engine: Wasmer
- Architecture: Host / Guest Isolation
- Serialization: Borsh

## 📦 사용 라이브러리
- wasmer
- wasmer-middlewares
- borsh
