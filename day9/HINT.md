# HINT

## 시작 전 체크
- Stage 개념이 생소하다면 Reth 문서의 *Header Accumulator* 챕터를 먼저 훑어보세요. 실제 구조가 어떻게 연결되는지 감이 잡힙니다.
- 테스트용 제네시스 헤더를 만드는 헬퍼 함수를 작성해 두면 여러 테스트에서 재사용할 수 있습니다.

## 단계별 가이드
1. **데이터 구조 결정**
   - `BlockHeader`를 정의할 때 `parent_hash`를 빈 문자열 혹은 `"0xgenesis"` 같은 상수로 통일하면 테스트가 단순해집니다.
   - `difficulty`는 작은 숫자라도 `u64`로 두고, 누적값을 받을 `u128`은 `from(header.difficulty)`로 변환하세요.

2. **버퍼 초기화**
   - `HeaderBuffer::new` 안에서 canonical 벡터에 바로 `genesis.clone()`을 push 한 뒤, `index_by_hash.insert(genesis.hash.clone(), 0)`으로 인덱스를 세팅합니다.
   - total difficulty는 `u128::from(genesis.difficulty)`로 시작하세요. 테스트 시 제네시스 난이도를 0 이상으로 설정해야 계산이 편해집니다.

3. **헤더 검증 순서**
   - `try_append`는 중간에 `return`하면서 에러를 돌려도 되지만, 가독성을 위해 작은 검사 함수를 빼내 `validate_header(&header, self)` 형태로 만들어도 좋습니다.
   - `self.head()`가 `None`을 돌려줄 일은 없지만, 제네시스가 항상 존재한다는 가정이 불안하다면 `expect("genesis must exist")`로 의도를 드러내세요.

4. **인덱스 업데이트**
   - canonical 길이는 push 이후 `self.canonical.len() - 1`이 새 헤더의 인덱스가 됩니다. 이 값을 그대로 `index_by_hash`에 저장하면 됩니다.
   - parent 검증을 통과했다면 `index_by_hash.contains_key(&header.parent_hash)`를 이미 체크한 상태이므로 `unwrap`을 사용해도 괜찮습니다.

5. **테스트 작성**
   - `assert_eq!(buffer.head().unwrap().hash, expected_hash);`처럼 head를 확인해 최종 체인이 원하는 방향으로 성장했는지 증명하세요.
   - 에러 케이스 테스트에서는 `matches!(err, HeaderInsertError::ParentNotFound { .. })` 구문을 사용하면 가독성이 좋습니다.

## 디버깅 팁
- 인덱스가 어긋난다면 `dbg!(&buffer.index_by_hash)`로 현재 상태를 확인해 보세요.
- 테스트에서 총 난이도가 기대값과 다르면 제네시스 난이도를 빼먹었는지, 혹은 중복 헤더가 삽입된 건 아닌지 확인합니다.

## 참고 자료
- Reth 공식 문서: https://paradigmxyz.github.io/reth/
- Ethereum Yellow Paper (헤더 구조): https://ethereum.github.io/yellowpaper/paper.pdf
- Rust 표준 라이브러리 HashMap: https://doc.rust-lang.org/std/collections/struct.HashMap.html
- `matches!` 매크로 설명: https://doc.rust-lang.org/std/macro.matches.html
