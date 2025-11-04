use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

/// 트랜잭션 삽입이 실패했을 때 호출자에게 사유를 돌려주기 위한 에러 타입입니다.
#[derive(Debug)]
pub enum TxInsertError {
    /// 동일 sender에서 이미 존재하는 nonce 재삽입 예: `alice`가 nonce 3을 이미 올렸는데 다시 nonce 3을 제출한 경우.
    DuplicateNonce { sender: String, nonce: u64 },
    /// 계정별 슬롯 상한 초과 시점 + `max_account_slots`가 0이라 신규 계정 자체가 허용되지 않는 상황도 포함합니다.
    AccountLimitReached { sender: String },
    /// capacity 도달 이후 축출까지 수행했으나 해당 삽입은 거부해야 함을 나타냅니다.
    PoolFull,
}

/// 배치 팝 연산 결과입니다.
pub enum PopResult {
    /// 배치 스케줄링 반환, drained는 priority_key 내림차순이 되도록 정렬되어 있습니다.
    Batch { drained: Vec<PendingTransaction> },
    /// 꺼낼 항목 없음.
    Empty,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTransaction {
    /// 트랜잭션 식별자(고유해야 함)로 lazy eviction 비교와 중복 감지를 모두 여기서 수행합니다.
    pub hash: String,
    /// 서명 계정 ID, 동일 sender 묶음은 TxPool::per_account 하나만 사용합니다.
    pub sender: String,
    /// 계정별 실행 순서를 보장하는 nonce, 같은 sender에서 중복을 허용하지 않습니다.
    pub nonce: u64,
    /// 전역 우선순위 보조 지표로 쓰이는 gas_price, 음수 불가이므로 부호 없는 정수만 저장합니다.
    pub gas_price: u64,
    /// reputation/tip 등 가중치, 가스 가격과 독립적으로 비교합니다.
    pub priority: u128,
}

impl PendingTransaction {
    /// priority 우선 비교, 동률 시 gas_price 비교를 수행하도록 `(priority, gas_price)` 튜플을 반환합니다.
    pub fn priority_key(&self) -> (u128, u64) {
        // priority는 reputation 개념이라 더 큰 값이 우선이라면 첫 번째 요소에서 비교합니다.
        // gas_price는 동률일 때 보조 지표이므로 두 번째 요소로 비교합니다.
        (self.priority, self.gas_price)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedTx {
    /// BinaryHeap 비교용 `(priority, gas_price)` 캐시이며 항상 `tx.priority_key()`와 동기화합니다.
    pub key: (u128, u64),
    /// 실제 대기 트랜잭션이며 내부에서 clone을 유지하고 외부에서 소유권을 공유하지 않습니다.
    pub tx: PendingTransaction,
}

impl QueuedTx {
    /// front 트랜잭션을 우선순위 큐에 넣기 전에 key 캐시를 만들어 둡니다.
    pub fn new(tx: PendingTransaction) -> Self {
        // priority_key 호출은 필수 불변식(캐시 동기화)을 지키기 위해 new 시점에만 수행합니다.
        let key = tx.priority_key();
        Self { key, tx }
    }
}

impl Ord for QueuedTx {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reth는 max-heap을 사용해 가장 높은 우선순위를 먼저 스케줄합니다.
        let by_priority = other.key.cmp(&self.key);
        // priority, gas_price가 모두 동일할 때는 더 낮은 nonce(= 먼저 실행 가능한 항목)를 우선시합니다.
        if by_priority == Ordering::Equal {
            return other.tx.nonce.cmp(&self.tx.nonce);
        }
        by_priority.reverse()
    }
}

impl PartialOrd for QueuedTx {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // BinaryHeap 요구사항에 맞춰 전체 순서를 정의하므로 그대로 cmp만 위임합니다.
        Some(self.cmp(other))
    }
}

pub struct TxPoolConfig {
    /// 풀 전체 허용 개수(capacity)이며 0 이상, 음수는 불가능합니다.
    pub capacity: usize,
    /// 계정별 허용 개수(max_account_slots)이며 0 이상, 0이면 신규 계정을 추가할 수 없습니다.
    pub max_account_slots: usize,
}

pub struct TxPool {
    /// 현재 설정으로, 생성 이후에는 변경하지 않는 불변 데이터를 저장합니다.
    config: TxPoolConfig,
    /// sender → VecDeque 매핑이며 각 큐는 항상 nonce 오름차순으로 정렬되어야 합니다.
    per_account: HashMap<String, VecDeque<PendingTransaction>>,
    /// BinaryHeap<QueuedTx> 구조로 각 계정의 front만 유지합니다.
    global_queue: BinaryHeap<QueuedTx>,
    /// 풀에서 보유 중인 트랜잭션 수로, per_account 전체 길이 합과 동일해야 합니다.
    total_txs: usize,
}

// 메서드 명세 요약:
// | 메서드 | 성공 조건 | 실패 조건 | 상태 변화 | 후속 처리 |
// | --- | --- | --- | --- | --- |
// | new | per_account/global_queue 비우고 total_txs=0 | 없음 | 모든 필드 초기화 | 없음 |
// | insert | 계정 큐 유지 + total_txs 증가 | DuplicateNonce, AccountLimitReached, PoolFull | nonce 오름차순 삽입, front 변경 시 global_queue 갱신 | 필요 시 lazy eviction 트리거 |
// | pop_batch | drained 길이 ≤ limit, priority 내림차순 | 반환 Empty | per_account/global_queue에서 제거, total_txs 감소 | 동일 sender 후속 nonce front 재등록 |
// | evict_lowest_priority | 최소 우선순위 1건 제거 | 유효 트랜잭션 없으면 조용히 종료 | per_account에서 제거, total_txs 감소 | front 변경 시 global_queue 재등록 |
impl TxPool {
    /// config만 받아 초기 상태를 구성합니다.
    pub fn new(config: TxPoolConfig) -> Self {
        // 초기화 시 빈 HashMap/Heap/카운터를 만들어 표의 성공 조건을 만족시킵니다.
        Self {
            config,
            per_account: HashMap::new(),
            global_queue: BinaryHeap::new(),
            total_txs: 0,
        }
    }

