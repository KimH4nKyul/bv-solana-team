# Day 11: Reth 스타일 트랜잭션 풀 우선순위 큐 구현

**난이도: MEDIUM (트랜잭션 스케줄링)**

## [전날 과제 요약]
- Day 10에서는 포크 선택기를 확장해 total difficulty가 더 큰 체인을 canonical로 재구성하는 로직을 완성했습니다.
- 헤더를 그래프로 저장하고, reorg 깊이를 계산해 체인 롤백 상황을 감지할 수 있게 되었습니다.
- 에러/결과 타입을 명시적으로 설계하며 Reth의 HeaderAccumulator 개념을 모사했습니다.

## [전날 과제로 얻은 역량]
- 블록 헤더 그래프 상태를 HashMap과 Vec으로 모델링하는 감각을 익혔습니다.
- 체인 동기화에서 canonical head를 유지하기 위한 재구성 절차를 이해했습니다.
- 테스트 주도로 포크 확장/유지/재구성 시나리오를 검증하는 습관을 들였습니다.

## [오늘 과제 목표]
- Reth의 TxPool이 트랜잭션을 우선순위 기반으로 정렬하는 핵심 개념을 체험합니다.
- 동일한 sender의 nonce 순서를 보장하면서도 전역 우선순위를 유지하는 자료구조를 설계합니다.
- 트랜잭션 삽입, 대기열 포화 시 축출(eviction), 배치(pop) 로직을 테스트 주도 방식으로 구현합니다.

## [오늘 과제 설명]
Reth TxPool은 각 계정(sender)의 nonce 순서를 깨지 않으면서도, 전역적으로 높은 가스 가격(gas_price)을 우선 처리하도록 설계되어 있습니다. 오늘은 메모리 기반 간단한 트랜잭션 풀을 만들어 이 개념을 연습합니다. 계정별로 정렬된 큐를 유지하고, 전역 우선순위 큐로 head 트랜잭션만 비교하며 풀 크기를 제한합니다.

1. **프로젝트 생성 및 기본 뼈대**
    - `cargo new day11_tx_pool --lib` 명령으로 라이브러리 크레이트를 생성합니다.
    - 모든 로직은 `src/lib.rs` 에 작성하며, 학습자가 직접 구현할 공간은 비워 둡니다.
    - 단위 테스트 스켈레톤은 `src/lib.rs` 하단 `#[cfg(test)] mod tests` 내부에 추가합니다. (통합 테스트 디렉터리는 사용하지 않습니다.)

2. **자료구조 및 불변식 정의 (`src/lib.rs`)**
    - 아래 표에 맞춰 구조체를 설계하고, 각 필드 위에 표 내용에 해당하는 한글 주석을 그대로 서술합니다.

      | 엔티티 | 필드 | 의미 | 필수 불변식 |
      | --- | --- | --- | --- |
      | `PendingTransaction` | `hash` | 트랜잭션 식별자 | 고유 식별자임을 주석으로 명시 |
      |  | `sender` | 서명 계정 ID | 동일 sender 묶음은 `per_account` 하나만 사용 |
      |  | `nonce` | 계정별 실행 순서 | 같은 sender에서 중복 금지 |
      |  | `gas_price` | 전역 우선순위 보조 지표 | 음수 불가(부호 없는 정수 유지) |
      |  | `priority` | reputation/tip 등 가중치 | 가스 가격과 독립적으로 비교 |
      | `QueuedTx` | `key` | `(priority, gas_price)` 캐시 | BinaryHeap 비교용, 항상 `tx.priority_key()`와 동기화 |
      |  | `tx` | 실제 대기 트랜잭션 | clone 유지, 외부에서 소유권 공유 금지 |
      | `TxPoolConfig` | `capacity` | 풀 전체 허용 개수 | 0 이상, 음수 불가 |
      |  | `max_account_slots` | 계정별 허용 개수 | 0 이상, 0이면 신규 계정 추가 금지 |
      | `TxPool` | `config` | 현재 설정 | 생성 이후 변경 금지 |
      |  | `per_account` | sender → VecDeque | 항상 nonce 오름차순 정렬 |
      |  | `global_queue` | BinaryHeap<QueuedTx> | 각 계정의 front만 존재 |
      |  | `total_txs` | 현재 보유 수 | per_account 총합과 동일해야 함 |

    - `PendingTransaction::priority_key()`는 `(priority, gas_price)`를 반환하며, 함수 위 주석으로 "priority 우선 비교, 동률 시 gas_price 비교" 이유를 명시합니다.
    - `QueuedTx`는 `#[derive(Clone, Eq, PartialEq)]`로 파생하고, `Ord`/`PartialOrd` 구현 시 다음 순서를 지킵니다.
        1. `other.key.cmp(&self.key)` 형태로 최대 힙을 유지.
        2. key 동률 시 `tx.nonce`가 작은 쪽(= 더 빠른 논스)을 먼저 선택.
        3. 구현 본문 첫 줄에 `// Reth는 max-heap을 사용해 가장 높은 우선순위를 먼저 스케줄합니다.` 주석 추가.

