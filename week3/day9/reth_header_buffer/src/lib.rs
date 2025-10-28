use std::collections::HashMap;

// `reth`는 대용량의 블록 정보를 모두 불러와 동기화 하는 것이 병목이기 때문에, 이 문제를 해결하려고 블록 헤더를 통해 체인 구조를 빠르게 파악하고 동기화 한다.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub difficulty: u64,
}

// `reth`는 실제로 아래 내용을 MBDX의 테이블에 매핑해 저장한다. 블록 헤더는 블록 해시를 키로 사용하는 테이블은 `Headers`, 블록 넘버를 키로 하는 블록 해시는 `CanonicalHeaders`, 개별 블록 시점당 누적 난이도는 `HeaderTD`에 저장한다.
pub struct HeaderBuffer {
    canonical: Vec<BlockHeader>,
    index_by_hash: HashMap<String, usize>,
    total_difficulty: u128,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HeaderInsertError {
    ParentNotFound { parent_hash: String },
    NumberMismatch { expected: u64, got: u64 },
    DuplicationHash { hash: String },
}

impl HeaderBuffer {
    pub fn try_append(&mut self, header: BlockHeader) -> Result<(), HeaderInsertError> {
        if self.index_by_hash.contains_key(header.hash.as_str()) {
            return Err(HeaderInsertError::DuplicationHash {
                hash: header.hash.clone(),
            });
        }

        if !self.index_by_hash.contains_key(header.parent_hash.as_str()) {
            return Err(HeaderInsertError::ParentNotFound {
                parent_hash: header.parent_hash.clone(),
            });
        }

        let parent_number = *self.index_by_hash.get(header.parent_hash.as_str()).unwrap();
        let parent = &self.canonical[parent_number];

        if header.number != parent.number + 1 {
            return Err(HeaderInsertError::NumberMismatch {
                expected: parent.number + 1,
                got: header.number,
            });
        }

        self.canonical.push(header.clone());
        self.index_by_hash
            .insert(header.hash.clone(), self.canonical.len() - 1);
        self.total_difficulty += header.difficulty as u128;

        Ok(())
    }

    pub fn total_difficulty(&self) -> u128 {
        self.total_difficulty
    }

    pub fn head(&self) -> Option<&BlockHeader> {
        self.canonical.last()
    }

    pub fn new(genesis: BlockHeader) -> Self {
        let hash = genesis.hash.clone();
        let difficulty = genesis.difficulty as u128;
        Self {
            canonical: vec![genesis],
            index_by_hash: HashMap::from([(hash, 0)]),
            total_difficulty: difficulty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn genesis_fixture() -> BlockHeader {
        BlockHeader {
            number: 0,
            hash: "0x1234".to_string(),
            parent_hash: "0x".to_string(),
            difficulty: 0,
        }
    }

    fn block_header_fixture(
        number: u64,
        hash: String,
        parent_hash: String,
        difficulty: u64,
    ) -> BlockHeader {
        BlockHeader {
            number,
            hash,
            parent_hash,
            difficulty,
        }
    }

    #[test]
    // total_difficulty는 제네시스와 새 헤더의 난이도를 누적해 반환해야 한다.
    fn should_get_total_difficulty() {
        let genesis = BlockHeader {
            number: 0,
            hash: "0xgenesis".to_string(),
            parent_hash: "0x".to_string(),
            difficulty: 2,
        };
        let mut buffer = HeaderBuffer::new(genesis.clone());

        let header = block_header_fixture(1, "0xaaaa".to_string(), genesis.hash.clone(), 3);
        buffer.try_append(header).expect("append should succeed");

        assert_eq!(buffer.total_difficulty(), (genesis.difficulty as u128) + 3);
    }

    #[test]
    // 생성 직후 버퍼는 제네시스 헤더만 canonical에 저장하고 인덱스도 0을 가리켜야 한다.
    fn should_create_new_header_buffer() {
        let genesis = genesis_fixture();
        let buffer = HeaderBuffer::new(genesis.clone());

        assert_eq!(buffer.canonical.len(), 1);
        assert_eq!(buffer.index_by_hash.len(), 1);
        assert_eq!(buffer.total_difficulty, genesis.difficulty as u128);

        let header = buffer.canonical.first();
        assert!(header.is_some());

        let unwrap_header = header.unwrap();
        assert_eq!(unwrap_header.difficulty, 0);
        assert_eq!(unwrap_header.number, 0);
        assert_eq!(unwrap_header.hash, "0x1234".to_string());
        assert_eq!(unwrap_header.parent_hash, "0x".to_string());

        let index_by_hash = buffer.index_by_hash;
        assert_eq!(index_by_hash.get(&unwrap_header.hash), Some(&0));
    }

    #[test]
    // 정상적인 부모-자식 관계면 헤더가 canonical 끝에 추가되어야 한다.
    fn should_append_header_when_parent_matches() {
        let mut buffer = HeaderBuffer::new(genesis_fixture());
        let new_header = block_header_fixture(1, "0x5678".to_string(), "0x1234".to_string(), 5);

        buffer
            .try_append(new_header.clone())
            .expect("append should succeed");

        assert_eq!(buffer.canonical.len(), 2);
        assert_eq!(buffer.head(), Some(&new_header));
        assert_eq!(buffer.total_difficulty, (new_header.difficulty) as u128);
    }

    #[test]
    // 이미 존재하는 해시를 가진 헤더는 중복으로 거절되어야 한다.
    fn should_reject_duplicate_hash() {
        let mut buffer = HeaderBuffer::new(genesis_fixture());
        let new_header = block_header_fixture(1, "0x5678".to_string(), "0x1234".to_string(), 1);

        buffer
            .try_append(new_header.clone())
            .expect("first append should succeed");
        let result = buffer.try_append(new_header);

        assert!(matches!(
            result,
            Err(HeaderInsertError::DuplicationHash { .. })
        ));
    }

    #[test]
    // 부모 해시를 찾을 수 없으면 ParentNotFound 에러를 내려야 한다.
    fn should_reject_when_parent_missing() {
        let mut buffer = HeaderBuffer::new(genesis_fixture());
        let orphan = block_header_fixture(1, "0xorphan".to_string(), "0xdeadbeef".to_string(), 1);

        let result = buffer.try_append(orphan);

        assert!(matches!(
            result,
            Err(HeaderInsertError::ParentNotFound { .. })
        ));
    }

    #[test]
    // 부모 넘버와 연결되지 않은 넘버는 NumberMismatch 에러가 나야 한다.
    fn should_reject_when_number_is_not_sequential() {
        let mut buffer = HeaderBuffer::new(genesis_fixture());
        let parent = block_header_fixture(1, "0xparent".to_string(), "0x1234".to_string(), 1);
        buffer
            .try_append(parent)
            .expect("parent append should succeed");

        let child = block_header_fixture(3, "0xchild".to_string(), "0xparent".to_string(), 1);
        let result = buffer.try_append(child);

        assert!(
            matches!(result, Err(HeaderInsertError::NumberMismatch { expected, got }) if expected == 2 && got == 3)
        );
    }
}
