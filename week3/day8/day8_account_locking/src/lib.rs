// aim: 동일한 계정을 동시에 두 트랜잭션이 사용하면 충돌. 따라서 대기열에 들어가기 전에 검증해야 함

use std::collections::{HashSet, VecDeque};

// TransactionMeta는 읽기 가능한 계정과 쓰기 가능한 계정을 독립된 대기열로 관리해 충돌을 회피한다.
// id는 이 트랜잭션의 컨텍스트를 구분하기 위한 식별자이며,
// compute_units는 트랜잭션이 사용할 연산량이다.
#[derive(Clone)]
pub struct TransactionMeta {
    pub id: String,
    pub writable_accounts: Vec<String>,
    pub readonly_accounts: Vec<String>,
    pub compute_units: u32,
}

// 슬롯에 이미 실행 중인 정보를 추적한다.
// 읽기 가능한 계정과 쓰기 가능한 계정은 독립적이되 중복된 계정이 사용해선 안되기 때문에 HashSet으로 정의한다.
pub struct SlotExecutionState {
    pub locked_writable: HashSet<String>,
    pub locked_readonly: HashSet<String>,
    pub consumed_compute_units: u32, // 슬롯에서 소비된 총 CU
}

pub struct BlockConstraint {
    pub max_compute_units: u32,
    pub max_transactions: usize,
}

pub enum AccountLockError {
    Conflict { account: String }, // 트랜잭션이 계정을 동시에 사용하려 할 때 충돌한다.
    ComputeLimitExceeded { requested: u32, limit: u32 }, // 한 슬롯에서 트랜잭션이 소비할 수 있는 CU가 초과되었다.
}

// 실행 대기 중인 트랜잭션을 대기열에 쌓기 위한 자료구조이다.
pub struct ExecutionQueue {
    constraint: BlockConstraint,
    state: SlotExecutionState,
    // Vec은 단방향 큐이다. 처리된 트랜잭션을 빼내게 되면 뒷 요소들을 전부 한 칸씩 옮겨야 하기 때문에 O(n) 시간 복잡도다.
    // 따라서, 양방향 큐인 VecDeque를 사용해 처리된 트랜잭션을 빼내고 나서 다음 트랜잭션을 가리키는 포인터만 이동시켜 O(1) 시간 복잡도를 만족한다.
    pending: VecDeque<TransactionMeta>,
}

impl ExecutionQueue {
    // 초기 상태에 아무런 잠금도 획득하지 못한다. 슬롯에 추가할 수 있는 제약만을 정의한다.
    pub fn new(constraint: BlockConstraint) -> Self {
        Self {
            constraint,
            state: SlotExecutionState {
                locked_writable: Default::default(),
                locked_readonly: Default::default(),
                consumed_compute_units: 0,
            },
            pending: Default::default(),
        }
    }

    pub fn try_enqueue(&mut self, tx: TransactionMeta) -> Result<(), AccountLockError> {
        if would_exceed_compute(&self.state, &tx, &self.constraint) {
            // `requested` 는 현재 소비량 + 요청량 CU
            return Err(AccountLockError::ComputeLimitExceeded {
                requested: self.state.consumed_compute_units + tx.compute_units,
                limit: self.constraint.max_compute_units,
            });
        }

        let account = check_account_conflicts(&self.state, &tx);
        if account.is_some() {
            return Err(AccountLockError::Conflict {
                account: account.unwrap(),
            });
        }

        for account in tx.readonly_accounts.iter() {
            self.state.locked_readonly.insert(account.clone());
        }
        for account in tx.writable_accounts.iter() {
            self.state.locked_writable.insert(account.clone());
        }
        self.state.consumed_compute_units += tx.compute_units;
        self.pending.push_back(tx);
        Ok(())
    }

    pub fn release(&mut self, tx_id: &str) {
        if let Some(pos) = self.pending.iter().position(|tx| tx.id == tx_id) {
            if let Some(tx) = self.pending.remove(pos) {
                for account in &tx.writable_accounts {
                    self.state.locked_writable.remove(account);
                }
                for account in &tx.readonly_accounts {
                    self.state.locked_readonly.remove(account);
                }
                self.state.consumed_compute_units =
                    self.state.consumed_compute_units.saturating_sub(tx.compute_units);
            }
        }
    }
}

// 현재 사용량에 새 트랜잭션의 컴퓨트 유닛을 더했을 때 제한 초과시 true
fn would_exceed_compute(
    state: &SlotExecutionState,
    tx: &TransactionMeta,
    constraint: &BlockConstraint,
) -> bool {
    state.consumed_compute_units + tx.compute_units > constraint.max_compute_units
}

// 충돌이 있으면 해당 계정 이름을 반환하고, 없으면 None
fn check_account_conflicts(state: &SlotExecutionState, tx: &TransactionMeta) -> Option<String> {
    // 1. 쓰기 계정 충돌
    // - 새 트랜잭션의 writable_accounts가 이미 잠긴 locked_writable 또는 locked_readonly와 겹치면 충돌이다.
    for account in &tx.writable_accounts {
        if state.locked_writable.contains(account) || state.locked_readonly.contains(account) {
            return Some(account.clone());
        }
    }

    // 2. 읽기 계정 충돌
    // - 새 트랜잭션의 readonly_accounts가 기존의 locked_writable과 겹치면 역시 충돌이다.
    for account in &tx.readonly_accounts {
        if state.locked_writable.contains(account) {
            return Some(account.clone());
        }
    }

    None
}