3. **에러/결과 타입 문서화 (`src/lib.rs`)**
    - `TxInsertError` 변형마다 다음 정보를 주석으로 남깁니다.
        - `DuplicateNonce`: "동일 sender에서 이미 존재하는 nonce 재삽입" 사례를 예시와 함께 기술.
        - `AccountLimitReached`: "계정별 슬롯 상한 초과 시점" + `max_account_slots`가 0일 때 신규 계정 거부 조건을 명시.
        - `PoolFull`: "capacity 도달 이후 축출까지 수행했으나 해당 삽입은 거부"라고 설명.
    - `PopResult::Batch` 주석에는 "배치 스케줄링 반환, drained는 priority_key 내림차순"을 명시하고, `Empty`는 "꺼낼 항목 없음" 설명을 추가합니다.

4. **핵심 메서드 명세 (`impl TxPool`)**
    - 아래 표를 `README` 참고용으로 그대로 구현 전 주석에 옮겨 적습니다.

      | 메서드 | 성공 조건 | 실패 조건 | 상태 변화 | 후속 처리 |
      | --- | --- | --- | --- | --- |
      | `new` | `per_account`/`global_queue` 비우고 `total_txs=0` | 없음 | 모든 필드 초기화 | 없음 |
      | `insert` | 계정 큐 유지 + `total_txs` 증가 | `DuplicateNonce`, `AccountLimitReached`, `PoolFull` | 새 트랜잭션을 nonce 오름차순으로 삽입, front 변경 시 global_queue 갱신 | 필요 시 lazy eviction 트리거 |
      | `pop_batch` | drained 길이 ≤ limit, priority 내림차순 | ( 반환값 `Empty` ) | per_account/gq에서 항목 제거, total_txs 감소 | 동일 sender 후속 nonce를 global_queue에 재등록 |
      | `evict_lowest_priority` | 최소 우선순위 1건 제거 | 유효 트랜잭션 없으면 조용히 종료 | per_account에서 해당 항목 제거, total_txs 감소 | front가 바뀐 계정은 global_queue에 재등록 |

    - `insert`와 `pop_batch`는 아래 의사코드를 참고해 단계별로 구현합니다.

      ```text
      insert(tx):
        sender_queue = per_account.entry(sender).or_insert(VecDeque::new)
        if sender_queue contains tx.nonce -> Err DuplicateNonce
        if sender_queue.len == max_account_slots -> Err AccountLimitReached
        if total_txs == capacity:
            evict_lowest_priority()
            return Err PoolFull
        remember old_front_hash = sender_queue.front()
        sender_queue에 nonce 기준으로 위치 찾아 삽입
        total_txs += 1
        if sender_queue.front() changed:
            global_queue.push(QueuedTx::new(front.clone()))
        Ok(())
      ```

      ```text
      pop_batch(limit):
        if limit == 0 -> Empty
        drained = []
        while drained.len < limit:
            candidate = global_queue.pop() 없으면 break
            if candidate not match per_account front (stale):
                continue
            queue = per_account.get_mut(sender)
            front_tx = queue.pop_front()
            drained.push(front_tx)
            total_txs -= 1
            if queue.empty -> per_account.remove(sender)
            else -> global_queue.push(queue.front().clone())
        drained.is_empty ? Empty : Batch { drained }
      ```

    - `evict_lowest_priority()`는 lazy eviction을 문서화한 순서(힙 pop → stale 판별 → per_account 제거 → total_txs 감소 → front 재등록)를 그대로 주석으로 작성하고, 코드에서도 순서를 지킵니다.

5. **stale 항목 판별 기준**
    - README와 코드 주석에 다음 체크리스트를 복사합니다.
        1. `per_account.get(&queued.tx.sender)`가 `None`이면 stale.
        2. 큐가 존재하더라도 `front()`가 같은 `hash`·`nonce`가 아니면 stale.
        3. stale이면 현재 인터레이션을 `continue`로 건너뛰고 다음 힙 항목을 확인.

