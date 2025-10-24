use reth_header_buffer::{BlockHeader, HeaderBuffer, HeaderInsertError};

fn make_header(number: u64, hash: &str, parent_hash: &str, difficulty: u64) -> BlockHeader {
    BlockHeader {
        number,
        hash: hash.to_string(),
        parent_hash: parent_hash.to_string(),
        difficulty,
    }
}

#[test]
// 여러 개의 헤더를 순서대로 추가하면 head와 난이도가 정확히 갱신되어야 한다.
fn appends_headers_in_sequence() {
    let genesis = make_header(0, "0xgenesis", "0x", 3);
    let mut buffer = HeaderBuffer::new(genesis.clone());

    let header1 = make_header(1, "0x1", "0xgenesis", 4);
    let header2 = make_header(2, "0x2", "0x1", 5);

    buffer
        .try_append(header1.clone())
        .expect("header1 append should succeed");
    buffer
        .try_append(header2.clone())
        .expect("header2 append should succeed");

    assert_eq!(buffer.head(), Some(&header2));
    assert_eq!(
        buffer.total_difficulty(),
        (genesis.difficulty + header1.difficulty + header2.difficulty) as u128
    );
}

#[test]
// 부모 해시는 존재하지만 넘버가 맞지 않으면 NumberMismatch 에러가 나야 한다.
fn rejects_out_of_sequence_number() {
    let genesis = make_header(0, "0xgenesis", "0x", 1);
    let mut buffer = HeaderBuffer::new(genesis.clone());

    let parent = make_header(1, "0xparent", "0xgenesis", 1);
    buffer
        .try_append(parent.clone())
        .expect("parent append should succeed");

    let invalid_child = make_header(3, "0xchild", "0xparent", 1);
    let result = buffer.try_append(invalid_child);

    assert!(
        matches!(result, Err(HeaderInsertError::NumberMismatch { expected, got }) if expected == 2 && got == 3)
    );
}
