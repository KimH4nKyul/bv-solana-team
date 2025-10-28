use day3_state_helper::*;

#[test]
fn when_given_none_value_then_return_no_blocks_yet() {
    // given
    let height: Option<u64> = None;

    // when
    let result = describe_sync_height(height);

    // then
    assert_eq!(result, "No blocks yet".to_string());
}

#[test]
fn when_given_some_value_then_return_current_height() {
    // given
    let height: Option<u64> = Some(100);

    // when
    let result = describe_sync_height(height);

    // then
    assert_eq!(result, "Current height: 100".to_string());
}

#[test]
fn when_error_then_return_zero() {
    // given
    let slot: Result<u64, String> = Err(String::new());

    // when
    let result = fallback_slot(slot);

    // then
    assert_eq!(result, 0);
}

#[test]
fn when_ok_then_return_u64() {
    // given
    let slot: Result<u64, String> = Ok(200);

    // when
    let result = fallback_slot(slot);

    // then
    assert_eq!(result, 200);
}

#[test]
fn when_ms_under_150_then_result_is_instant() {
    // given
    let ms: u64 = 150;

    // when
    let result = classify_peer_speed(ms);

    // then
    assert_eq!(result, "Instant");
}

#[test]
fn when_ms_151_then_result_is_acceptable() {
    // given
    let ms: u64 = 151;

    // when
    let result = classify_peer_speed(ms);

    // then
    assert_eq!(result, "Acceptable");
}

#[test]
fn when_ms_under_400_then_result_is_acceptable() {
    // given
    let ms: u64 = 400;

    // when
    let result = classify_peer_speed(ms);

    // then
    assert_eq!(result, "Acceptable");
}

#[test]
fn when_ms_over_400_then_result_is_lagging() {
    // given
    let ms: u64 = 401;

    // when
    let result = classify_peer_speed(ms);

    // then
    assert_eq!(result, "Lagging");
}
