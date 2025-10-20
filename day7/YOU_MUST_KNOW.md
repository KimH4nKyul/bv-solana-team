# YOU_MUST_KNOW Day 7

## 📚 핵심 개념 완벽 정리

### 🔒 슬롯과 블록 제약은 왜 생길까?

블록체인에서는 한 번에 처리할 수 있는 계산량과 트랜잭션 수가 제한됩니다.

#### 🎢 일상 비유: 놀이공원 롤러코스터

놀이공원 롤러코스터를 생각해 보세요:
- **최대 탑승 인원**: 한 번에 20명까지만 탈 수 있습니다
- **무게 제한**: 총 무게가 1,500kg을 넘으면 안전하지 않습니다
- **운행 시간**: 한 번 운행하는 데 5분이 걸립니다

블록체인의 슬롯도 똑같습니다:
- **트랜잭션 개수 제한**: 한 슬롯에 담을 수 있는 트랜잭션 수
- **컴퓨트 유닛 제한**: 총 계산량 한도
- **슬롯 시간**: Solana는 약 400ms마다 새 슬롯 생성

#### ⚡ Solana의 실제 제한

**컴퓨트 유닛(Compute Unit, CU)이란?**
- 트랜잭션이 사용하는 **계산 자원**을 숫자로 나타낸 단위입니다
- 예시:
  - 간단한 토큰 전송: 약 200,000 CU
  - 복잡한 스마트 컨트랙트 호출: 최대 1,400,000 CU
  - 한 슬롯의 최대 용량: **48,000,000 CU**

**왜 제한이 필요한가요?**
1. **네트워크 안정성**: 한 슬롯에 너무 많은 작업이 몰리면 검증자가 처리할 수 없습니다
2. **공정성**: 모든 사용자가 공평하게 블록 공간을 사용할 수 있어야 합니다
3. **예측 가능성**: 트랜잭션 처리 시간을 예측할 수 있어야 합니다

#### 📊 실제 사례

```
슬롯 #12345의 상황:
- 최대 CU: 48,000,000
- 현재 사용: 45,000,000 (94% 사용)
- 남은 용량: 3,000,000

새로운 트랜잭션 요청:
- 트랜잭션 A: 2,000,000 CU, 수수료 0.001 SOL ✅ 가능
- 트랜잭션 B: 5,000,000 CU, 수수료 0.01 SOL  ❌ 불가능 (용량 초과)

→ 트랜잭션 A는 이번 슬롯에, B는 다음 슬롯으로!
```

---

### 🎯 그리디(Greedy) 전략이란?

#### 기본 개념

**그리디 알고리즘**은 **매 순간 최선의 선택**을 반복하여 전체 문제를 해결하는 방법입니다.

#### 🍪 일상 비유: 쿠키 담기

상황: 가방에 쿠키를 최대한 많이 담고 싶은데, 가방의 무게 제한이 있습니다.

**전략 1: 그리디 (가벼운 것부터)**
```
쿠키들: [100g/500원, 200g/800원, 150g/600원]
가방 한도: 300g

선택 과정:
1. 100g 쿠키 담기 → 남은 용량: 200g ✅
2. 150g 쿠키 담기 → 남은 용량: 50g ✅
3. 200g 쿠키는 못 담음 (용량 초과) ❌

결과: 2개 쿠키, 총 1,100원
```

**전략 2: 최적해 (동적 프로그래밍)**
- 모든 조합을 시도해서 최고 가치를 찾음
- 시간이 더 걸리지만 최선의 결과 보장

#### 💰 블록 번들링에서의 그리디

**목표:** 수수료 수익 최대화

```rust
// 수수료가 높은 순서로 정렬된 트랜잭션들
트랜잭션들 = [
    { compute: 1000, fee: 5000 },  // 높은 수수료
    { compute: 800,  fee: 3000 },  // 중간 수수료
    { compute: 1500, fee: 2000 },  // 낮은 수수료
]

제한 = { max_compute: 2000, max_txs: 10 }

그리디 선택:
1️⃣ 첫 번째 추가 → compute: 1000/2000, fee: 5000 ✅
2️⃣ 두 번째 추가 → compute: 1800/2000, fee: 8000 ✅
3️⃣ 세 번째 불가 → compute 초과 (1800+1500 > 2000) ❌

최종 번들: 2개 트랜잭션, 총 수수료 8,000
```

#### ⚖️ 그리디의 장단점

**장점:**
- ✅ 구현이 간단합니다
- ✅ 속도가 빠릅니다 (O(n log n) - 정렬 시간)
- ✅ 메모리 사용량이 적습니다
- ✅ 실시간 결정에 적합합니다

**단점:**
- ❌ 항상 최적해를 보장하지는 않습니다
- ❌ "이미 담은 것을 빼고 더 좋은 조합"을 고려하지 않습니다

