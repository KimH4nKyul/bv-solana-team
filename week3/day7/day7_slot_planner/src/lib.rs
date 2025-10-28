use std::cmp::Ordering;
use std::collections::BinaryHeap;

// 메모리풀에 대기 중인 트랜잭션 하나를 나타낸다.
#[derive(Debug, Clone, Eq)]
pub struct MempoolEntry {
    // 이 트랜잭션이 사용할 계산 자원, 트랜잭션이 실행될 때 사용하는 계산량이며 블록 한도를 제한하기 위해 필요하다.
    pub compute_units: u32,
    // 사용자가 지불한 수수료, 수수료가 높을수록 우선순위도 높아진다.
    pub fee_micro_lamports: u64,
}

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

// 한 블록(슬롯)이 수용할 수 있는 최대 제한을 나타낸다.
pub struct BlockConstraint {
    // 총 계산 자원 한도, 솔라나는 슬롯마다 최대 4_800만 CU를 사용할 수 있다.
    pub max_compute_units: u32,
    // 한 블록(슬롯)에 담을 수 있는 트랜잭션 수도 제한된다.
    pub max_transactions: usize,
}

// 블록에 담기로 선택된 트랜잭션 묶음과 그 합계 정보를 나타낸다.
// 합계를 미리 따로 저장해야 매번 벡터를 순회해 계산하는 것보다 빠르다.
// 새 트랜잭션을 추가할 때마다 합계만 업데이트하면 된다.
pub struct PlannedBundle {
    pub entries: Vec<MempoolEntry>,
    pub total_compute_units: u32,
    pub total_fee_micro_lamports: u64,
}

impl PlannedBundle {
    // 주어진 제약 조건에서 남은 용량을 계산한다.
    pub fn remaining_capacity(&self, block_constraint: &BlockConstraint) -> (u32, usize) {
        let remaining_compute = block_constraint
            .max_compute_units
            .saturating_sub(self.total_compute_units);
        let remaining_transactions: usize = block_constraint
            .max_transactions
            .saturating_sub(self.entries.len());
        (remaining_compute, remaining_transactions)
    }
}

pub fn drain_sorted_by_fee(heap: &mut BinaryHeap<MempoolEntry>) -> Vec<MempoolEntry> {
    let mut sorted = Vec::new();
    while let Some(entry) = heap.pop() {
        sorted.push(entry);
    }
    sorted
}

// 블록 제약 조건 내에서 최적의 트랜잭션 묶음을 선택하는 플래너다.
pub struct SlotPlanner {
    // 이 블록이 지켜야 할 제한
    block_constraint: BlockConstraint,
    // 현재까지 선택된 트랜잭션들
    current_bundle: PlannedBundle,
}

impl SlotPlanner {
    // 새 플래너를 생성한다.
    // 처음에는 빈 번들로 시작해, 트랜잭션을 하나씩 추가할 수 있어야 한다.
    pub fn new(block_constraint: BlockConstraint) -> Self {
        Self {
            block_constraint,
            current_bundle: PlannedBundle {
                entries: Vec::new(),
                total_compute_units: 0,
                total_fee_micro_lamports: 0,
            },
        }
    }

    // 트랜잭션을 슬롯에 추가해도 되는지 확인할 수 있어야 한다.
    pub fn can_add(&self, entry: &MempoolEntry) -> bool {
        // 추가했을 때의 예상 합계를 계산한다.
        let next_compute = self.current_bundle.total_compute_units + entry.compute_units;
        let next_count = self.current_bundle.entries.len() + 1;

        // 두 제한을 모두 확인한다.
        // CU가 한도를 넘지 않으며 트랜잭션 개수가 한도를 넘지 않아야 한다.
        next_compute <= self.block_constraint.max_compute_units
            && next_count <= self.block_constraint.max_transactions
    }

    // 새 트랜잭션을 번들에 추가한다.
    // 성공시 true, 제한을 넘어 실패하면 false이다.
    pub fn try_add(&mut self, entry: MempoolEntry) -> bool {
        if self.can_add(&entry) {
            self.current_bundle.total_compute_units += entry.compute_units;
            self.current_bundle.total_fee_micro_lamports += entry.fee_micro_lamports;
            self.current_bundle.entries.push(entry);
            true
        } else {
            false
        }
    }

    // 지금까지 선택한 트랜잭션 묶음을 반환하고 플래너를 소비한다.
    // finalize가 호출되면 플래너가 완료됨을 의미한다.
    // 완료 후에는 더이상 트랜잭션을 추가할 수 없어야 한다.
    // Rust의 소유권 시스템이 이를 컴파일 타임에 보장한다.
    pub fn finalize(self) -> PlannedBundle {
        self.current_bundle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_planner_should_finalize() {
        let constraint = BlockConstraint {
            max_compute_units: 100,
            max_transactions: 2,
        };

        let mut planner = SlotPlanner::new(constraint);

        let tx1 = MempoolEntry {
            compute_units: 100,
            fee_micro_lamports: 50,
        };

        planner.try_add(tx1);

        let result = planner.finalize();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.total_fee_micro_lamports, 50);
        assert_eq!(result.total_compute_units, 100);
    }

