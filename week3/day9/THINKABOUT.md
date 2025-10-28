# THINKABOUT

1. **메모리 전용 버퍼의 한계**  
   - 긴 체인을 동기화할 때 메모리에만 canonical 체인을 두면 어떤 동기화·복구 문제가 생길까요?  
   - 디스크 기반 DB를 병행할 때 발생하는 일관성 이슈(예: 프로그램 재시작, 부분 동기화 실패)를 어떻게 완화할 수 있을지 생각해 보세요.

2. **Stage 파이프라인 연결 고리**  
   - 헤더 Stage가 완료된 뒤 Body Stage, Execution Stage는 각각 어떤 추가 데이터를 필요로 할까요?  
   - 우리가 만든 헤더 버퍼에서 어떤 인터페이스를 노출해야 다음 Stage가 효율적으로 동작할지 설계 관점에서 정리해 보세요.

3. **체인 선택 규칙의 진화**  
   - PoS 전환 이후 Ethereum이 total difficulty 외에 어떤 기준(예: terminal total difficulty, finalized checkpoint, fork choice rule)을 활용하는지 조사해 보세요.  
   - 이러한 규칙이 코드 차원에서 어떤 추가 상태 추적을 요구하는지도 함께 고민해 봅니다.
