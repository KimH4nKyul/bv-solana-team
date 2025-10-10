[잘못된 부분]
- 확인된 문제 없음

[개선하면 좋은 부분]
- `fastest_peer` 통합 테스트에 가장 빠른 피어가 첫 번째가 아닌 경우도 추가 검증하면 회귀 방지가 더 탄탄해집니다.
- `count_uninitialized`에서는 `if let None = peer.last_slot` 같은 패턴 매칭을 활용해 보면 `Option` 처리 감각을 더 키울 수 있습니다.
