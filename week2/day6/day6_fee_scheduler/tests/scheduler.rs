// 수수료 우선순위 스케줄러 통합 테스트
// Integration tests for the fee priority scheduler
//
// 이 테스트는 다음을 검증합니다:
// This test suite verifies:
// 1. push 메서드의 유효성 검사 (수수료, 계산 단위)
//    Validation logic in push method (fee, compute units)
// 2. pop 메서드의 우선순위 정렬 (높은 수수료 우선, 동점시 id 역순)
//    Priority ordering in pop method (high fee first, id reverse on tie)
// 3. 대규모 엔트리 처리 성능 (1,000개 항목)
//    Performance with large number of entries (1,000 items)

use day6_fee_scheduler::{MempoolEntry, PriorityScheduler, SchedulerError, TxClass};

#[test]
fn test_push_validation_fee_too_low() {
    // 수수료가 0일 때 FeeTooLow 에러를 반환하는지 확인
    // Verify that FeeTooLow error is returned when fee is zero
    let mut scheduler = PriorityScheduler::new();

    let entry = MempoolEntry {
        id: "tx1".to_string(),
        fee_micro_lamports: 0,
        compute_units: 100_000,
        class: TxClass::Standard,
    };

    let result = scheduler.push(entry);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SchedulerError::FeeTooLow));
}

#[test]
fn test_push_validation_compute_units_out_of_range() {
    // 계산 단위가 200_000을 초과할 때 ComputeUnitsOutOfRange 에러를 반환하는지 확인
    // Verify that ComputeUnitsOutOfRange error is returned when compute units exceed 200_000
    let mut scheduler = PriorityScheduler::new();

    let entry = MempoolEntry {
        id: "tx2".to_string(),
        fee_micro_lamports: 1000,
        compute_units: 200_001,
        class: TxClass::HighPriority,
    };

    let result = scheduler.push(entry);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SchedulerError::ComputeUnitsOutOfRange
    ));
}

#[test]
fn test_push_validation_success() {
    // 유효한 엔트리가 성공적으로 추가되는지 확인
    // Verify that valid entries are successfully added
    let mut scheduler = PriorityScheduler::new();

    let entry = MempoolEntry {
        id: "tx3".to_string(),
        fee_micro_lamports: 5000,
        compute_units: 150_000,
        class: TxClass::Standard,
    };

    let result = scheduler.push(entry);
    assert!(result.is_ok());
    assert_eq!(scheduler.len(), 1);
}

#[test]
fn test_pop_priority_ordering_by_fee() {
    // 수수료가 높은 순서대로 pop 되는지 확인
    // Verify that entries are popped in descending order of fee
    let mut scheduler = PriorityScheduler::new();

    // 낮은 수수료
    scheduler
        .push(MempoolEntry {
            id: "tx_low".to_string(),
            fee_micro_lamports: 1000,
            compute_units: 100_000,
            class: TxClass::LowPriority,
        })
        .unwrap();

    // 높은 수수료
    scheduler
        .push(MempoolEntry {
            id: "tx_high".to_string(),
            fee_micro_lamports: 5000,
            compute_units: 100_000,
            class: TxClass::HighPriority,
        })
        .unwrap();

    // 중간 수수료
    scheduler
        .push(MempoolEntry {
            id: "tx_mid".to_string(),
            fee_micro_lamports: 3000,
            compute_units: 100_000,
            class: TxClass::Standard,
        })
        .unwrap();

    // 높은 수수료부터 꺼내져야 함
    assert_eq!(scheduler.pop().unwrap().id, "tx_high");
    assert_eq!(scheduler.pop().unwrap().id, "tx_mid");
    assert_eq!(scheduler.pop().unwrap().id, "tx_low");
    assert!(scheduler.is_empty());
}

#[test]
fn test_pop_priority_ordering_with_id_tiebreaker() {
    // 수수료가 같을 때 id의 역순(사전식으로 큰 값)으로 pop 되는지 확인
    // Verify that when fees are equal, entries are popped by id in reverse lexicographic order
    let mut scheduler = PriorityScheduler::new();

    // 동일한 수수료와 compute_units로 여러 엔트리 추가
    scheduler
        .push(MempoolEntry {
            id: "tx_a".to_string(),
            fee_micro_lamports: 2000,
            compute_units: 100_000,
            class: TxClass::Standard,
        })
        .unwrap();

    scheduler
        .push(MempoolEntry {
            id: "tx_c".to_string(),
            fee_micro_lamports: 2000,
            compute_units: 100_000,
            class: TxClass::Standard,
        })
        .unwrap();

    scheduler
        .push(MempoolEntry {
            id: "tx_b".to_string(),
            fee_micro_lamports: 2000,
            compute_units: 100_000,
            class: TxClass::Standard,
        })
        .unwrap();

    // 동점일 때 id의 사전식 역순으로 나와야 함: tx_c -> tx_b -> tx_a
    assert_eq!(scheduler.pop().unwrap().id, "tx_c");
    assert_eq!(scheduler.pop().unwrap().id, "tx_b");
    assert_eq!(scheduler.pop().unwrap().id, "tx_a");
}

