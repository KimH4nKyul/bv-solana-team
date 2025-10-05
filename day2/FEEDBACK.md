# Day 2 Feedback

## 잘못된 부분
- 요구사항에서 지정한 통합 테스트 파일 `tests/borrowing.rs`가 여전히 존재하지 않고, 모든 검증이 `src/lib.rs` 내부 유닛 테스트로만 작성되어 요구 조건을 충족하지 못합니다 (`day2/day2_ownership/src/lib.rs:63`).
- `annotate_borrowing` 테스트가 필수 키워드 `"immutable reference"`, `"no data race"`를 확인하지 않고, 원본 문자열이 소모되지 않았는지도 증명하지 못했습니다 (`day2/day2_ownership/src/lib.rs:67`).
- `mutate_wallet` 정상 경로 테스트가 결과값을 단순히 `is_ok`로만 검사해 함수가 잔액을 올바르게 변경했는지 확인하지 않습니다 (`day2/day2_ownership/src/lib.rs:73`).

## 개선하면 좋은 부분
- `mutate_wallet`에서 음수 판별 전에 `checked_add`를 사용하면 오버플로우를 방지하면서 요구된 로직을 더 안전하게 표현할 수 있습니다 (`day2/day2_ownership/src/lib.rs:45`).
- `summarize_slice` 테스트에서 사용한 블록 스코프 내 변수 `task_one`은 사용되지 않으므로 `_task_one` 등으로 경고를 제거하면 깔끔합니다 (`day2/day2_ownership/src/lib.rs:99`).
