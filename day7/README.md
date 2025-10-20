# Day 7: 슬롯 플래너로 천천히 묶음 만들기

**난이도: EASY (입문자 친화형 연습)**

## [전날 과제 요약]
Day 6에서 우리는 블록체인 네트워크에서 트랜잭션의 우선순위를 결정하는 핵심 메커니즘을 직접 구현해 보았습니다:

- **수수료 기반 우선순위 큐**: 높은 수수료를 지불한 트랜잭션을 먼저 처리하도록 `BinaryHeap`을 활용한 큐를 만들었습니다.
- **커스텀 정렬**: `Ord`, `PartialOrd` 트레이트를 구현하여 우리만의 정렬 기준을 설정하고, 자료구조를 안전하게 조작하는 방법을 배웠습니다.
- **테스트 주도 개발(TDD)**: 테스트 코드를 먼저 작성하고, 그 테스트를 통과하도록 구현하면서 요구사항이 정확히 충족되는지 검증하는 방법을 익혔습니다.

## [전날 과제로 얻은 역량]
Day 6를 마치면서 여러분은 다음과 같은 실전 개발 능력을 갖추게 되었습니다:

- **우선순위 기반 데이터 처리**: 정렬된 데이터를 반복해서 꺼내며 처리 순서를 제어하는 능력. 실제 블록체인에서 트랜잭션 순서를 결정하는 핵심 스킬입니다.
- **명확한 데이터 모델링**: 구조체(struct)와 열거형(enum)을 활용하여 복잡한 비즈니스 로직을 이해하기 쉬운 타입으로 표현하는 감각.
- **안전한 개발 습관**: 작은 단위의 테스트부터 작성하여 요구사항을 검증하면서 구현하는 TDD 방식의 개발 흐름.

## [오늘 과제 목표]
오늘은 Day 6에서 배운 우선순위 큐를 실제로 활용하여 블록을 구성하는 플래너를 만들어 봅니다:

- **슬롯 플래너 구현**: 블록에 넣을 트랜잭션 묶음을 선택하는 `SlotPlanner`를 만들어 봅니다. 실제 블록체인 검증자(validator)가 하는 일을 단순화한 버전입니다.
- **그리디(탐욕) 알고리즘 학습**: 현재 상황에서 가장 좋은 선택을 반복하며 전체 문제를 해결하는 그리디 패턴을 직접 구현합니다.
- **제약 조건 관리**: 컴퓨트 유닛과 트랜잭션 수 제한을 넘지 않도록 합계를 계산하고 검증하는 로직을 작성합니다.
- **정렬된 데이터 활용**: Day 6서 만든 `BinaryHeap`을 활용하여 수수료가 높은 순으로 정렬된 트랜잭션 목록을 만들고 처리합니다.
- **테스트 주도 학습**: 성공 케이스부터 시작해서 점진적으로 복잡한 시나리오를 추가하는 TDD 방식을 계속 연습합니다.

## [오늘 과제 설명]
오늘 과제는 Day 6에서 만든 우선순위 큐의 실전 활용입니다. 수수료가 높은 순으로 정렬된 트랜잭션 목록을 받아서, 블록의 제약(컴퓨트 유닛 한도, 트랜잭션 수 한도)을 넘지 않는 범위에서 최대한 많은 트랜잭션을 선택하는 플래너를 만듭니다.

**핵심 규칙은 매우 간단합니다:**
- 담을 수 있으면 담습니다 ✅
- 제한을 넘으면 건너뜁니다 ⏭️
- 복잡한 에러 처리나 롤백은 없습니다!

이것이 실제 블록체인에서 "번들 빌더(Bundle Builder)"가 하는 일의 핵심입니다. 아래 단계를 천천히 따라가며 직접 구현해 봅시다.

### 1️⃣ **프로젝트 생성**

먼저 새로운 Rust 라이브러리 프로젝트를 만듭니다:

