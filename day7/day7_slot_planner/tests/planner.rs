use day7_slot_planner::{
    BlockConstraint, MempoolEntry, SlotPlanner, drain_sorted_by_fee,
};
use std::collections::BinaryHeap;

// 제한을 넘는 트랜잭션을 추가하려고 할 때 올바르게 거부하는지 확인한다.
#[test]
fn test_reject_when_exceeds_limit() {
    let constraint = BlockConstraint {
        max_compute_units: 1000,
        max_transactions: 2,
    };

    let mut planner = SlotPlanner::new(constraint);

    let tx1 = MempoolEntry {
        compute_units: 500,
        fee_micro_lamports: 1000,
    };
    assert!(planner.try_add(tx1), "첫 번째 트랜잭션은 추가되어야 함");

    let tx2 = MempoolEntry {
        compute_units: 400,
        fee_micro_lamports: 800,
    };
    assert!(planner.try_add(tx2), "두 번째 트랜잭션은 추가되어야 함");

    let tx3 = MempoolEntry {
        compute_units: 200,
        fee_micro_lamports: 500,
    };

    // 여기서 세 번째 트랜잭션은 거부되어 false가 반환되어야 한다.
    let result = planner.try_add(tx3);
    assert_eq!(result, false);
}

// 수수료가 높은 순으로 트랜잭션을 선택하는지 검증한다.
#[test]
fn test_sorted_selection_by_fee() {
    let mut heap = BinaryHeap::new();
    heap.push(MempoolEntry {
        compute_units: 100,
        fee_micro_lamports: 500,
    });
    heap.push(MempoolEntry {
        compute_units: 200,
        fee_micro_lamports: 2000,
    });
    heap.push(MempoolEntry {
        compute_units: 150,
        fee_micro_lamports: 1500,
    });

    let sorted = drain_sorted_by_fee(&mut heap);

    assert_eq!(sorted[0].fee_micro_lamports, 2000);
}