**언제 사용하나요?**
- 블록 생성처럼 **빠른 결정**이 필요한 경우
- "충분히 좋은 해"로 충분한 경우
- 최적해를 찾는 시간이 너무 오래 걸리는 경우

#### 🔄 실제 블록체인에서의 적용

```
검증자의 선택 과정:
1. 메모리풀에서 대기 중인 트랜잭션들 확인
2. 수수료 기준으로 정렬 (BinaryHeap 사용)
3. 제한 내에서 하나씩 추가 (그리디)
4. 블록 생성 및 전파

⏱️ 이 모든 과정이 400ms 안에 완료되어야 합니다!
→ 그래서 빠른 그리디 알고리즘을 사용합니다
```

---

### 🗂️ `BinaryHeap`은 어떻게 정렬을 도와줄까?

#### 힙(Heap) 자료구조 이해하기

**힙**은 특별한 규칙을 가진 트리 구조입니다.

#### 🌳 최대 힙의 구조

```
        [5000]              ← 루트: 가장 큰 값
        /    \
    [3000]  [2000]         ← 부모보다 작거나 같음
    /  \      /
[1000][800][500]           ← 자식도 부모보다 작거나 같음

규칙: 부모 노드가 항상 자식보다 크거나 같다
```

#### ⚡ 시간 복잡도 비교

| 작업 | BinaryHeap | 정렬된 Vec | 일반 Vec |
|------|------------|------------|----------|
| 최댓값 찾기 | O(1) | O(1) | O(n) |
| 삽입 | O(log n) | O(n) | O(1) |
| 최댓값 제거 | O(log n) | O(n) | O(n) |
| 전체 정렬 | O(n log n) | 이미 정렬됨 | O(n log n) |

**결론:** 우선순위가 가장 높은 것을 반복적으로 꺼낼 때 `BinaryHeap`이 최적!

#### 🔧 Rust에서의 BinaryHeap 사용

**1. 기본 사용법**

```rust
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::new();

// 삽입 (순서 상관없이)
heap.push(3000);
heap.push(5000);
heap.push(1000);

// 꺼내기 (자동으로 큰 값부터)
assert_eq!(heap.pop(), Some(5000));  // 가장 큰 값
assert_eq!(heap.pop(), Some(3000));  // 두 번째로 큰 값
assert_eq!(heap.pop(), Some(1000));  // 세 번째로 큰 값
assert_eq!(heap.pop(), None);        // 비어있음
```

**2. 커스텀 타입 정렬**

```rust
#[derive(Eq, PartialEq)]
pub struct MempoolEntry {
    pub compute_units: u32,
    pub fee_micro_lamports: u64,
}

// 수수료 기준으로 비교
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
```

이제 `BinaryHeap<MempoolEntry>`는 자동으로 수수료가 높은 순으로 정렬됩니다!

#### 🎯 왜 정렬된 Vec이 아닌 힙을 쓸까?

**시나리오: 실시간으로 트랜잭션이 계속 들어오는 상황**

```rust
// ❌ Vec + sort: 트랜잭션이 올 때마다 전체 정렬
let mut txs = Vec::new();
txs.push(new_tx);
txs.sort();  // O(n log n) - 매번 전체 정렬!

// ✅ BinaryHeap: 삽입만 하면 자동으로 우선순위 유지
let mut heap = BinaryHeap::new();
heap.push(new_tx);  // O(log n) - 훨씬 빠름!
```

**메모리풀의 실제 상황:**
- 초당 수천 개의 트랜잭션이 들어옵니다
- 동시에 블록에 담기 위해 꺼내갑니다
- `BinaryHeap`은 이런 실시간 상황에 최적화되어 있습니다!

#### 🔄 힙에서 정렬된 벡터 만들기

```rust
pub fn drain_sorted_by_fee(heap: &mut BinaryHeap<MempoolEntry>) -> Vec<MempoolEntry> {
    let mut sorted = Vec::new();
    while let Some(entry) = heap.pop() {
        sorted.push(entry);  // 큰 것부터 차례로
    }
    sorted  // [5000, 3000, 1000] - 내림차순 완성!
}
```

**장점:**
- 필요한 만큼만 `pop`할 수 있습니다 (전체를 꺼내지 않아도 됨)
- 메모리 효율적입니다
- 코드가 간결합니다

---

### ✅ `bool` 반환이 왜 편할까?

#### 간단함의 힘

`try_add` 함수의 반환 타입으로 `bool`을 선택한 이유를 알아봅시다.

**시그니처:**
```rust
pub fn try_add(&mut self, entry: MempoolEntry) -> bool
```