```bash
cargo new day7_slot_planner --lib
cd day7_slot_planner
```

**파일 구조:**
- `src/lib.rs`: 주요 로직과 데이터 구조
- `tests/planner.rs`: 통합 테스트 파일 (직접 생성 필요)

---

### 2️⃣ **데이터 구조 만들기 (`src/lib.rs`)**

블록체인의 트랜잭션과 제약을 표현하는 세 가지 핵심 구조체를 정의합니다.

#### **`MempoolEntry` - 트랜잭션 정보**
Day 6에서 사용했던 구조체를 다시 정의합니다:

```rust
/// 메모리풀에 대기 중인 트랜잭션 하나를 나타냅니다.
pub struct MempoolEntry {
    pub compute_units: u32,         // 이 트랜잭션이 사용할 계산 자원
    pub fee_micro_lamports: u64,    // 사용자가 지불한 수수료
}
```

💡 **왜 이 두 필드가 중요할까요?**
- `compute_units`: 트랜잭션이 실행될 때 사용하는 계산량입니다. 블록에는 한도가 있어요.
- `fee_micro_lamports`: 수수료가 높을수록 우선순위가 높아집니다.

#### **`BlockConstraint` - 블록 제한**
한 블록에 담을 수 있는 최대치를 정의합니다:

```rust
/// 한 블록(슬롯)이 수용할 수 있는 최대 제한을 나타냅니다.
pub struct BlockConstraint {
    pub max_compute_units: u32,      // 총 계산 자원 한도
    pub max_transactions: usize,     // 담을 수 있는 트랜잭션 개수 한도
}
```

💡 **실제 블록체인에서는?**
- Solana는 슬롯(slot)마다 최대 4,800만 컴퓨트 유닛을 사용할 수 있습니다.
- 한 블록에 담을 수 있는 트랜잭션 수도 제한되어 있습니다.

#### **`PlannedBundle` - 선택된 트랜잭션 묶음**
실제로 블록에 담을 트랜잭션들과 통계를 저장합니다:

```rust
/// 블록에 담기로 선택된 트랜잭션 묶음과 그 합계 정보를 나타냅니다.
pub struct PlannedBundle {
    pub entries: Vec<MempoolEntry>,        // 선택된 트랜잭션 목록
    pub total_compute_units: u32,          // 선택된 트랜잭션들의 컴퓨트 합계
    pub total_fee_micro_lamports: u64,     // 선택된 트랜잭션들의 수수료 합계
}
```

💡 **왜 합계를 따로 저장하나요?**
- 매번 벡터를 순회하며 합계를 계산하는 것보다 훨씬 빠릅니다.
- 새 트랜잭션을 추가할 때마다 합계만 업데이트하면 됩니다.

---

### 3️⃣ **BinaryHeap으로 입력 정렬하기 (`src/lib.rs`)**

수수료가 높은 트랜잭션부터 처리하면 블록의 수익을 극대화할 수 있습니다. `BinaryHeap`을 활용하여 자동으로 정렬된 벡터를 만드는 도우미 함수를 작성합니다.

#### **정렬 기준 구현**

먼저 `MempoolEntry`가 수수료 기준으로 비교될 수 있도록 트레이트를 구현합니다:

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

// 수수료로 비교 가능하도록 만들기
impl Ord for MempoolEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fee_micro_lamports.cmp(&other.fee_micro_lamports)
    }
}

impl PartialOrd for MempoolEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MempoolEntry {
    fn eq(&self, other: &Self) -> bool {
        self.fee_micro_lamports == other.fee_micro_lamports
    }
}

