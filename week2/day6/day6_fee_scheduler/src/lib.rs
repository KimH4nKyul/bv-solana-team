// 멤풀 데이터를 다듬기 위한 "수수료 우선순위 스케줄러"를 만든다.
// 높은 수수료 트랜잭션을 먼저 꺼내는 큐를 구현하고, 의도치 않은 입력은 에러로 막는다.
// 스케줄러는 블록 생성 전 대기열을 깔끔하게 정렬해 준다.
use std::collections::BinaryHeap;

// 트랜잭션 우선순위 등급
// Transaction priority class
// - HighPriority: 긴급하게 처리해야 하는 트랜잭션 (예: 청산, 타임센서티브 작업)
//   Transactions that need urgent processing (e.g., liquidations, time-sensitive operations)
// - Standard: 일반적인 트랜잭션
//   Regular transactions
// - LowPriority: 낮은 우선순위 트랜잭션 (예: 일괄 처리 작업)
//   Low priority transactions (e.g., batch operations)
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TxClass {
    HighPriority,
    Standard,
    LowPriority,
}

// 멤풀에 대기 중인 트랜잭션 엔트리
// Mempool entry representing a pending transaction
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MempoolEntry {
    pub id: String,
    pub fee_micro_lamports: u64,
    pub compute_units: u32,
    pub class: TxClass,
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("fee must be greater then zero")]
    FeeTooLow,
    #[error("compute units must be within 200_000")]
    ComputeUnitsOutOfRange,
}

// 스케줄링을 위해 점수가 매겨진 트랜잭션
// Transaction with calculated score for scheduling
// 우선순위 큐에서 사용되며, score가 높을수록 먼저 처리됨
// Used in priority queue, higher score gets processed first
#[derive(Eq, PartialEq)]
pub struct ScheduledTx {
    pub entry: MempoolEntry,
    pub score: u128,
}

// ScheduledTx의 정렬 규칙을 정의
// Define ordering rules for ScheduledTx
// 1. 점수가 높은 항목이 먼저 나옴 (max-heap)
//    Higher score comes first (max-heap)
// 2. 점수가 동일하면 id의 사전식 역순 (큰 값 우선)
//    On tie, reverse lexicographic order of id (larger value first)
impl Ord for ScheduledTx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 먼저 score를 비교 (높은 값이 우선)
        match self.score.cmp(&other.score) {
            std::cmp::Ordering::Equal => {
                // 점수가 같으면 id를 역순으로 비교
                self.entry.id.cmp(&other.entry.id)
            }
            other => other,
        }
    }
}

impl PartialOrd for ScheduledTx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// 수수료 기반 우선순위 스케줄러
// Fee-based priority scheduler
#[derive(Default)]
pub struct PriorityScheduler {
    pub scheduled_txs: BinaryHeap<ScheduledTx>,
}

impl PriorityScheduler {
    /// 새로운 스케줄러 인스턴스를 생성
    /// Creates a new scheduler instance
    pub fn new() -> Self {
        PriorityScheduler {
            scheduled_txs: BinaryHeap::new(),
        }
    }

    /// 트랜잭션을 큐에 추가하고 유효성을 검사
    /// Adds a transaction to the queue with validation
    ///
    /// 수수료와 계산 단위를 검증한 후, 점수를 계산하여 우선순위 큐에 삽입
    /// Validates fee and compute units, then calculates score and inserts into priority queue
    pub fn push(&mut self, entry: MempoolEntry) -> Result<(), SchedulerError> {
        let fee_micro_lamports = entry.fee_micro_lamports;
        if fee_micro_lamports == 0 {
            return Err(SchedulerError::FeeTooLow);
        }
        let compute_units = entry.compute_units;
        if compute_units > 200_000 {
            return Err(SchedulerError::ComputeUnitsOutOfRange);
        }

        // 점수 계산: fee * 1_000 + (200_000 - compute_units)
        // Score calculation: fee * 1_000 + (200_000 - compute_units)
        // 높은 수수료와 낮은 계산 단위가 우선순위를 가짐
        // Higher fee and lower compute units get priority
        let score = fee_micro_lamports as u128 * 1_000 + (200_000 - compute_units as u128);

        self.scheduled_txs.push(ScheduledTx { entry, score });
        Ok(())
    }

    /// 가장 높은 우선순위 트랜잭션을 큐에서 제거하고 반환
    /// Removes and returns the highest priority transaction from the queue
    pub fn pop(&mut self) -> Option<MempoolEntry> {
        Some(self.scheduled_txs.pop()?.entry)
    }

    /// 큐에 있는 트랜잭션 수를 반환
    /// Returns the number of transactions in the queue
    pub fn len(&self) -> usize {
        self.scheduled_txs.len()
    }

    /// 큐가 비어있는지 확인
    /// Checks if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