**의미:**
- `true` = "성공! 트랜잭션을 번들에 추가했습니다" ✅
- `false` = "실패! 제한을 초과하여 추가하지 않았습니다" ❌

#### 🔄 대안 비교

**옵션 1: `bool` (현재 방식)**
```rust
if planner.try_add(tx) {
    println!("추가 성공!");
} else {
    println!("제한 초과로 건너뜀");
}
```

**옵션 2: `Result<(), Error>` (더 상세한 정보)**
```rust
enum AddError {
    ComputeExceeded,
    TxCountExceeded,
}

match planner.try_add(tx) {
    Ok(()) => println!("추가 성공!"),
    Err(AddError::ComputeExceeded) => println!("컴퓨트 한도 초과"),
    Err(AddError::TxCountExceeded) => println!("트랜잭션 수 초과"),
}
```

**옵션 3: `Option<RejectionReason>` (실패 시 이유)**
```rust
match planner.try_add(tx) {
    None => println!("추가 성공!"),
    Some(reason) => println!("실패: {:?}", reason),
}
```

#### 🎯 언제 `bool`을 사용하나요?

**적합한 경우:**
- ✅ 성공/실패만 구분하면 충분할 때
- ✅ 실패 이유가 명확하거나 중요하지 않을 때
- ✅ 빠른 판단이 필요한 루프 안에서
- ✅ 간단한 프로토타입이나 학습 코드

**부적합한 경우:**
- ❌ 실패 원인을 구체적으로 알아야 할 때
- ❌ 다양한 에러 타입을 처리해야 할 때
- ❌ 로깅이나 디버깅이 중요한 프로덕션 코드

---

### 🔁 반복자(Iterator)를 활용하면 뭐가 좋을까?

#### 반복자란?

**반복자**는 컬렉션의 항목을 **하나씩 순회**하는 표준화된 방법입니다.

#### 📦 일상 비유: 선물 상자

```
선물 상자 = 트랜잭션 컬렉션
선물 하나씩 꺼내기 = 반복자

for 선물 in 상자 {
    확인하기(선물);
}
```

#### 🔧 실전 활용

```rust
// 방법 1: for 루프 (내부적으로 Iterator 사용)
for tx in transactions {
    planner.try_add(tx);
}

// 방법 2: Iterator 메서드 체이닝
let added_count = transactions
    .into_iter()
    .filter(|tx| planner.try_add(tx))
    .count();

// 방법 3: IntoIterator 제네릭
impl SlotPlanner {
    pub fn try_add_many<I>(&mut self, txs: I) -> usize
    where
        I: IntoIterator<Item = MempoolEntry>
    {
        txs.into_iter()
           .filter(|tx| self.try_add(tx.clone()))
           .count()
    }
}

// 다양한 타입으로 호출 가능!
planner.try_add_many(vec![tx1, tx2]);        // Vec
planner.try_add_many(&[tx1, tx2]);           // 슬라이스
planner.try_add_many(heap.into_iter());       // BinaryHeap
```

**장점:**
- 코드 재사용성 증가
- 다양한 컬렉션 타입 지원
- 함수형 프로그래밍 스타일

---

### 📊 남은 용량 계산은 왜 중요할까?

#### 실시간 모니터링

```rust
let (remaining_cu, remaining_txs) = bundle.remaining_capacity(&constraint);

println!("남은 컴퓨트: {}", remaining_cu);
println!("더 담을 수 있는 트랜잭션: {}", remaining_txs);
```

**활용 예시:**

**1. 조기 종료 최적화**
```rust
for tx in sorted_txs {
    let (remaining_cu, remaining_txs) = bundle.remaining_capacity(&constraint);

    // 남은 용량이 0이면 더 시도할 필요 없음
    if remaining_cu == 0 || remaining_txs == 0 {
        break;  // 조기 종료!
    }

    planner.try_add(tx);
}
```

**2. 진행률 표시**
```rust
let usage_percent = (bundle.total_compute_units as f64
    / constraint.max_compute_units as f64) * 100.0;
println!("번들 사용률: {:.1}%", usage_percent);
```

**3. 로깅 및 디버깅**
```rust
println!("현재 상태: {}/{} CU, {}/{} txs",
    bundle.total_compute_units,
    constraint.max_compute_units,
    bundle.entries.len(),
    constraint.max_transactions
);
```

---

### 🛡️ `saturating_sub`는 어떤 역할을 할까?

#### 안전한 빼기 연산

**문제 상황:**
```rust
let x: u32 = 5;
let y: u32 = 10;
let z = x - y;  // 💥 패닉! (디버그 모드)
                // ⚠️ 언더플로우! (릴리즈 모드 - 잘못된 값)
```

**해결책:**
```rust
let x: u32 = 5;
let y: u32 = 10;
let z = x.saturating_sub(y);  // ✅ 0 (안전!)
```

