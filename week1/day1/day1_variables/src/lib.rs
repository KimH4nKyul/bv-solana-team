pub fn describe_mutability() -> String {
    // immutable(불변 변수)
    let _immutable: String = String::from("immutable");
    // mutable(가변 변수)
    let mut _mutable: String = String::from("mutable");

    // _immutable = String::from("new immutable"); // immutable(불변 변수)에 재할당이 일어나면 컴파일 타임에 에러가 발생한다.
    _mutable = String::from("new mutable"); // 반면 mutable(가변 변수)는 재할당이 가능하다.
    format!("let _immutable = String::from(...) 같은 immutable(불변 변수)는 재할당이 일어나면 컴파일 타임에 에러가 발생하지만 let mut _mutable: String = String::from(...) 같은 mutable(가변 변수)는 재할당이 가능하다. 그 결과 mutable은 mutable에서 {}가 된다.", _mutable)
}

pub fn shadow_example() -> Vec<String> {
    let mut history = Vec::new();

    // 1단계: Vec<i32>
    let v: Vec<i32> = vec![1, 2, 3];
    history.push(format!("{:?}", v));

    // 2단계: Vec<String> (shadowing)
    let v: Vec<String> = v.iter().map(|x| x.to_string()).collect();
    history.push(format!("{:?}", v));

    // 3단계: String (shadowing)
    let v: String = v.join(", ");
    history.push(v.clone());

    history
}

pub fn move_semantics_demo() -> Result<(), String> {
    let a: String = String::from("a");
    let _b = a; // 여기서 소유권(ownership) 이전이 발생한다.

    // println!("a: {}, b: {}", a, b); // a는 소유권(ownership)이 b로 이동(move)되었기 때문에 사용될 수 없다.
    Err("a는 소유권(ownership)이 _b로 이동(move)되었기 때문에 사용될 수 없다.".to_string())
}