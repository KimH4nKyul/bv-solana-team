use day1_variables::*;

#[test]
fn test_describe_mutability() {
    let result = describe_mutability();
    assert_eq!(result, "let _immutable = String::from(...) 같은 immutable(불변 변수)는 재할당이 일어나면 컴파일 타임에 에러가 발생하지만 let mut _mutable: String = String::from(...) 같은 mutable(가변 변수)는 재할당이 가능하다. 그 결과 mutable은 mutable에서 new mutable가 된다.");
}

#[test]
fn test_shadow_example() {
    let result = shadow_example();
    assert_eq!(
        result,
        vec![
            "[1, 2, 3]".to_string(),
            "[\"1\", \"2\", \"3\"]".to_string(),
            "1, 2, 3".to_string(),
        ]
    );
}


#[test]
fn test_move_semantics_demo() {
    let result = move_semantics_demo();
    assert_eq!(
        result.err(),
        Some("a는 소유권(ownership)이 _b로 이동(move)되었기 때문에 사용될 수 없다.".to_string())
    )
}

