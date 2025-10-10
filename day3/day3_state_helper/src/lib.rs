pub fn describe_sync_height(height: Option<u64>) -> String {
    match height {
        Some(value) => format!("Current height: {value}"),
        None => "No blocks yet".to_string(),
    }
}

pub fn fallback_slot(slot: Result<u64, String>) -> u64 {
    // slot.unwarp_or_else(|_| 0)
    // 클로저 사용에 대한 전형적인 안티 패턴으로, 익명 함수 호출이라는 불필요한 연산을 수행하려 하므로,
    // 빠르게 값을 반환하는 early evaluation 을 아래와 같이 채택해야만 한다.
    slot.unwrap_or(0)
}

pub fn classify_peer_speed(ms: u64) -> &'static str {
    match ms {
        0..=150 => "Instant",
        151..=400 => "Acceptable",
        401.. => "Lagging",
    }
}
