# HINT Day 7

## [구현 힌트]

### 💡 `Ord` 트레이트 구현하기

`MempoolEntry`에 `Ord`를 구현할 때 주의할 점:

```rust
impl Ord for MempoolEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // 오직 수수료만 비교합니다
        self.fee_micro_lamports.cmp(&other.fee_micro_lamports)
    }
}
```

**왜 `fee_micro_lamports`만 비교하나요?**
- `BinaryHeap`은 최대 힙(max-heap)이므로 큰 값이 먼저 나옵니다.
- 수수료가 높은 트랜잭션이 우선순위가 높아야 합니다.
- `compute_units`는 비교 기준에서 제외합니다.

**대안:** `#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]`를 사용하면 모든 필드를 순서대로 비교하는데, 이 경우 수수료만 비교하려면 직접 구현하는 것이 낫습니다.

---

### 💡 BinaryHeap에서 정렬된 벡터 만들기

안전하고 읽기 좋은 패턴을 사용하세요:

```rust
pub fn drain_sorted_by_fee(heap: &mut BinaryHeap<MempoolEntry>) -> Vec<MempoolEntry> {
    let mut sorted = Vec::new();
    // while let 패턴: heap이 비어있으면 자동으로 종료
    while let Some(entry) = heap.pop() {
        sorted.push(entry);
    }
    sorted
}
```

**왜 이 패턴이 좋을까요?**
- `while let`은 `None`이 나올 때까지 자동으로 반복합니다.
- 명시적인 길이 체크가 필요 없습니다.
- 안전하고 Rust다운(idiomatic) 코드입니다.

---

### 💡 `try_add` 구현 순서

**항상 이 순서를 지키세요:**

```rust
pub fn try_add(&mut self, entry: MempoolEntry) -> bool {
    // 1️⃣ 먼저 계산 (예상 합계)
    if self.can_add(&entry) {
        // 2️⃣ 비교 통과 시 합계 업데이트
        self.current_bundle.total_compute_units += entry.compute_units;
        self.current_bundle.total_fee_micro_lamports += entry.fee_micro_lamports;

        // 3️⃣ 마지막에 push
        self.current_bundle.entries.push(entry);
        true
    } else {
        false
    }
}
```

**왜 이 순서가 중요한가요?**
- 합계를 먼저 업데이트하면 데이터 일관성이 보장됩니다.
- `push` 전에 검증하므로 실패 시 원상태를 유지합니다.

---

### 💡 성능 최적화 팁

**합계를 구조체에 저장하세요:**

```rust
pub struct PlannedBundle {
    pub entries: Vec<MempoolEntry>,
    pub total_compute_units: u32,           // ← 이미 계산된 합계
    pub total_fee_micro_lamports: u64,      // ← 이미 계산된 합계
}
```

**장점:**
- 매번 `entries.iter().map(|e| e.compute_units).sum()`를 호출할 필요가 없습니다.
- O(n) 연산을 O(1)로 단축합니다.
- 트랜잭션이 수천 개일 때 큰 성능 차이가 납니다.

---

### 💡 `remaining_capacity` 구현

안전한 빼기 연산을 사용하세요:

```rust
pub fn remaining_capacity(&self, constraint: &BlockConstraint) -> (u32, usize) {
    let remaining_compute = constraint.max_compute_units
        .saturating_sub(self.total_compute_units);  // ← 음수 방지

    let remaining_transactions = constraint.max_transactions
        .saturating_sub(self.entries.len());

    (remaining_compute, remaining_transactions)
}
```

**`saturating_sub`의 장점:**
- 결과가 음수가 되려고 하면 자동으로 0을 반환합니다.
- 패닉(panic) 없이 안전합니다.
- 예: `5u32.saturating_sub(10)` → `0`

**일반 빼기의 문제:**
```rust
let x: u32 = 5;
let y: u32 = 10;
let z = x - y;  // ❌ 패닉 발생! (디버그 모드)
```

---

## [디버깅 팁]

### 🐛 테스트가 실패할 때

**문제:** `try_add`가 항상 `false`를 반환합니다.
```rust
// 🔍 체크 포인트:
// 1. can_add의 비교 연산자가 올바른가? (<= vs <)
// 2. 초기값이 0으로 설정되었는가?
// 3. 제약 조건이 너무 작게 설정되지 않았는가?
```

**문제:** 합계가 맞지 않습니다.
```rust
// 🔍 체크 포인트:
// 1. try_add에서 합계를 업데이트하고 있는가?
// 2. 업데이트 순서가 올바른가? (+=를 두 번 했나?)
// 3. entries.push() 전에 합계를 업데이트했는가?
```

**문제:** BinaryHeap이 원하는 순서로 정렬되지 않습니다.
```rust
// 🔍 체크 포인트:
// 1. Ord 구현에서 fee_micro_lamports를 비교하는가?
// 2. cmp의 순서가 올바른가? (self vs other)
// 3. PartialOrd도 함께 구현했는가?
```

---

## [일반적인 실수]

### ❌ 실수 1: 검증 없이 추가하기
```rust
pub fn try_add(&mut self, entry: MempoolEntry) -> bool {
    self.current_bundle.entries.push(entry);  // ❌ 검증 없음!
    true
}
```

### ✅ 올바른 방법:
```rust
pub fn try_add(&mut self, entry: MempoolEntry) -> bool {
    if self.can_add(&entry) {  // ✅ 먼저 검증
        // 추가 로직...
        true
    } else {
        false
    }
}
```

---

### ❌ 실수 2: 합계를 나중에 계산하기
```rust
pub fn finalize(self) -> PlannedBundle {
    // ❌ 매번 순회하며 계산 (느림!)
    let total = self.current_bundle.entries
        .iter()
        .map(|e| e.compute_units)
        .sum();
    // ...
}
```

### ✅ 올바른 방법:
```rust
// ✅ 추가할 때마다 합계 업데이트 (빠름!)
self.current_bundle.total_compute_units += entry.compute_units;
```

---

## [참고자료]

### 📚 공식 문서
- **Solana 트랜잭션 처리**: https://docs.solanalabs.com/validator/transaction-processing
- **Rust Book - 구조체**: https://doc.rust-lang.org/book/ch05-00-structs.html
- **Rust Book - 테스트 작성**: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
- **Rust std::collections::BinaryHeap**: https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html

### 🔑 핵심 키워드
검색할 때 유용한 키워드들:

- **greedy selection** (그리디 선택 알고리즘)
- **compute budget** (컴퓨트 예산 관리)
- **saturating arithmetic** (포화 산술 연산)
- **happy path testing** (정상 시나리오 테스트)
- **max-heap ordering** (최대 힙 정렬)
- **transaction bundling** (트랜잭션 번들링)
- **priority queue** (우선순위 큐)

---

## [추가 도전 과제]

구현을 완료했다면 다음 기능들을 추가해 보세요:

1. **로깅 추가**: 각 트랜잭션이 추가될 때마다 로그 출력
2. **통계 메서드**: 평균 수수료, 최대/최소 컴퓨트 유닛 계산
3. **벤치마크**: 1만 개의 트랜잭션 처리 시간 측정
4. **Iterator 구현**: `IntoIterator`를 구현하여 for 루프 사용 가능하게 만들기