impl Eq for MempoolEntry {}
```

💡 **핵심 포인트:**
- `BinaryHeap`은 기본적으로 **최대 힙(max-heap)**입니다.
- 수수료가 큰 값일수록 먼저 나오도록 `fee_micro_lamports`만 비교합니다.

#### **정렬된 벡터 만들기**

힙에서 항목을 하나씩 꺼내 벡터로 변환하는 도우미 함수를 작성합니다:

```rust
/// BinaryHeap에서 수수료가 높은 순으로 정렬된 벡터를 만듭니다.
/// Day 6에서 만든 우선순위 큐를 실제로 활용하는 함수입니다.
pub fn drain_sorted_by_fee(heap: &mut BinaryHeap<MempoolEntry>) -> Vec<MempoolEntry> {
    let mut sorted = Vec::new();
    while let Some(entry) = heap.pop() {
        sorted.push(entry);
    }
    sorted  // 수수료가 높은 순서대로 정렬됨
}
```

💡 **작동 원리:**
- `heap.pop()`은 매번 가장 큰 값(수수료가 가장 높은 항목)을 반환합니다.
- 반복하면 자연스럽게 내림차순으로 정렬된 벡터가 만들어집니다.

---

### 4️⃣ **슬롯 플래너 기본 뼈대 (`src/lib.rs`)**

이제 실제로 트랜잭션을 선택하는 플래너를 만들 차례입니다.

#### **구조체 정의**

```rust
/// 블록 제약 조건 내에서 최적의 트랜잭션 묶음을 선택하는 플래너입니다.
pub struct SlotPlanner {
    constraint: BlockConstraint,      // 이 블록이 지켜야 할 제한
    current_bundle: PlannedBundle,    // 현재까지 선택된 트랜잭션들
}
```

#### **생성자 구현**

```rust
impl SlotPlanner {
    /// 새로운 슬롯 플래너를 생성합니다.
    /// 처음에는 빈 번들로 시작하며, 트랜잭션을 하나씩 추가해 나갑니다.
    pub fn new(constraint: BlockConstraint) -> Self {
        Self {
            constraint,
            current_bundle: PlannedBundle {
                entries: Vec::new(),
                total_compute_units: 0,
                total_fee_micro_lamports: 0,
            },
        }
    }
}
```

💡 **왜 생성자에서 번들을 비워 두나요?**
- 플래너는 "빈 상태"에서 시작해서 트랜잭션을 하나씩 추가해 나갑니다.
- 마치 빈 박스에서 시작해서 물건을 하나씩 담아가는 것과 같습니다.
- 초기 합계는 모두 0이고, 벡터는 비어 있습니다.

---

### 5️⃣ **핵심 메서드 구현 (`src/lib.rs`)**

플래너의 핵심 기능인 트랜잭션 추가와 완료 로직을 구현합니다.

#### **도우미 함수: 추가 가능 여부 확인**

먼저 새 트랜잭션을 추가해도 괜찮은지 확인하는 내부 함수를 만듭니다:

```rust
impl SlotPlanner {
    /// 새 항목을 추가해도 제한을 넘지 않는지 확인합니다.
    fn can_add(&self, entry: &MempoolEntry) -> bool {
        // 추가했을 때의 예상 합계 계산
        let next_compute = self.current_bundle.total_compute_units + entry.compute_units;
        let next_count = self.current_bundle.entries.len() + 1;

        // 두 가지 제한을 모두 확인
        next_compute <= self.constraint.max_compute_units
            && next_count <= self.constraint.max_transactions
    }
}
```

💡 **핵심 로직:**
- 먼저 추가했을 때의 합계를 **미리 계산**합니다.
- 두 가지 조건을 **모두** 만족해야 합니다:
  - 컴퓨트 유닛이 한도를 넘지 않아야 함
  - 트랜잭션 개수가 한도를 넘지 않아야 함

#### **메서드 1: `try_add` - 트랜잭션 추가 시도**

```rust
impl SlotPlanner {
    /// 새 트랜잭션을 번들에 추가를 시도합니다.
    /// 성공하면 true, 제한을 넘어서 실패하면 false를 반환합니다.
    pub fn try_add(&mut self, entry: MempoolEntry) -> bool {
        if self.can_add(&entry) {
            // 제한 내에 있으면 추가하고 합계 업데이트
            self.current_bundle.total_compute_units += entry.compute_units;
            self.current_bundle.total_fee_micro_lamports += entry.fee_micro_lamports;
            self.current_bundle.entries.push(entry);
            true  // 성공!
        } else {
            false  // 제한 초과로 실패
        }
    }
}
```

💡 **왜 `bool`을 반환하나요?**
- `true`: "성공적으로 추가했어요!" → 호출자는 계속 다음 항목을 시도합니다.
- `false`: "이건 못 담겠어요" → 호출자는 이 항목을 건너뛰고 다음으로 넘어갑니다.
- 복잡한 에러 타입 없이도 충분히 의도를 전달할 수 있습니다.

#### **메서드 2: `finalize` - 완성된 번들 반환**

```rust
impl SlotPlanner {
    /// 지금까지 선택한 트랜잭션 묶음을 반환하고 플래너를 소비합니다.
    pub fn finalize(self) -> PlannedBundle {
        self.current_bundle
    }
}
```

💡 **왜 `self`를 소비하나요?**
- `finalize`를 호출하면 플래너가 "완료"되었다는 의미입니다.
- 완료 후에는 더 이상 트랜잭션을 추가할 수 없어야 합니다.
- Rust의 소유권 시스템이 이를 컴파일 타임에 보장해 줍니다!

---

### 6️⃣ **잔여 용량 계산 (`src/lib.rs`)**

현재 번들에 얼마나 더 추가할 수 있는지 계산하는 유틸리티 함수를 구현합니다.

```rust
impl PlannedBundle {
    /// 주어진 제약 조건에서 남은 용량을 계산합니다.
    /// 반환값: (남은 컴퓨트 유닛, 남은 트랜잭션 개수)
    pub fn remaining_capacity(&self, constraint: &BlockConstraint) -> (u32, usize) {
        let remaining_compute = constraint.max_compute_units
            .saturating_sub(self.total_compute_units);

        let remaining_transactions = constraint.max_transactions
            .saturating_sub(self.entries.len());

        (remaining_compute, remaining_transactions)
    }
}
```

💡 **`saturating_sub`란?**
- 일반 빼기(`-`)는 음수가 나올 수 있습니다.
- `saturating_sub`는 결과가 음수가 되려고 하면 **자동으로 0**을 반환합니다.
- 예: `5u32.saturating_sub(10)` → `0` (패닉 없음!)

💡 **왜 이 함수가 유용한가요?**
- 디버깅할 때 현재 상태를 쉽게 파악할 수 있습니다.
- 로깅이나 모니터링에 활용할 수 있습니다.
- 테스트 작성 시 검증 용도로 사용됩니다.

---

### 7️⃣ **테스트 작성 (`tests/planner.rs`)**

이제 우리가 만든 코드가 올바르게 동작하는지 검증하는 테스트를 작성합니다.

#### **테스트 파일 생성**

먼저 `tests` 디렉토리를 만들고 `planner.rs` 파일을 생성합니다:

```bash
mkdir tests
touch tests/planner.rs
```

#### **테스트 1: 제한 초과 시나리오**

제한을 넘는 트랜잭션을 추가하려고 할 때 올바르게 거부하는지 확인합니다:

```rust
//! SlotPlanner의 제약 조건 검증 및 트랜잭션 선택 로직을 테스트합니다.

