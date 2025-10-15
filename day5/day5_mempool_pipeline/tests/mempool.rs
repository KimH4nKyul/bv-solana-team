// 이 테스트 묶음은 멤풀 필터와 계정 통계를 확인해요. // These tests verify mempool filters and account stats.
use day5_mempool_pipeline::*;

#[test]
fn threshold_filter_checks_fee_payload_and_rejections() {
    let valid = PendingTx {
        id: "tx-valid".into(),
        account: "alice".into(),
        fee_micro_lamports: 500,
        payload_size: 200,
        status: TxStatus::Pending,
    };
    let low_fee = PendingTx {
        fee_micro_lamports: 100,
        ..valid.clone()
    };
    let heavy_payload = PendingTx {
        payload_size: 900,
        ..valid.clone()
    };
    let rejected = PendingTx {
        status: TxStatus::Rejected {
            reason: "simulation fail".into(),
        },
        ..valid.clone()
    };

    let filter = ThresholdFilter {
        min_fee: 300,
        max_payload: 512,
        reject_simulation_failures: true,
    };

    assert!(filter.allow(&valid));
    assert!(!filter.allow(&low_fee));
    assert!(!filter.allow(&heavy_payload));
    assert!(!filter.allow(&rejected));
}

#[test]
fn filter_transactions_returns_references_with_original_lifetime() {
    let txs = vec![
        PendingTx {
            id: "keep".into(),
            account: "bob".into(),
            fee_micro_lamports: 1_000,
            payload_size: 100,
            status: TxStatus::Pending,
        },
        PendingTx {
            id: "drop".into(),
            account: "bob".into(),
            fee_micro_lamports: 10,
            payload_size: 100,
            status: TxStatus::Rejected {
                reason: "low fee".into(),
            },
        },
    ];

    let filter = ThresholdFilter {
        min_fee: 500,
        max_payload: 256,
        reject_simulation_failures: true,
    };

    let allowed = filter_transactions(&txs, &filter);

    assert_eq!(allowed.len(), 1);
    assert_eq!(allowed[0].id, "keep");
    assert!(std::ptr::eq(allowed[0], &txs[0]));
}

#[test]
fn grouping_and_stats_summarize_accounts_correctly() {
    let txs = vec![
        PendingTx {
            id: "a1".into(),
            account: "alice".into(),
            fee_micro_lamports: 400,
            payload_size: 120,
            status: TxStatus::Pending,
        },
        PendingTx {
            id: "a2".into(),
            account: "alice".into(),
            fee_micro_lamports: 600,
            payload_size: 200,
            status: TxStatus::Simulated {
                compute_units: 50_000,
            },
        },
        PendingTx {
            id: "b1".into(),
            account: "bob".into(),
            fee_micro_lamports: 300,
            payload_size: 150,
            status: TxStatus::Pending,
        },
        PendingTx {
            id: "b2".into(),
            account: "bob".into(),
            fee_micro_lamports: 200,
            payload_size: 90,
            status: TxStatus::Rejected {
                reason: "conflict".into(),
            },
        },
    ];

    let grouped = group_by_account(&txs);
    let keys: Vec<_> = grouped.keys().cloned().collect();
    assert_eq!(keys, vec!["alice".to_string(), "bob".to_string()]);

    assert_eq!(grouped["alice"].len(), 2);
    assert_eq!(grouped["bob"].len(), 2);

    let stats = compute_account_stats(&grouped);
    let alice_stats = &stats["alice"];
    assert_eq!(alice_stats.total_fee, 1_000);
    assert_eq!(alice_stats.total_bytes, 320);
    assert_eq!(alice_stats.pending, 1);

    let bob_stats = &stats["bob"];
    assert_eq!(bob_stats.total_fee, 500);
    assert_eq!(bob_stats.total_bytes, 240);
    assert_eq!(bob_stats.pending, 1);
}