    /// 트랜잭션을 pool에 삽입하고 필요 시 lazy eviction을 트리거합니다.
    pub fn insert(&mut self, tx: PendingTransaction) -> Result<(), TxInsertError> {
        // sender는 HashMap key이자 오류 메시지에 필요하므로 선 복사합니다.
        let sender = tx.sender.clone();

        // 기존 큐 존재 여부를 조사해 중복 nonce와 계정별 슬롯 제한을 확인합니다.
        if let Some(queue) = self.per_account.get(&sender) {
            // 같은 sender에서 같은 nonce가 발견되면 DuplicateNonce를 즉시 반환합니다.
            if queue.iter().any(|existing| existing.nonce == tx.nonce) {
                return Err(TxInsertError::DuplicateNonce {
                    sender: sender.clone(),
                    nonce: tx.nonce,
                });
            }
            // 계정별 슬롯 상한을 초과하면 AccountLimitReached를 반환합니다.
            if queue.len() >= self.config.max_account_slots {
                return Err(TxInsertError::AccountLimitReached {
                    sender: sender.clone(),
                });
            }
        } else if self.config.max_account_slots == 0 {
            // 신규 sender인데 계정 슬롯이 0이면 즉시 거부합니다.
            return Err(TxInsertError::AccountLimitReached { sender });
        }

        // capacity에 도달했다면 축출을 먼저 시도하고, 새 트랜잭션은 거부(PoolFull)합니다.
        if self.total_txs >= self.config.capacity {
            self.evict_lowest_priority();
            return Err(TxInsertError::PoolFull);
        }

        // front 비교를 위해 삽입 전의 해시를 저장합니다.
        let previous_front_hash = self
            .per_account
            .get(&sender)
            .and_then(|queue| queue.front().map(|front| front.hash.clone()));

        // VecDeque를 준비하고 nonce 기준으로 올바른 위치에 삽입합니다.
        let queue = self
            .per_account
            .entry(sender.clone())
            .or_insert_with(VecDeque::new);
        let insert_pos = queue
            .iter()
            .position(|existing| existing.nonce > tx.nonce)
            .unwrap_or(queue.len());
        queue.insert(insert_pos, tx);

        // 총 트랜잭션 수를 갱신합니다. (per_account 합과 동일해야 함)
        self.total_txs += 1;

        // front가 바뀌었으면 BinaryHeap에 새 head를 푸시합니다(기존 head는 lazy eviction으로 제거).
        let new_front_hash = queue.front().map(|front| front.hash.clone());
        if new_front_hash != previous_front_hash {
            if let Some(front) = queue.front().cloned() {
                self.global_queue.push(QueuedTx::new(front));
            }
        }

        Ok(())
    }