#[test]
fn test_pop_priority_complex_scenario() {
    // 수수료와 compute_units가 다양한 복잡한 시나리오 테스트
    // Test complex scenario with various fees and compute units
    let mut scheduler = PriorityScheduler::new();

    // score = fee * 1_000 + (200_000 - compute_units)
    // tx1: 3000 * 1000 + (200000 - 50000) = 3_000_000 + 150_000 = 3_150_000
    scheduler
        .push(MempoolEntry {
            id: "tx1".to_string(),
            fee_micro_lamports: 3000,
            compute_units: 50_000,
            class: TxClass::Standard,
        })
        .unwrap();

    // tx2: 3000 * 1000 + (200000 - 150000) = 3_000_000 + 50_000 = 3_050_000
    scheduler
        .push(MempoolEntry {
            id: "tx2".to_string(),
            fee_micro_lamports: 3000,
            compute_units: 150_000,
            class: TxClass::Standard,
        })
        .unwrap();

    // tx3: 5000 * 1000 + (200000 - 100000) = 5_000_000 + 100_000 = 5_100_000
    scheduler
        .push(MempoolEntry {
            id: "tx3".to_string(),
            fee_micro_lamports: 5000,
            compute_units: 100_000,
            class: TxClass::HighPriority,
        })
        .unwrap();

    // 점수 순서: tx3 (5_100_000) > tx1 (3_150_000) > tx2 (3_050_000)
    assert_eq!(scheduler.pop().unwrap().id, "tx3");
    assert_eq!(scheduler.pop().unwrap().id, "tx1");
    assert_eq!(scheduler.pop().unwrap().id, "tx2");
}

#[test]
fn test_benchmark_style_1000_entries() {
    // 1,000개의 엔트리를 빠르게 추가하고 제거하는 벤치마크 스타일 테스트
    // Benchmark-style test to quickly add and remove 1,000 entries
    let mut scheduler = PriorityScheduler::new();

    // 1,000개의 임의 엔트리 추가
    for i in 0..1000 {
        let entry = MempoolEntry {
            id: format!("tx_{:04}", i),
            fee_micro_lamports: (i % 100 + 1) * 100, // 100 ~ 10,000 범위
            compute_units: ((i % 200) * 1000) as u32, // 0 ~ 199,000 범위
            class: match i % 3 {
                0 => TxClass::HighPriority,
                1 => TxClass::Standard,
                _ => TxClass::LowPriority,
            },
        };

        scheduler.push(entry).unwrap();
    }

    // 정확히 1,000개가 들어갔는지 확인
    assert_eq!(scheduler.len(), 1000);

    // 모든 엔트리를 pop하여 큐가 비워지는지 확인
    let mut count = 0;
    let mut last_score: Option<u128> = None;

    while let Some(entry) = scheduler.pop() {
        count += 1;

        // 각 항목의 점수를 계산하여 내림차순인지 확인
        let score =
            entry.fee_micro_lamports as u128 * 1_000 + (200_000 - entry.compute_units as u128);

        if let Some(prev_score) = last_score {
            // 이전 점수가 현재 점수보다 크거나 같아야 함 (내림차순)
            assert!(
                prev_score >= score,
                "Priority order violated: prev={}, current={}",
                prev_score,
                score
            );
        }

        last_score = Some(score);
    }

    // 정확히 1,000개가 pop 되었는지 확인
    assert_eq!(count, 1000);
    assert!(scheduler.is_empty());
}

#[test]
fn test_empty_scheduler() {
    // 빈 스케줄러에서 pop 시 None이 반환되는지 확인
    // Verify that pop returns None on empty scheduler
    let mut scheduler = PriorityScheduler::new();

    assert!(scheduler.is_empty());
    assert_eq!(scheduler.len(), 0);
    assert!(scheduler.pop().is_none());
}