use day7_slot_planner::{BlockConstraint, MempoolEntry, SlotPlanner};

#[test]
fn test_reject_when_exceeds_limit() {
    // 작은 제한을 설정
    let constraint = BlockConstraint {
        max_compute_units: 1000,
        max_transactions: 2,
    };

    let mut planner = SlotPlanner::new(constraint);

    // 첫 번째 트랜잭션: 성공
    let tx1 = MempoolEntry {
        compute_units: 500,
        fee_micro_lamports: 1000,
    };
    assert!(planner.try_add(tx1), "첫 번째 트랜잭션은 추가되어야 함");

    // 두 번째 트랜잭션: 성공
    let tx2 = MempoolEntry {
        compute_units: 400,
        fee_micro_lamports: 800,
    };
    assert!(planner.try_add(tx2), "두 번째 트랜잭션은 추가되어야 함");

    // 세 번째 트랜잭션: 실패 (컴퓨트 한도 초과)
    let tx3 = MempoolEntry {
        compute_units: 200,  // 900 + 200 = 1100 > 1000
        fee_micro_lamports: 500,
    };
    assert!(!planner.try_add(tx3), "제한 초과로 거부되어야 함");

    // 번들 확인
    let bundle = planner.finalize();
    assert_eq!(bundle.entries.len(), 2, "2개만 추가되어야 함");
    assert_eq!(bundle.total_compute_units, 900);
    assert_eq!(bundle.total_fee_micro_lamports, 1800);
}
```

💡 **무엇을 검증하나요?**
- 제한을 넘는 트랜잭션은 추가되지 않습니다.
- 거부된 후에도 기존 번들은 변경되지 않습니다.

#### **테스트 2: BinaryHeap 정렬 검증**

수수료가 높은 순으로 트랜잭션을 선택하는지 확인합니다:

```rust
use std::collections::BinaryHeap;
use day7_slot_planner::drain_sorted_by_fee;

