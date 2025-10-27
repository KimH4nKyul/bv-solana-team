// HeaderAccumulator가 포크를 추적해 가장 무거운 체인을 선택해야 한다.
// total difficulty가 더 큰 체인이 들어오면 canonical 체인을 reorg 해야 한다.
// 포크 정보와 결과를 명확한 타입으로 표현해야 한다.

// 다수의 헤더를 그래프로 표현해야 한다.
// 새로운 헤더가 들어왔을 때 어떤 포크가 canonical이 되어야 하는지 결정해야 한다.

// 해시 -> StoredHeader 매핑을 빠르게 조회하기 위해 HashMap을 사용한다.
use std::collections::HashMap;

// 헤더 삽입 과정에서 발생할 수 있는 모든 실패 유형을 열거한다.
#[derive(Debug)]
pub enum HeaderInsertError {
    // 동일 해시가 이미 저장돼 있을 때
    DuplicateHash { hash: String },
    // 부모 해시를 찾을 수 없을 때
    UnknownParent { parent_hash: String },
    // 부모 번호 + 1 규칙을 어겼을 때
    NumberMismatch { expected: u64, got: u64 },
}

// canonical 체인이 어떤 식으로 반응했는지를 호출자에게 알려주는 결과 타입이다.
#[derive(Debug)]
pub enum ReorgOutcome {
    // canonical이 변하지 않은 경우
    NoReorg,
    Extended {
        // 기존 head 바로 뒤로 새로운 헤더가 붙은 경우
        new_head: BlockHeader,
    },
    Reorganized {
        // 다른 포크가 canonical이 되면서 head가 변경된 경우
        new_head: BlockHeader,
        old_head: BlockHeader,
        // 롤백된 블록 수를 나타낸다.
        depth: usize,
    },
}

// 네트워크에서 받아온 원본 헤더 데이터를 표현한다.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    // 블록 번호는 체인의 순서를 정의한다.
    pub number: u64,
    // 블록을 식별하는 고유 해시다.
    pub hash: String,
    // 부모 해시가 Option으로 감싸져 있어 제네시스를 표현할 수 있다.
    pub parent_hash: Option<String>,
    // 난이도는 누적 난이도 계산에 사용된다.
    pub difficulty: u64,
}

// 캐시된 부모 링크와 누적 난이도를 담아 두기 위한 구조체다.
pub struct StoredHeader {
    // 원본 헤더를 그대로 보존한다.
    pub header: BlockHeader,
    // 빠른 역추적을 위해 부모 해시를 별도로 저장한다.
    pub parent: Option<String>,
    // 제네시스부터 해당 헤더까지의 누적 난이도 합이다.
    pub total_difficulty: u128,
}

// 포크 선택기의 전체 상태를 관리한다.
pub struct HeaderForkChoice {
    // 제네시스 해시를 기억해 canonical이 비어 있어도 기준점을 유지한다.
    genesis_hash: String,
    // 제네시스부터 현재 head까지의 해시를 순서대로 가진다.
    canonical: Vec<String>,
    // 저장된 모든 헤더를 해시 기준으로 접근할 수 있게 한다.
    nodes: HashMap<String, StoredHeader>,
}

impl HeaderForkChoice {
    // 제네시스 헤더만으로 초기 상태를 구성한다.
    pub fn new(genesis: BlockHeader) -> Self {
        // 제네시스를 StoredHeader 형태로 변환한다.
        let stored_header = StoredHeader {
            header: genesis.clone(),
            parent: None,
            total_difficulty: genesis.difficulty as u128,
        };

        // 여러 곳에서 재사용할 제네시스 해시 복사본을 확보한다.
        let genesis_hash = genesis.hash.clone();

        // 노드 맵에 제네시스를 넣고 canonical 벡터의 초기 상태를 만든다.
        let mut nodes = HashMap::new();
        // 제네시스 StoredHeader를 해시 키로 등록한다.
        nodes.insert(genesis_hash.clone(), stored_header);

        Self {
            // canonical은 제네시스 하나로 시작한다.
            canonical: vec![genesis_hash.clone()],
            genesis_hash,
            nodes,
        }
    }

    // 현재 canonical head에 해당하는 BlockHeader 참조를 돌려준다.
    pub fn head(&self) -> &BlockHeader {
        // canonical 마지막 요소가 현재 head 해시다.
        let last_canonical = self.canonical.last().unwrap();
        // 노드 맵에서 head에 해당하는 StoredHeader를 찾는다.
        let stored_header = self.nodes.get(last_canonical).unwrap();
        // 내부 BlockHeader 참조를 반환한다.
        &stored_header.header
    }

    // canonical 체인의 모든 해시를 순회할 수 있는 이터레이터를 제공한다.
    pub fn canonical_hashes(&self) -> impl Iterator<Item = &String> {
        // 외부에서 canonical을 순회할 수 있도록 iter를 그대로 노출한다.
        self.canonical.iter()
    }