6. **단위 테스트 스켈레톤 작성 (`src/lib.rs`의 `mod tests`)**
    - 각 테스트는 아래 표의 Given/When/Then 시나리오와 예상 어서션을 주석으로 명시합니다. (실제 assert는 학습자가 작성)

      | 테스트 이름 | Given | When | Then (필수 어서션) |
      | --- | --- | --- | --- |
      | `new_pool_starts_empty` | 기본 config | `TxPool::new` 호출 | `total_txs == 0`, `per_account.is_empty()`, `global_queue.is_empty()` |
      | `insert_orders_nonce_within_account` | 동일 sender nonce 0/2/1 | 순서대로 insert | per_account 큐 nonce `[0,1,2]` |
      | `insert_updates_global_queue_head_only` | 동일 sender nonce 0,1 | 두 번 insert | global_queue.len == 1, head hash == nonce 0 |
      | `insert_rejects_duplicate_nonce` | nonce 0 사전 삽입 | nonce 0 재삽입 | `Err DuplicateNonce`, 큐 길이 유지 |
      | `insert_respects_account_slot_limit` | max_account_slots=2 | 세 번째 insert 시도 | `Err AccountLimitReached`, 큐 길이 2 |
      | `insert_evicts_lowest_priority_when_full` | capacity=3, 우선순위 다양한 tx | 4번째 insert | `Err PoolFull`, 낮은 우선순위 hash 제거 |
      | `pop_batch_drains_highest_priority_first` | 서로 다른 sender 우선순위 | `pop_batch(limit=3)` | drained priority_key 내림차순 |
      | `pop_batch_refreshes_next_nonce_head` | 한 sender nonce 연속 + 타 sender | `pop_batch(limit>=2)` | drained[0]이 전역 최고 우선순위, 동일 sender의 다음 nonce가 head |
      | `lazy_eviction_skips_stale_entries_on_pop` | evict 후 stale 남긴 상태 | `pop_batch` | drained에 stale 없음, 구조 일관성 유지 |

    - 주석 포맷은 반드시 `// Given: ...`, `// When: ...`, `// Then: ...` 형태를 사용하고, 특정 필드와 예상 값(예: `assert_eq!(pool.total_txs, 0)`)까지 주석에 기재합니다.
    - 테스트는 TDD를 위해 실패 상태로 시작해야 하므로, 학습자가 주석을 코드로 바꾸기 전에는 컴파일되지 않거나 panic이 발생하도록 일부 `todo!()`/`unimplemented!()` 등을 활용할 수 있다고 안내합니다.

7. **마무리 루틴 안내**
    - README 마지막에 학습자가 실행해야 할 명령을 아래 순서대로 안내합니다.
        - `cargo fmt`
        - `cargo clippy`
        - `cargo test`

## [이해를 돕기 위한 예시]
아래는 sender별 큐의 프런트만 전역 큐에 노출하는 로직 예시입니다. 실제 구현 시 주석을 적절히 보강해 주세요.

```rust
fn refresh_global_queue(
    sender: &str,
    per_account: &HashMap<String, VecDeque<PendingTransaction>>,
    global_queue: &mut BinaryHeap<QueuedTx>,
) {
    if let Some(queue) = per_account.get(sender) {
        if let Some(front) = queue.front() {
            // BinaryHeap은 max-heap이므로 높은 priority_key가 먼저 나옵니다.
            global_queue.push(QueuedTx::new(front.clone()));
        }
    }
}
```

- 이 패턴은 Reth TxPool이 계정별로 정렬된 큐를 유지하면서 전역적으로 헤드 트랜잭션만 비교하는 방식을 단순화한 것입니다.
- lazy eviction은 Reth가 내부적으로 사용하는 전략으로, BinaryHeap에서 제거된 항목이 실제 상태와 불일치하면 pop 시점에 무시하여 일관성을 유지합니다.
- priority_key를 튜플로 정의하면 가스 가격이 동일할 때 priority(예: reputation score)로 추가 정렬이 가능합니다.

---

### 오늘의 TIL (Today I Learned)
- 트랜잭션 풀에서 계정별 순서와 전역 우선순위를 동시에 유지하는 패턴을 구현했습니다.

> 마무리 전: `cargo fmt` → `cargo clippy` → `cargo test`