#[test]
fn test_sorted_selection_by_fee() {
    // BinaryHeap에 트랜잭션 추가 (순서는 무작위)
    let mut heap = BinaryHeap::new();
    heap.push(MempoolEntry {
        compute_units: 100,
        fee_micro_lamports: 500,  // 낮은 수수료
    });
    heap.push(MempoolEntry {
        compute_units: 200,
        fee_micro_lamports: 2000,  // 높은 수수료
    });
    heap.push(MempoolEntry {
        compute_units: 150,
        fee_micro_lamports: 1500,  // 중간 수수료
    });

    // 정렬된 벡터로 변환
    let sorted = drain_sorted_by_fee(&mut heap);

    // 수수료가 높은 순서로 나와야 함
    assert_eq!(sorted[0].fee_micro_lamports, 2000);
    assert_eq!(sorted[1].fee_micro_lamports, 1500);
    assert_eq!(sorted[2].fee_micro_lamports, 500);

    // 플래너에 순서대로 추가
    let constraint = BlockConstraint {
        max_compute_units: 10000,
        max_transactions: 10,
    };
    let mut planner = SlotPlanner::new(constraint);

    for entry in sorted {
        planner.try_add(entry);
    }

    let bundle = planner.finalize();
    assert_eq!(bundle.entries.len(), 3);
    assert_eq!(bundle.total_compute_units, 450);

    // 잔여 용량 확인
    let (remaining_compute, remaining_txs) = bundle.remaining_capacity(&constraint);
    assert_eq!(remaining_compute, 9550);
    assert_eq!(remaining_txs, 7);
}
```

💡 **무엇을 검증하나요?**
- `BinaryHeap`이 수수료 순으로 정렬합니다.
- `drain_sorted_by_fee`가 올바른 순서를 반환합니다.
- `remaining_capacity`가 정확한 잔여량을 계산합니다.

#### **테스트 실행**

```bash
cd day7_slot_planner
cargo test
```

모든 테스트가 통과하면 성공입니다! ✅

---

### 8️⃣ **마무리 루틴**

코드 작성이 끝났다면 다음 순서로 코드 품질을 점검합니다:

#### **1단계: 코드 포맷팅**
```bash
cargo fmt
```
코드 스타일을 Rust 표준에 맞게 자동으로 정리합니다.

#### **2단계: 린팅 (코드 검사)**
```bash
cargo clippy
```
잠재적인 버그나 비효율적인 코드를 찾아서 개선 제안을 해줍니다.

#### **3단계: 테스트 실행**
```bash
cargo test
```
모든 테스트가 통과하는지 확인합니다.

💡 **모든 단계가 통과하면 Day 7 완료입니다!**

---

## [실전 활용 예시]

다음은 실제로 플래너를 사용하는 전체 흐름입니다:

```rust
use std::collections::BinaryHeap;
use day7_slot_planner::*;

