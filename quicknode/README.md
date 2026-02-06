# 🔐 Blockchain Node Quick Service

QuickNode 플랫폼을 이용한 블록체인 노드 통신 API 모듈

## 📌 프로젝트 소개
QuickNode 플랫폼을 기반으로 블록체인 노드와 상호작용하는  
Backend API 모듈입니다.

Endpoint 모듈을 통해 비트코인(Bitcoin), 이더리움(Ethereum),
트론(Tron) 네트워크의 노드 RPC 호출을 지원하며,
지갑 생성, 서명(Sign), 서명 검증 기능을 제공합니다.

서명 로직은 자체 라이브러리로 함수화되어 있으며,
해당 기능은 네트워크 통신 없이 로컬에서 수행됩니다.

또한 Stream 기능을 통해 각 네트워크별로 지정된 지갑 주소를
필터링하여, 실시간으로 매칭된 트랜잭션 결과를 수신할 수 있습니다.

## 🛠 기술 스택
- Language: TypeScript
- Runtime: Node.js
- Framework: Express
- Blockchain: Bitcoin, Ethereum, Tron
- Node Provider: QuickNode

## 📦 사용 라이브러리
- HTTP Client: axios
- Environment: dotenv
- Big Number: big.js
- Ethereum: ethers
- Tron: tronweb
- Server: express, body-parser
