# HWP-RS: HWP 파일 포맷 파서 (Rust)

hwp.js를 Rust로 포팅한 프로젝트로, WebAssembly와 데스크톱 CLI 배포를 목표로 합니다.

## 프로젝트 구조

```
hwp-rs/
├── crates/
│   ├── hwp-core/      # 핵심 데이터 모델 및 상수
│   ├── hwp-parser/    # 바이너리 파싱 로직
│   ├── hwp-cli/       # CLI 애플리케이션
│   └── hwp-wasm/      # WebAssembly 바인딩
```

## 현재 구현 상태 (Phase 1 완료)

### ✅ 완료된 작업

#### 1. **Rust 워크스페이스 구조 설정**
- Cargo workspace 구성
- 4개 크레이트 생성 및 의존성 설정
- 최적화 프로파일 구성

#### 2. **hwp-core 크레이트**
- 데이터 모델 정의
  - `HwpHeader`: 파일 헤더 (signature, version, properties)
  - `HwpDocument`: 문서 구조
  - `Section`, `Paragraph`: 섹션 및 단락 구조
  - `Record`: 태그 기반 레코드 시스템
- 상수 정의
  - Tag IDs (DocInfo, Section)
  - Control IDs
  - Fill Types
- 에러 처리 시스템 (`thiserror` 기반)

#### 3. **hwp-parser 크레이트**
- `ByteReader`: Little-endian 바이너리 리더
  - 기본 타입 읽기 (u8, u16, u32, u64)
  - UTF-16LE 문자열 읽기
  - EUC-KR 문자열 읽기
  - 서브 리더 생성
- 헤더 파싱 구현
  - Signature 검증
  - Version 파싱
  - Properties 비트 플래그 처리
- 압축 모듈 (`flate2` 기반)

#### 4. **hwp-cli 크레이트**
- CLI 기본 구조 (`clap` 기반)
- 명령어 인터페이스 정의
  - `inspect`: 메타데이터 확인
  - `convert`: 포맷 변환
  - `validate`: 유효성 검사

#### 5. **hwp-wasm 크레이트**
- WebAssembly 바인딩 기본 구조
- `wasm-bindgen` 인터페이스
- JavaScript API 제공

#### 6. **테스트**
- 단위 테스트 (10개)
- 통합 테스트 (3개)
- 모든 테스트 통과 ✅

## 빌드 및 테스트

```bash
# 빌드
cargo build

# 테스트
cargo test

# CLI 실행
cargo run --bin hwp -- --help

# WASM 빌드 (wasm-pack 필요)
wasm-pack build crates/hwp-wasm --target web
```

## 다음 단계 (Phase 2-4)

### Phase 2: DocInfo 및 Section 파싱 (2주)
- [ ] CFB 컨테이너 처리
- [ ] DocInfo 스트림 파싱
- [ ] Section 레코드 파싱
- [ ] Paragraph 텍스트 추출

### Phase 3: CLI 기능 구현 (1주)
- [ ] 파일 검사 기능
- [ ] JSON/텍스트 변환
- [ ] 배치 처리

### Phase 4: WASM 최적화 (2주)
- [ ] 번들 크기 최소화
- [ ] 성능 최적화
- [ ] JavaScript 예제

## 기술 스택

- **언어**: Rust 2021 Edition
- **주요 의존성**:
  - `byteorder`: 바이너리 파싱
  - `flate2`: 압축/해제
  - `encoding_rs`: 한글 인코딩
  - `clap`: CLI 인터페이스
  - `wasm-bindgen`: WebAssembly 바인딩

## 라이선스

MIT OR Apache-2.0