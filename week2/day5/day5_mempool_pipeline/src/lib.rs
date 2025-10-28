use std::collections::BTreeMap;

// 이 상태는 트랜잭션의 진행 상황을 쉽게 보여줘요. // This enum tells kids how a transaction is doing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TxStatus {
    Pending,
    Simulated { compute_units: u64 },
    Rejected { reason: String },
}

// 이 구조체는 멤풀에 줄 서 있는 트랜잭션 정보를 담아요. // This struct keeps the info of a waiting mempool transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingTx {
    pub id: String,              // Transaction hash
    pub account: String,         // Sender's public key
    pub fee_micro_lamports: u64, // Paid fee (1 lamports = 10^-6 SOL)
    pub payload_size: u32,       // Serialized transaction size(byte unit)
    pub status: TxStatus,        // Transaction status enum
}

pub trait MempoolFilter {
    fn allow(&self, tx: &PendingTx) -> bool;
}

pub struct ThresholdFilter {
    pub min_fee: u64,                     // 허용할 최소 수수료
    pub max_payload: u32,                 // 허용할 최대 페이로드 크기
    pub reject_simulation_failures: bool, // 시뮬레이션 실패(`TxStatus::Rejected`)를 거부할지 여부
}

impl MempoolFilter for ThresholdFilter {
    fn allow(&self, tx: &PendingTx) -> bool {
        // 1. 최소 수수료 미만은 거부한다.
        if tx.fee_micro_lamports < self.min_fee {
            return false;
        }
        // 2. 페이로드 크기가 한도를 넘으면 거부한다.
        if tx.payload_size > self.max_payload {
            return false;
        }
        // 3. 실패한 시뮬레이션을 거부하도록 설정했다면 Rejected 상태는 막는다.
        if self.reject_simulation_failures && matches!(tx.status, TxStatus::Rejected { .. }) {
            return false;
        }
        true
    }
}

// 필터를 통과한 트랜잭션 참조만 새 벡터로 돌려줘요. // Returns references for transactions that pass the filter.
pub fn filter_transactions<'a, F: MempoolFilter>(
    txs: &'a [PendingTx],
    filter: &F,
) -> Vec<&'a PendingTx> {
    let mut allowed = Vec::new();
    for tx in txs {
        if filter.allow(tx) {
            allowed.push(tx);
        }
    }
    allowed
}

// 계정별로 트랜잭션을 묶어 사전식 맵으로 돌려줘요. // Groups transactions per account, sorted by key.
pub fn group_by_account(txs: &[PendingTx]) -> BTreeMap<String, Vec<PendingTx>> {
    let mut grouped = BTreeMap::new();
    for tx in txs {
        grouped
            .entry(tx.account.clone())
            .or_insert_with(Vec::new)
            .push(tx.clone());
    }
    grouped
}

pub struct AccountStats {
    pub total_fee: u64,
    pub total_bytes: u32,
    pub pending: usize,
}

// 계정별로 총 수수료, 크기, 대기 수를 계산해요. // Calculates per account totals and pending count.
pub fn compute_account_stats(
    grouped: &BTreeMap<String, Vec<PendingTx>>,
) -> BTreeMap<String, AccountStats> {
    let mut stats = BTreeMap::new();
    for (account, txs) in grouped {
        let mut total_fee = 0u64;
        let mut total_bytes = 0u32;
        let mut pending = 0usize;
        for tx in txs {
            total_fee += tx.fee_micro_lamports;
            total_bytes += tx.payload_size;
            match tx.status {
                TxStatus::Pending => pending += 1,
                TxStatus::Simulated { .. } => {}
                TxStatus::Rejected { .. } => {}
            }
        }
        stats.insert(
            account.clone(),
            AccountStats {
                total_fee,
                total_bytes,
                pending,
            },
        );
    }
    stats
}
