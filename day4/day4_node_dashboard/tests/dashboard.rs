use day4_node_dashboard::*;

#[test]
fn test_count_uninitialized() {
    // given
    let peers: &[NodePeer] = &[
        NodePeer {
            name: "peer1".to_string(),
            last_slot: None,
            latency_ms: 123,
        },
        NodePeer {
            name: "peer2".to_string(),
            last_slot: None,
            latency_ms: 456,
        },
    ];

    // when
    let result = count_uninitialized(peers);

    // then
    assert_eq!(result, 2);
}

#[test]
fn test_fastest_peer_is_empty() {
    // given
    let peers: &[NodePeer; 0] = &[];

    // when
    let result = fastest_peer(peers);

    // then
    assert!(result.is_none());
}

#[test]
fn test_fastest_peer_not_empty() {
    // given
    let peers: &[NodePeer] = &[
        NodePeer {
            name: "peer1".to_string(),
            last_slot: Some(1),
            latency_ms: 123,
        },
        NodePeer {
            name: "peer2".to_string(),
            last_slot: Some(1),
            latency_ms: 456,
        },
        NodePeer {
            name: "peer3".to_string(),
            last_slot: Some(1),
            latency_ms: 789,
        },
    ];

    // when
    let result = fastest_peer(peers);

    // then
    assert_eq!(result.unwrap().name, "peer1".to_string());
}

#[test]
fn test_summarize_slots() {
    // given
    let peers: &[NodePeer] = &[
        NodePeer {
            name: "peer1".to_string(),
            last_slot: None,
            latency_ms: 123,
        },
        NodePeer {
            name: "peer2".to_string(),
            last_slot: Some(1),
            latency_ms: 456,
        },
    ];

    // when
    let result = summarize_slots(peers);

    // then
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "peer1 awaiting first block");
    assert_eq!(result[1], "peer2 synced up to slot 1");
}
