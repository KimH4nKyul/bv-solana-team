// 네트워크 기반 블록체인 노드 간 관계 표현하는 핵심 데이터 구조
// 블록체인 노드는 혼자 존재하지 않고 항상 여러 개의 피어 노드와 연결됨
// 각 피어로부터 블록, 슬롯, 트랜잭션 정보를 교환해 동기화함
// 이 때, 각 피어의 상태를 관리하지 않으면,
// 어떤 노드가 최신 상태인지, 느려서 제외해야 하는지, 불안정한지 판단 못함
// 즉, NodePeer는 연결 중인 피어들 상태를 실시간 추적하기 위한 모델
pub struct NodePeer {
    pub name: String,           // 피어 이름
    pub last_slot: Option<u64>, // 마지막으로 보고된 슬롯 번호, 아직 블록을 못받은 피어라면 None
    pub latency_ms: u64,        // 왕복 지연시간
}

// 연결된 피어 노드 중에 어떤게 아직 블록을 못받아 동기화 못했는 지 해당 피어 노드 수를 체크한다.
pub fn count_uninitialized(peers: &[NodePeer]) -> usize {
    let mut count: usize = 0;
    for peer in peers {
        if peer.last_slot.is_none() {
            count += 1;
        }
    }
    count
}

// 지연 시간이 가장 낮은 피어와 연결되도록 연결할 피어들의 레이턴시를 체크하고, 가장 지연시간이 낮은 피어를 반환한다.
pub fn fastest_peer<'a>(peers: &'a [NodePeer]) -> Option<&'a NodePeer> {
    // 피어가 하나도 없다면 None
    if peers.is_empty() {
        return None;
    }

    // 가장 간단하게
    // 루프를 돌며 입력된 피어 노드와 현재 비교할 피어 노드의 레이턴시를 측정해
    // 가장 낮은 지연시간을 가진 피어 노드를 반환한다.
    let mut fastest: Option<&'a NodePeer> = None;
    for peer in peers {
        match fastest {
            None => fastest = Some(peer),
            Some(current) if peer.latency_ms < current.latency_ms => fastest = Some(peer),
            _ => {} // 기존보다 느리면 무시한다.
        };
    }

    // 혹은 아래와 같이 min_by_key API를 통해 레이턴시에 대한 최솟값을 갖는 피어 노드를 찾을 수도 있다.
    // peers.iter().min_by_key(|peer| peer.latency_ms);

    fastest
}

// 현재 슬롯이 동기화된 피어들과 그렇지 않은 피어들을 요약 보고한다.
pub fn summarize_slots(peers: &[NodePeer]) -> Vec<String> {
    // 함수형으로 불변 데이터를 유지하며 코드를 더 간결하고 가독성있게 한다.
    peers
        .iter()
        .map(|peer| match peer.last_slot {
            None => format!("{} awaiting first block", peer.name),
            Some(slot) => format!("{} synced up to slot {}", peer.name, slot),
        })
        .collect()
}