    // 새 헤더를 삽입하고 canonical이 필요하면 재구성한다.
    pub fn try_insert(&mut self, header: BlockHeader) -> Result<ReorgOutcome, HeaderInsertError> {
        // 동일 해시가 이미 저장돼 있으면 중복 삽입을 막는다.
        if self.nodes.contains_key(&header.hash) {
            return Err(HeaderInsertError::DuplicateHash {
                hash: header.hash.clone(),
            });
        }

        // 제네시스가 아닌데 부모 해시가 비어 있으면 삽입을 거부한다.
        let parent_hash = header.parent_hash.clone().ok_or_else(|| {
            // 부모가 비어 있는 헤더가 어느 것인지 바로 알 수 있게 상세한 문구를 남긴다.
            HeaderInsertError::UnknownParent {
                parent_hash: format!("none (header {})", header.hash),
            }
        })?;
        // 부모 해시가 실제 저장소에 있는지 확인해 구조를 보존한다.
        let parent_header =
            self.nodes
                .get(parent_hash.as_str())
                .ok_or(HeaderInsertError::UnknownParent {
                    parent_hash: parent_hash.clone(),
                })?;

        // 부모 번호와의 관계가 깨지면 바로 에러를 반환한다.
        if header.number != parent_header.header.number + 1 {
            return Err(HeaderInsertError::NumberMismatch {
                expected: parent_header.header.number + 1,
                got: header.number,
            });
        }

        // 이후 절차에서 여러 번 사용하므로 해시를 복사해 둔다.
        let hash = header.hash.clone();
        // 부모 누적 난이도에 현재 난이도를 더해 새 total difficulty를 계산한다.
        let total_difficulty = parent_header.total_difficulty + header.difficulty as u128;
        // StoredHeader에 부모 링크와 누적 난이도를 채워 넣는다.
        let stored_header = StoredHeader {
            header,
            parent: Some(parent_hash.clone()),
            total_difficulty,
        };
        // 새 헤더를 nodes 맵에 등록한다.
        self.nodes.insert(hash.clone(), stored_header);

        // 재구성 전에 기존 head 해시를 따로 저장해 둔다.
        let previous_head_hash = self.canonical.last().cloned();
        // 재구성 여부를 판별하고 리턴 정보에 활용하기 위해 기존 head 해시를 기억해 둔다.
        let (canonical_changed, reorg_depth) = self.rebuild_canonical_if_needed(&hash);

        // canonical이 그대로면 NoReorg로 처리하고 조기 반환한다.
        if !canonical_changed {
            return Ok(ReorgOutcome::NoReorg);
        }

        // canonical이 변경됐으니 최신 head를 복사해 둔다.
        let new_head = self.head().clone();
        // 깊이가 0이면 기존 head 뒤로 새로운 블록이 붙은 것이라 재구성이 아니다.
        if reorg_depth == 0 {
            return Ok(ReorgOutcome::Extended { new_head });
        }

        // 재구성이 일어난 경우 이전 head를 복원해 결과에 포함한다.
        let old_head_hash = previous_head_hash.expect("canonical always has a head");
        let old_head = self
            .nodes
            .get(&old_head_hash)
            .expect("old head must stay in the map")
            .header
            .clone();

        // 새 head, 이전 head, 재구성 깊이를 함께 전달한다.
        Ok(ReorgOutcome::Reorganized {
            new_head,
            old_head,
            depth: reorg_depth,
        })
    }

    // 더 무거운 체인이 나타났을 때만 canonical을 갱신하고, 변화 여부와 reorg 깊이를 반환한다.
    fn rebuild_canonical_if_needed(&mut self, new_hash: &str) -> (bool, usize) {
        // 현재 canonical head의 해시를 가져온다.
        let current_head_hash = self
            .canonical
            .last()
            .expect("canonical should never be empty")
            .clone();
        // head가 nodes에 반드시 존재해야 한다.
        let current_head = self.nodes.get(&current_head_hash).expect("head must exist");

        // 새 후보 헤더가 없으면 로직상 오류다.
        let candidate = self.nodes.get(new_hash).expect("new header must exist");
        // 후보 누적 난이도가 작거나 같으면 canonical을 유지한다.
        if candidate.total_difficulty <= current_head.total_difficulty {
            return (false, 0);
        }

        // 새 후보에서 시작해 부모 방향으로 경로를 역추적한다.
        let mut path = Vec::new();
        // 새 헤더에서 역추적을 시작하기 위해 커서를 초기화한다.
        let mut cursor = Some(new_hash.to_string());
        while let Some(hash) = cursor {
            // 현재 노드를 경로에 누적한다.
            path.push(hash.clone());
            // 부모 포인터를 따라 한 단계 위로 이동한다.
            cursor = self.nodes.get(&hash).and_then(|node| node.parent.clone());
        }
        // 역추적한 목록을 뒤집어 제네시스 -> 후보 순서로 바꾼다.
        path.reverse();
        debug_assert_eq!(
            path.first()
                .expect("canonical path must contain at least genesis"),
            &self.genesis_hash,
            "new canonical path must keep genesis at the front"
        );

        // 기존 canonical과 새 경로의 공통 prefix 길이를 계산한다.
        let mut prefix = 0;
        while prefix < self.canonical.len()
            && prefix < path.len()
            && self.canonical[prefix] == path[prefix]
        {
            // 공통된 구간이 이어지는 동안 prefix를 증가시킨다.
            prefix += 1;
        }
        // canonical에서 잘려 나가는 구간의 길이가 reorg 깊이가 된다.
        let reorg_depth = self.canonical.len().saturating_sub(prefix);

        // canonical을 새 경로로 교체하고 변경됨을 알린다.
        self.canonical = path;
        // 변경 여부(true)와 reorg 깊이를 반환한다.
        (true, reorg_depth)
    }
}
