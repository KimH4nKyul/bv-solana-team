pub fn annotate_borrowing(message: &str) -> String {
    format!(
        "Rust가 `&str`을 불변 참조(immutable reference)로 빌릴 때, 원본 메시지 `{msg}`는 여전히 그대로 남습니다.\n\
         - immutable reference 는 여러 스레드가 동시에 읽어도 no data race 를 보장합니다.\n\
         - 소유권(ownership) 이동 없이 참조만 공유하므로 블록체인 노드 상태를 읽기 전용으로 확인할 때 안전합니다.",
        msg = message
    )
}

/**
balance는 가변 참조로 하나만 존재할 수 있고, 동시에 참조될 수 없기 때문에
누군가 balance를 읽고 있다면 다른 누군가는 이에 대한 쓰기 연산을 수행할 수 없다.
이는 Rust의 Borrow Checker 규칙이 적용된 결과이다.
*/
pub fn mutate_wallet(balance: &mut i64, delta: i64) -> Result<(), String> {
    /*****
    // 오버플로 안전 처리
    let new_balance = balance.checked_add(delta).ok_or_else(|| "overflow while applying delta".to_owned())?;
    if new_balance < 0 { return Err("balance would become negative".to_owned()) }

    // 실제 빌림은 단일 가변 빌림 스코프에서만 수행한다.
    {
        // 이 블록이 가변 빌림의 유효 범위이다.
        // 동일 시점에 다른 불변/가변 참조는 허락되지 않는다.
        *balance = new_balance;
    } // <- 여기서 가변 빌림은 끝이 난다.

    // 이제 불변 빌림이 안전하게 가능해 진다. (읽기 전용)
    // 아래는 읽어서 어떤 부수 작업을 할 수 있음을 확인한다.
    let _snapshot: &i64 = &*balance;
    let _ = *_snapshot;

    Ok(())

    // 이렇게 해서 가변 빌림을 하나의 내부 블록으로 제한해 Borrow Checker 규칙을 코드 구조로 드러내고,
    // 에러는 Result로 전파한다.
    // 오버플로우/언더플로우를 방어해 안정성을 높인다.
    *****/

    // balance는 이 함수의 스코프에서 하나의 빌림(borrowing)만 유효하다.
    if *balance + delta < 0 {
        return Err("Balance is negative".to_owned());
    }
    *balance += delta;
    Ok(())
}

/**
블록 높이 목록을 슬라이스로 받아,
앞부분 3개의 요소만 가리키는 서브 슬라이스(3개 미만이면 가능한 만큼)와
총 요소 수를 함께 반환합니다.
*/
pub fn summarize_slice<'a>(blocks: &'a [u64]) -> (&'a [u64], usize) {
    // 앞에 3개 요소만 가리키는 서브 슬라이스(3개 미만이면 가능한 만큼)
    (&blocks[..std::cmp::min(3, blocks.len())], blocks.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    //`annotate_borrowing` 결과가 원본 문자열을 소비하지 않고, 필수 키워드를 포함하는지 테스트하세요.
    fn test_annotate_borrowing() {
        let note = "staking ledger";
        let annotated = annotate_borrowing(note);

        assert!(annotated.contains("immutable reference"));
        assert!(annotated.contains("no data race"));
        assert!(annotated.contains(note));
        assert_eq!("staking ledger", note);
    }

    #[test]
    //`mutate_wallet`이 정상적인 증감과 음수 잔액 방지 로직을 올바르게 처리하는지 `Result` 비교로 검증하세요.
    fn test_mutate_wallet() {
        // 정상적인 증감
        // given
        let mut balance = 1000;

        // when
        let result = mutate_wallet(&mut balance, 1);

        // then
        assert!(result.is_ok());
        assert_eq!(balance, 1001)
    }

    #[test]
    fn test_mutate_wallet_err() {
        let result = mutate_wallet(&mut 10, -11);
        assert!(result.is_err());
    }

    #[test]
    //`summarize_slice`가 슬라이스 참조를 복사 없이 공유한다는 것을 보여 주기 위해,
    // 원본 배열 값을 변경하면 반환된 슬라이스에도 반영되는지 테스트하세요.
    fn test_summarize_slice() {
        // 먼저 슬라이스 참조는 메모리의 영역 중에 어디에 올라가는 거지?
        // 보통 String literal의 슬라이스는 데이터 영역의 배열에 그대로 올라가고,
        // 그에 대해 참조할 수 있는 데이터는 ptr과 len이 스택 프레임에 올라가잖아?

        // given
        let mut blocks = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        // when
        let _task_one = {
            let (slice, len) = summarize_slice(&blocks);
            assert_eq!(9, len);
            assert_eq!(&[1, 2, 3], slice);
        };
        blocks[0] = 10;

        // then
        let (slice, len) = summarize_slice(&blocks);
        assert_eq!(9, len);
        assert_eq!(&[10, 2, 3], slice);
    }
}