#### 🔢 작동 방식

```rust
// 일반 빼기
5u32 - 10  // 패닉 또는 래핑(4294967291)

// saturating_sub
5u32.saturating_sub(10)   // 0 ← 음수 대신 최솟값
100u32.saturating_sub(30)  // 70 ← 정상 동작
```

#### 💡 실전 예시

```rust
pub fn remaining_capacity(&self, constraint: &BlockConstraint) -> (u32, usize) {
    // 만약 버그로 total이 max보다 크다면?
    // 일반 빼기: 패닉 또는 이상한 큰 수
    // saturating_sub: 안전하게 0 반환

    let remaining_compute = constraint.max_compute_units
        .saturating_sub(self.total_compute_units);

    let remaining_transactions = constraint.max_transactions
        .saturating_sub(self.entries.len());

    (remaining_compute, remaining_transactions)
}
```

#### 🛡️ 안전성의 가치

**프로덕션 환경에서:**
- 예상치 못한 상황에서도 프로그램이 죽지 않습니다
- 잘못된 입력에 대한 방어 코드가 됩니다
- 디버깅이 쉬워집니다 (0이면 문제가 있다는 신호)

---

## 📝 오늘의 핵심 정리

### 🎓 배운 것들

| 개념 | 핵심 내용 | 실전 적용 |
|------|----------|----------|
| **슬롯 제약** | 컴퓨트 & 트랜잭션 개수 한도 | 모든 추가 전 검증 필수 |
| **그리디 알고리즘** | 매 순간 최선의 선택 반복 | 빠른 블록 생성에 최적 |
| **BinaryHeap** | 최댓값 O(1)로 접근 | 수수료 우선순위 큐 |
| **bool 반환** | 간단한 성공/실패 신호 | 단순한 API 설계 |
| **Iterator** | 다형적 컬렉션 순회 | 코드 재사용성 증가 |
| **saturating_sub** | 안전한 빼기 연산 | 언더플로우 방지 |

### 🔗 개념 간 연결

```
메모리풀 트랜잭션들
        ↓
   BinaryHeap (정렬)
        ↓
수수료 높은 순으로 나열
        ↓
   그리디 알고리즘
        ↓
제약 조건 검증 (saturating_sub)
        ↓
   bool 반환 (추가 성공 여부)
        ↓
    최종 번들 완성
```

### 💡 실무 인사이트

**Day 7에서 만든 것은 실제 블록체인의:**
- 📦 **번들 빌더(Bundle Builder)** - MEV(Maximal Extractable Value) 추출 도구
- ⚡ **검증자(Validator)** - 블록 생성의 핵심 로직
- 🎯 **우선순위 시스템** - 제한된 자원의 효율적 배분

**왜 그리디를 쓸까?**
- Solana는 **400ms마다 새 슬롯** 생성
- 최적해를 찾는 동적 프로그래밍은 너무 느림
- 그리디는 **"충분히 좋은 해"**를 빠르게 제공

### 🚀 다음 단계

Day 7을 마스터했다면 다음 주제를 탐구해 보세요:

1. **계정 잠금(Account Locks)**
   - 여러 트랜잭션이 같은 계정을 건드리는 경우 처리

2. **동적 우선순위**
   - 시간에 따라 변하는 우선순위 (수수료 + 대기 시간)

3. **번들 최적화**
   - 배낭 문제(Knapsack) 알고리즘으로 더 높은 수익

4. **병렬 처리**
   - 여러 번들을 동시에 계획하기

---

## 🎯 학습 완료 체크리스트

스스로 확인해 보세요:

- [ ] 슬롯과 컴퓨트 유닛의 개념을 다른 사람에게 설명할 수 있다
- [ ] 그리디 알고리즘의 장단점을 3가지씩 말할 수 있다
- [ ] `BinaryHeap`의 시간 복잡도를 이해한다 (push: O(log n), pop: O(log n))
- [ ] `saturating_sub`가 일반 빼기와 어떻게 다른지 예시를 들 수 있다
- [ ] `try_add`가 `false`를 반환하는 두 가지 경우를 말할 수 있다
- [ ] 실제 Solana에서 이 개념들이 어떻게 사용되는지 이해했다

**모두 체크했다면 Day 7 완벽 마스터! 🎉**

---

## 🌟 마지막 한마디

> "오늘 만든 SlotPlanner는 단순해 보이지만, 실제 블록체인에서 초당 수천 건의 트랜잭션을 처리하는 핵심 로직입니다. 여러분은 이제 블록체인의 '심장'이 어떻게 뛰는지 이해하게 되었습니다!"

**축하합니다! 🎊 이제 Day 8로 넘어갈 준비가 되었습니다!**