    #[test]
    fn slot_planner_should_check_to_try_add_transaction() {
        let constraint = BlockConstraint {
            max_compute_units: 100,
            max_transactions: 2,
        };

        let mut planner = SlotPlanner::new(constraint);

        let tx1 = MempoolEntry {
            compute_units: 100,
            fee_micro_lamports: 50,
        };

        let result = planner.try_add(tx1);
        assert_eq!(result, true);
        assert_eq!(planner.current_bundle.entries.len(), 1);
        assert_eq!(planner.current_bundle.total_fee_micro_lamports, 50);

        let tx2 = MempoolEntry {
            compute_units: 100,
            fee_micro_lamports: 50,
        };

        assert_eq!(planner.try_add(tx2), false);
    }

    #[test]
    fn slot_planner_should_check_to_can_add_transaction() {
        let constraint = BlockConstraint {
            max_compute_units: 100,
            max_transactions: 2,
        };

        let planner = SlotPlanner::new(constraint);

        let tx1 = MempoolEntry {
            compute_units: 100,
            fee_micro_lamports: 50,
        };

        let result = planner.can_add(&tx1);
        assert_eq!(result, true);
        assert_eq!(planner.current_bundle.entries.len(), 0);
        assert_eq!(planner.current_bundle.total_compute_units, 0);
        assert_eq!(planner.current_bundle.total_fee_micro_lamports, 0);
    }

    #[test]
    fn slot_planner_should_initialize() {
        let constraint = BlockConstraint {
            max_compute_units: 100,
            max_transactions: 10,
        };

        let planner = SlotPlanner::new(constraint);

        assert_eq!(planner.block_constraint.max_compute_units, 100);
        assert_eq!(planner.block_constraint.max_transactions, 10);

        assert_eq!(planner.current_bundle.entries.len(), 0);
        assert_eq!(planner.current_bundle.total_compute_units, 0);
        assert_eq!(planner.current_bundle.total_fee_micro_lamports, 0);
    }

    #[test]
    fn sort_helper_should_sort_by_fee() {
        let mut heap = BinaryHeap::new();
        heap.push(MempoolEntry {
            compute_units: 1,
            fee_micro_lamports: 10,
        });
        heap.push(MempoolEntry {
            compute_units: 2,
            fee_micro_lamports: 20,
        });
        heap.push(MempoolEntry {
            compute_units: 3,
            fee_micro_lamports: 15,
        });

        let result = drain_sorted_by_fee(&mut heap);

        assert_eq!(result.len(), 3);
        assert_eq!(result.first().unwrap().fee_micro_lamports, 20);
    }

    #[test]
    fn bundle_should_calculate_remining_capacity() {
        let constraint = BlockConstraint {
            max_compute_units: 100,
            max_transactions: 2,
        };

        let entries = vec![
            MempoolEntry {
                compute_units: 50,
                fee_micro_lamports: 1,
            },
            MempoolEntry {
                compute_units: 20,
                fee_micro_lamports: 2,
            },
        ];

        let mut bundle = PlannedBundle {
            entries,
            total_compute_units: 50 + 20,
            total_fee_micro_lamports: 1 + 2,
        };

        let (cu, tx_count) = bundle.remaining_capacity(&constraint);
        assert_eq!(bundle.entries.len(), 2);
        assert_eq!(cu, 30);
        assert_eq!(tx_count, 0);

        bundle.entries.push(MempoolEntry {
            compute_units: 100,
            fee_micro_lamports: 3,
        });
        bundle.total_compute_units = 50 + 20 + 100;
        bundle.total_fee_micro_lamports = 1 + 2 + 3;

        let (cu, tx_count) = bundle.remaining_capacity(&constraint);
        assert_eq!(bundle.entries.len(), 3);
        assert_eq!(cu, 0);
        assert_eq!(tx_count, 0);
    }

    #[test]
    fn bundle_should_have_transactions() {
        let entries = vec![
            MempoolEntry {
                compute_units: 1,
                fee_micro_lamports: 1,
            },
            MempoolEntry {
                compute_units: 2,
                fee_micro_lamports: 2,
            },
        ];

        let bundle = PlannedBundle {
            entries,
            total_compute_units: 0,
            total_fee_micro_lamports: 0,
        };

        let result = bundle.entries.len();

        assert_eq!(result, 2);
    }

    #[test]
    fn each_entry_should_compare() {
        let tx1 = MempoolEntry {
            compute_units: 10,
            fee_micro_lamports: 1,
        };

        let tx2 = MempoolEntry {
            compute_units: 20,
            fee_micro_lamports: 2,
        };

        let result = tx1.cmp(&tx2);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn each_entry_should_not_eq() {
        let tx1 = MempoolEntry {
            compute_units: 10,
            fee_micro_lamports: 1,
        };

        let tx2 = MempoolEntry {
            compute_units: 20,
            fee_micro_lamports: 2,
        };
        assert_eq!(tx1.eq(&tx2), false);

        // let mut heap: BinaryHeap<MempoolEntry> = BinaryHeap::new();
        //
        // heap.push(tx1);
        // heap.push(tx2);
        //
        // let result = heap.pop();
        //
        // assert!(result.is_some());
        // assert_eq!(result.unwrap().fee_micro_lamports, 2);
    }
}