    /// lazy eviction으로 최소 우선순위 트랜잭션을 하나 제거합니다.
    fn evict_lowest_priority(&mut self) {
        // lazy eviction 순서: 힙 pop → stale 판별 → per_account 제거 → total_txs 감소 → front 재등록
        let mut buffer: Vec<QueuedTx> = Vec::new();
        while let Some(queued) = self.global_queue.pop() {
            // stale 체커 체크리스트:
            // 1. per_account.get(&queued.tx.sender)가 None이면 stale.
            // 2. 큐가 존재해도 front()가 동일 hash·nonce가 아니면 stale.
            // 3. stale이면 continue로 건너뛰고 다음 힙 항목 확인.
            let sender = queued.tx.sender.clone();
            let hash = queued.tx.hash.clone();
            let nonce = queued.tx.nonce;

            let is_stale = match self.per_account.get(sender.as_str()) {
                None => true,
                Some(queue) => match queue.front() {
                    Some(front) if front.hash == hash && front.nonce == nonce => false,
                    _ => true,
                },
            };
            if is_stale {
                continue;
            }
            buffer.push(queued);
        }

        // 유효한 front가 없으면 축출할 것이 없으니 종료합니다.
        if buffer.is_empty() {
            // 유효한 항목이 하나도 없었다면 조용히 종료합니다.
            return;
        }

        // 우선순위 키가 가장 작은(= 최저 우선순위) 항목을 찾습니다.
        let mut lowest_idx = 0usize;
        for idx in 1..buffer.len() {
            let current = &buffer[idx];
            let lowest = &buffer[lowest_idx];
            let is_lower_priority = current.key < lowest.key
                || (current.key == lowest.key && current.tx.nonce > lowest.tx.nonce);
            if is_lower_priority {
                lowest_idx = idx;
            }
        }

        let lowest = buffer.swap_remove(lowest_idx);

        // 나머지 front는 다시 힙으로 되돌립니다.
        for queued in buffer {
            self.global_queue.push(queued);
        }

        let sender = lowest.tx.sender.clone();
        let mut remove_sender = false;
        let mut new_front: Option<PendingTransaction> = None;

        if let Some(queue) = self.per_account.get_mut(sender.as_str()) {
            // front가 반드시 동일해야 하므로 pop_front로 제거합니다.
            let removed = queue.pop_front();
            if removed.is_some() && self.total_txs > 0 {
                self.total_txs -= 1;
            }

            if queue.is_empty() {
                // 큐가 비면 sender를 HashMap에서 제거합니다.
                remove_sender = true;
            } else {
                new_front = queue.front().cloned();
            }
        }

        if remove_sender {
            self.per_account.remove(sender.as_str());
        } else if let Some(front) = new_front {
            // front가 바뀌었으므로 새 head를 힙에 등록합니다.
            self.global_queue.push(QueuedTx::new(front));
        }

        // 최저 우선순위 하나를 제거했으므로 작업 종료입니다.
    }

