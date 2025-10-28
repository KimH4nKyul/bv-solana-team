# Day 9: Reth 스타일 헤더 버퍼 기초 구현

**난이도: EASY (Reth 코어 입문)**

## 왜 이 과제를 하나요?
- Day 8에서 만든 실행 대기열을 넘어, 이제 합법적인 헤더만 누적할 수 있는 얕은 체인 관리기를 만들어 Stage 파이프라인의 첫 단계를 이해합니다.
- 헤더를 메모리에 쌓으면서 부모-자식 관계, 블록 번호, 누적 난이도로 체인을 검증하는 방법을 손으로 익힙니다.
- 간단한 구조체와 테스트만으로도 Reth가 채택하는 방어적 프로그래밍 패턴을 재현할 수 있다는 감각을 얻습니다.

## 오늘 완성해야 할 산출물
- `day9_reth_header_buffer` 라이브러리 프로젝트
- `src/lib.rs` 내부에 정의된 `BlockHeader`, `HeaderBuffer`, `HeaderInsertError`
- `tests/header_buffer.rs`에 세부 시나리오가 포함된 통합 테스트

## 사전 준비
- Rust 1.75 이상과 `cargo`가 설치되어 있어야 합니다.
- 새 프로젝트는 빈 디렉터리에서 시작하세요. 현재 워크스페이스에 이미 프로젝트가 있다면 하위 폴더로 이동한 뒤 진행합니다.

## 프로젝트 생성과 기본 구조
```bash
cargo new reth_header_buffer --lib
cd reth_header_buffer
```

생성 직후 디렉터리 구조는 다음과 같습니다.
```
reth_header_buffer
├── Cargo.toml
├── src
│   └── lib.rs
└── tests
    └── header_buffer.rs      (직접 생성)
```
- 모든 라이브러리 로직은 `src/lib.rs`에 작성합니다.
- 통합 테스트 파일 `tests/header_buffer.rs`를 새로 만들고, 각 테스트가 무엇을 보장하는지 한 문장짜리 주석을 맨 위에 추가합니다.

## 구현 로드맵
1. **헤더 표현**
   - `BlockHeader` 구조체에 `#[derive(Clone, Debug, PartialEq, Eq)]`를 붙이고 다음 필드를 정의합니다.
     ```rust
     pub struct BlockHeader {
         pub number: u64,
         pub hash: String,
         pub parent_hash: String,
         pub difficulty: u64,
     }
     ```
   - 선언부 위에 *“이 구조체가 Reth에서 어떤 정보를 흉내 내는지”* 설명하는 한 줄짜리 한국어 주석을 추가하세요.

2. **버퍼 상태**
   - `use std::collections::HashMap;`으로 맵을 가져옵니다.
   - `HeaderBuffer` 구조체를 선언하고 다음 필드를 넣습니다.
     ```rust
     pub struct HeaderBuffer {
         canonical: Vec<BlockHeader>,
         index_by_hash: HashMap<String, usize>,
         total_difficulty: u128,
     }
     ```
   - 각 필드 위에 *“Reth에서 어떤 구성요소가 비슷한 역할을 하는지”* 간단한 주석을 달아 맥락을 연결합니다.

3. **필수 메서드**
   - `impl HeaderBuffer` 블록에 아래 메서드를 구현합니다.
     - `pub fn new(genesis: BlockHeader) -> Self`  
       제네시스를 canonical 체인에 첫 항목으로 넣고, 인덱스와 total difficulty를 초기화합니다. 함수 선언 위에 *“제네시스가 왜 특별 취급되는가”*를 설명하는 주석을 적습니다.
     - `pub fn head(&self) -> Option<&BlockHeader>`  
       canonical 체인의 마지막 헤더를 참조로 반환합니다.
     - `pub fn total_difficulty(&self) -> u128`  
       현재까지 누적된 난이도를 그대로 돌려줍니다.

4. **헤더 추가 검증**
   - 아래 변형을 갖는 `HeaderInsertError` enum을 정의합니다. 각 변형 위에는 *“이 오류가 언제 발생하는지”*를 적어 놓으세요.
     - `ParentNotFound { parent_hash: String }`
     - `NumberMismatch { expected: u64, got: u64 }`
     - `DuplicateHash { hash: String }`
   - `pub fn try_append(&mut self, header: BlockHeader) -> Result<(), HeaderInsertError>`를 구현합니다.
     - 이미 존재하는 해시라면 `DuplicateHash`.
     - 부모 해시가 `index_by_hash`에 없다면 `ParentNotFound`.
     - 헤더 번호가 `head.number + 1`이 아니라면 `NumberMismatch`.
     - 검증을 모두 통과하면 canonical 벡터에 push 하고, 인덱스와 total difficulty를 갱신합니다.

5. **테스트 시나리오 (`tests/header_buffer.rs`)**
   - `// 이 테스트 스위트가 ...` 형태의 한 문장 주석으로 파일 목적을 설명합니다.
   - 최소 세 개의 테스트를 작성합니다.
     1. **정상 연결**: 제네시스 뒤에 두 개의 연속된 헤더를 추가하고 head, canonical 길이, total difficulty를 검증합니다.
     2. **부모 없음**: 등록되지 않은 `parent_hash`를 가진 헤더 삽입 시 `HeaderInsertError::ParentNotFound`가 나는지 확인합니다.
     3. **번호 불일치**: 부모는 맞지만 번호가 건너뛰어진 헤더일 때 `HeaderInsertError::NumberMismatch`가 반환되는지 확인합니다.
   - 각 테스트에서 `try_append` 결과뿐 아니라 canonical 벡터 길이나 해시 인덱스 상태 등 하나 이상의 부가 조건을 함께 검증하세요.

## 검증 순서 미리 보기
```rust
fn validate(
    header: &BlockHeader,
    head: &BlockHeader,
    index: &HashMap<String, usize>,
) -> Result<(), HeaderInsertError> {
    if index.contains_key(&header.hash) {
        return Err(HeaderInsertError::DuplicateHash { hash: header.hash.clone() });
    }
    if !index.contains_key(&header.parent_hash) {
        return Err(HeaderInsertError::ParentNotFound { parent_hash: header.parent_hash.clone() });
    }
    if header.number != head.number + 1 {
        return Err(HeaderInsertError::NumberMismatch { expected: head.number + 1, got: header.number });
    }
    Ok(())
}
```
- Reth의 헤더 Stage도 거의 동일한 순서를 따르며, 단지 실제 구현에서는 DB와 Stage 파이프라인으로 확장됩니다.
- 우리는 메모리 기반으로 축소했지만, 체인을 보호하는 핵심 규칙(부모-자식, 번호, total difficulty)은 그대로 유지됩니다.

## 마무리 체크리스트
- 제네시스 헤더는 테스트 전반에서 동일한 값을 사용해 일관성을 유지합니다 (`parent_hash`는 빈 문자열 혹은 고정된 더미 해시).
- `total difficulty`는 `u128`으로 관리해 `u64` 난이도가 누적돼도 안전합니다.
- 해시 인덱스(`HashMap<String, usize>`)를 통해 부모 조회가 O(1)에 가깝게 이뤄집니다.
- 오류 타입이 명확하면 이후 Stage나 상위 모듈에서 상황별로 대처하기 쉽습니다.

## 실행 전 필수 명령
- `cargo fmt`
- `cargo clippy`
- `cargo test`

### 오늘의 TIL
- 헤더 동기화의 기본 규칙(부모 해시, 블록 번호, total difficulty)을 손으로 구현해 Stage 파이프라인의 출발점을 체험했습니다.