fn main() {
    // 1. 메모리풀에서 트랜잭션 수집
    let mut mempool = BinaryHeap::new();
    mempool.push(MempoolEntry { compute_units: 100, fee_micro_lamports: 500 });
    mempool.push(MempoolEntry { compute_units: 200, fee_micro_lamports: 2000 });
    mempool.push(MempoolEntry { compute_units: 150, fee_micro_lamports: 1500 });

    // 2. 수수료 순으로 정렬
    let sorted_txs = drain_sorted_by_fee(&mut mempool);

    // 3. 블록 제약 설정
    let constraint = BlockConstraint {
        max_compute_units: 10000,
        max_transactions: 100,
    };

    // 4. 플래너로 최적의 번들 생성
    let mut planner = SlotPlanner::new(constraint);
    for tx in sorted_txs {
        if !planner.try_add(tx) {
            println!("트랜잭션 제한 초과로 건너뜀");
        }
    }

    // 5. 결과 확인
    let bundle = planner.finalize();
    println!("선택된 트랜잭션: {}개", bundle.entries.len());
    println!("총 컴퓨트: {}", bundle.total_compute_units);
    println!("총 수수료: {} micro-lamports", bundle.total_fee_micro_lamports);
}
```

💡 **핵심 패턴:**
1. 우선순위 큐로 정렬
2. 제약 조건 설정
3. 그리디 알고리즘으로 선택
4. 결과 확인

이것이 실제 Solana 검증자가 블록을 만드는 방식의 단순화 버전입니다!

---

## [오늘 배운 핵심 개념 정리]

### 🎯 그리디 알고리즘 (Greedy Algorithm)
- 매 순간 최선의 선택을 반복하여 전체 문제를 해결하는 방법
- 블록 번들링에서는 "수수료가 높은 트랜잭션부터 담기"가 그리디 전략

### 🔢 제약 조건 관리 (Constraint Management)
- 컴퓨트 유닛과 트랜잭션 개수라는 두 가지 제한을 동시에 관리
- `saturating_sub`로 안전한 산술 연산 구현

### 📊 우선순위 큐 활용 (Priority Queue Usage)
- `BinaryHeap`을 사용해 수수료 기준 정렬 자동화
- `Ord` 트레이트 구현으로 커스텀 비교 기준 설정

### 🧪 테스트 주도 개발 (TDD)
- 성공 케이스와 실패 케이스를 모두 테스트
- `assert!`와 `assert_eq!`로 예상 동작 검증

---

## [다음 단계 제안]

Day 7을 마쳤다면 다음 주제들을 고려해 보세요:

1. **최적화**: 더 복잡한 선택 알고리즘 (배낭 문제, 동적 프로그래밍)
2. **확장**: 계정 잠금(account locks) 고려하기
3. **실전**: Solana 실제 트랜잭션 데이터로 시뮬레이션
4. **성능**: 벤치마크 추가하여 처리 속도 측정

---

### 오늘의 TIL (Today I Learned)
*아래에 오늘 배운 내용이나 느낀 점을 자유롭게 작성해 보세요:*

- BinaryHeap
- PartialOrd, Ord 
- BlockConstraint, SlotPlanner

---

> **최종 점검 체크리스트:**
> ✅ `cargo fmt` - 코드 포맷팅
> ✅ `cargo clippy` - 코드 품질 검사
> ✅ `cargo test` - 모든 테스트 통과
>
> **축하합니다! Day 7 완료!** 🎉