    /// 우선순위가 높은 항목부터 최대 `limit`개를 배치로 꺼냅니다.
    pub fn pop_batch(&mut self, limit: usize) -> PopResult {
        // 0을 요청하면 비어 있는 결과를 돌려줘 호출자가 실수로 0을 넣어도 혼란이 없습니다.
        if limit == 0 {
            return PopResult::Empty;
        }

        // drained 벡터에 순차적으로 모아 나중에 그대로 돌려줍니다.
        let mut drained = Vec::new();

        while drained.len() < limit {
            // 힙에서 후보를 꺼내고 없으면 반복을 종료합니다.
            let Some(queued) = self.global_queue.pop() else {
                break;
            };

            // stale 체크 (README 체크리스트와 동일합니다).
            // 1. per_account.get(&queued.tx.sender)가 None이면 stale.
            // 2. front()의 hash/nonce가 일치하지 않으면 stale.
            // 3. stale이면 continue.
            let sender = queued.tx.sender.clone();
            let hash = queued.tx.hash.clone();
            let nonce = queued.tx.nonce;

            let is_stale = match self.per_account.get(sender.as_str()) {
                None => true,
                Some(queue) => match queue.front() {
                    Some(front) if front.hash == hash && front.nonce == nonce => false,
                    _ => true,
                },
            };
            if is_stale {
                continue;
            }

            // front를 제거하고 drained에 담은 뒤, 다음 nonce를 힙에 재등록합니다.
            let mut remove_sender = false;
            let mut new_front: Option<PendingTransaction> = None;
            let mut drained_tx: Option<PendingTransaction> = None;

            if let Some(queue) = self.per_account.get_mut(sender.as_str()) {
                let front = queue
                    .pop_front()
                    .expect("lazy eviction에서 stale이 아닌 front는 반드시 존재해야 합니다");
                drained_tx = Some(front);

                if self.total_txs > 0 {
                    self.total_txs -= 1;
                }

                if queue.is_empty() {
                    remove_sender = true;
                } else {
                    new_front = queue.front().cloned();
                }
            }

            if remove_sender {
                self.per_account.remove(sender.as_str());
            } else if let Some(front) = new_front {
                self.global_queue.push(QueuedTx::new(front));
            }

            if let Some(tx) = drained_tx {
                drained.push(tx);
            }
        }

        if drained.is_empty() {
            PopResult::Empty
        } else {
            // drained가 priority_key 내림차순이 되도록 정렬합니다.
            drained.sort_by(|a, b| {
                b.priority_key()
                    .cmp(&a.priority_key())
                    .then(a.nonce.cmp(&b.nonce))
            });
            PopResult::Batch { drained }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tx(
        sender: &str,
        nonce: u64,
        priority: u128,
        gas_price: u64,
        tag: &str,
    ) -> PendingTransaction {
        PendingTransaction {
            hash: format!("{sender}-{nonce}-{tag}"),
            sender: sender.to_string(),
            nonce,
            gas_price,
            priority,
        }
    }

    #[test]
    fn new_pool_starts_empty() {
        // Given: 기본 config
        let config = TxPoolConfig {
            capacity: 16,
            max_account_slots: 8,
        };
        // When: TxPool::new 호출
        let pool = TxPool::new(config);
        // Then: total_txs == 0, per_account.is_empty(), global_queue.is_empty()
        assert_eq!(pool.total_txs, 0);
        assert!(pool.per_account.is_empty());
        assert!(pool.global_queue.is_empty());
    }

    #[test]
    fn insert_orders_nonce_within_account() {
        // Given: 동일 sender nonce 0/2/1
        let config = TxPoolConfig {
            capacity: 10,
            max_account_slots: 10,
        };
        let mut pool = TxPool::new(config);
        // When: 순서대로 insert
        pool.insert(make_tx("alice", 0, 10, 100, "a"))
            .expect("first insert");
        pool.insert(make_tx("alice", 2, 8, 90, "b"))
            .expect("second insert");
        pool.insert(make_tx("alice", 1, 9, 95, "c"))
            .expect("third insert");
        // Then: per_account 큐 nonce [0,1,2]
        let queue = pool.per_account.get("alice").expect("alice queue");
        let nonces: Vec<u64> = queue.iter().map(|tx| tx.nonce).collect();
        assert_eq!(nonces, vec![0, 1, 2]);
    }

    #[test]
    fn insert_updates_global_queue_head_only() {
        // Given: 동일 sender nonce 0,1
        let config = TxPoolConfig {
            capacity: 10,
            max_account_slots: 10,
        };
        let mut pool = TxPool::new(config);
        // When: 두 번 insert
        pool.insert(make_tx("alice", 0, 5, 100, "a"))
            .expect("first insert");
        pool.insert(make_tx("alice", 1, 4, 110, "b"))
            .expect("second insert");
        // Then: global_queue.len == 1, head hash == nonce 0
        assert_eq!(pool.global_queue.len(), 1);
        let head = pool.global_queue.peek().expect("head exists");
        assert_eq!(head.tx.nonce, 0);
        assert_eq!(head.key, (5, 100));
    }

    #[test]
    fn insert_rejects_duplicate_nonce() {
        // Given: nonce 0 사전 삽입
        let config = TxPoolConfig {
            capacity: 10,
            max_account_slots: 10,
        };
        let mut pool = TxPool::new(config);
        pool.insert(make_tx("alice", 0, 5, 100, "a"))
            .expect("initial insert");
        // When: nonce 0 재삽입
        let duplicated = make_tx("alice", 0, 6, 120, "dup");
        let result = pool.insert(duplicated);
        // Then: Err DuplicateNonce, 큐 길이 유지
        assert!(matches!(
            result,
            Err(TxInsertError::DuplicateNonce {
                sender,
                nonce: 0
            }) if sender == "alice"
        ));
        let queue = pool.per_account.get("alice").expect("alice queue");
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn insert_respects_account_slot_limit() {
        // Given: max_account_slots=2
        let config = TxPoolConfig {
            capacity: 10,
            max_account_slots: 2,
        };
        let mut pool = TxPool::new(config);
        pool.insert(make_tx("alice", 0, 5, 100, "a"))
            .expect("first insert");
        pool.insert(make_tx("alice", 1, 4, 110, "b"))
            .expect("second insert");
        // When: 세 번째 insert 시도
        let result = pool.insert(make_tx("alice", 2, 3, 90, "c"));
        // Then: Err AccountLimitReached, 큐 길이 2
        assert!(matches!(
            result,
            Err(TxInsertError::AccountLimitReached { sender }) if sender == "alice"
        ));
        let queue = pool.per_account.get("alice").expect("alice queue");
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn insert_evicts_lowest_priority_when_full() {
        // Given: capacity=3, 우선순위 다양한 tx
        let config = TxPoolConfig {
            capacity: 3,
            max_account_slots: 3,
        };
        let mut pool = TxPool::new(config);
        let high = make_tx("alice", 0, 10, 100, "high");
        let mid = make_tx("bob", 0, 7, 100, "mid");
        let low = make_tx("carol", 0, 2, 100, "low");
        pool.insert(high.clone()).unwrap();
        pool.insert(mid.clone()).unwrap();
        pool.insert(low.clone()).unwrap();
        // When: 4번째 insert
        let extra_low = make_tx("dave", 0, 1, 90, "extra");
        let result = pool.insert(extra_low.clone());
        // Then: Err PoolFull, 낮은 우선순위 hash가 제거됨
        assert!(matches!(result, Err(TxInsertError::PoolFull)));
        let remaining_hashes: Vec<String> = pool
            .per_account
            .values()
            .flat_map(|queue| queue.iter().map(|tx| tx.hash.clone()))
            .collect();
        assert!(!remaining_hashes.contains(&low.hash));
        assert!(!remaining_hashes.contains(&extra_low.hash));
        assert!(pool.total_txs <= pool.config.capacity);
    }

    #[test]
    fn pop_batch_drains_highest_priority_first() {
        // Given: 서로 다른 sender 우선순위
        let config = TxPoolConfig {
            capacity: 10,
            max_account_slots: 3,
        };
        let mut pool = TxPool::new(config);
        pool.insert(make_tx("alice", 0, 8, 100, "a"))
            .expect("alice insert");
        pool.insert(make_tx("bob", 0, 10, 90, "b"))
            .expect("bob insert");
        pool.insert(make_tx("carol", 0, 8, 120, "c"))
            .expect("carol insert");
        // When: pop_batch(limit=3)
        let result = pool.pop_batch(3);
        // Then: drained priority_key 내림차순
        let PopResult::Batch { drained } = result else {
            panic!("expected drained batch");
        };
        let keys: Vec<(u128, u64)> = drained.iter().map(|tx| tx.priority_key()).collect();
        assert_eq!(keys, vec![(10, 90), (8, 120), (8, 100)]);
    }

    #[test]
    fn pop_batch_refreshes_next_nonce_head() {
        // Given: 한 sender nonce 연속 + 타 sender
        let config = TxPoolConfig {
            capacity: 10,
            max_account_slots: 3,
        };
        let mut pool = TxPool::new(config);
        pool.insert(make_tx("alice", 0, 5, 100, "a0"))
            .expect("alice0");
        pool.insert(make_tx("alice", 1, 5, 90, "a1"))
            .expect("alice1");
        pool.insert(make_tx("bob", 0, 6, 110, "b0")).expect("bob0");
        // When: pop_batch(limit>=2)
        let PopResult::Batch { drained } = pool.pop_batch(2) else {
            panic!("expected batch");
        };
        // Then: drained[0]이 최고 우선순위, 동일 sender의 다음 nonce가 head
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].hash, "bob-0-b0");
        assert_eq!(drained[1].hash, "alice-0-a0");
        let alice_queue = pool.per_account.get("alice").expect("alice queue");
        assert_eq!(alice_queue.len(), 1);
        assert_eq!(alice_queue.front().unwrap().hash, "alice-1-a1");
        let head = pool.global_queue.peek().expect("next head");
        assert_eq!(head.tx.hash, "alice-1-a1");
    }

    #[test]
    fn lazy_eviction_skips_stale_entries_on_pop() {
        // Given: evict 후 stale 남긴 상태
        let config = TxPoolConfig {
            capacity: 1,
            max_account_slots: 1,
        };
        let mut pool = TxPool::new(config);
        pool.insert(make_tx("alice", 0, 5, 100, "a"))
            .expect("initial insert");
        // 새 트랜잭션으로 인해 eviction을 시도하게 만들어 stale 항목이 힙에 남습니다.
        let result = pool.insert(make_tx("bob", 0, 4, 90, "b"));
        assert!(matches!(result, Err(TxInsertError::PoolFull)));
        // When: pop_batch
        let pop_result = pool.pop_batch(1);
        // Then: drained에 stale 없음, 구조 일관성 유지
        assert!(matches!(pop_result, PopResult::Empty));
        assert_eq!(pool.total_txs, 0);
        assert!(pool.per_account.is_empty());
    }
}
