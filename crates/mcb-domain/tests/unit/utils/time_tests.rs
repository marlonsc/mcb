use mcb_domain::utils::time::{epoch_nanos_u128, epoch_secs_i64, epoch_secs_u64};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn epoch_secs_i64_returns_positive() -> TestResult {
    let ts = epoch_secs_i64()?;
    assert!(ts > 0, "timestamp should be positive, got {ts}");
    Ok(())
}

#[test]
fn epoch_secs_u64_returns_positive() -> TestResult {
    let ts = epoch_secs_u64()?;
    assert!(ts > 0, "timestamp should be positive, got {ts}");
    Ok(())
}

#[test]
fn epoch_nanos_u128_returns_positive() -> TestResult {
    let nanos = epoch_nanos_u128()?;
    assert!(nanos > 0, "nanos should be positive, got {nanos}");
    Ok(())
}

#[test]
fn epoch_secs_i64_monotonic() -> TestResult {
    let a = epoch_secs_i64()?;
    let b = epoch_secs_i64()?;
    assert!(b >= a, "expected monotonic: {b} >= {a}");
    Ok(())
}

#[test]
fn epoch_secs_u64_monotonic() -> TestResult {
    let a = epoch_secs_u64()?;
    let b = epoch_secs_u64()?;
    assert!(b >= a, "expected monotonic: {b} >= {a}");
    Ok(())
}

#[test]
fn epoch_nanos_u128_monotonic() -> TestResult {
    let a = epoch_nanos_u128()?;
    let b = epoch_nanos_u128()?;
    assert!(b >= a, "expected monotonic: {b} >= {a}");
    Ok(())
}

#[test]
fn epoch_secs_i64_reasonable_range() -> TestResult {
    use mcb_domain::constants::time::{TIMESTAMP_MAX_BOUNDARY, TIMESTAMP_MIN_BOUNDARY};
    let ts = epoch_secs_i64()?;
    // Should be after 2020-01-01 and before 2100-01-01
    assert!(ts > TIMESTAMP_MIN_BOUNDARY, "timestamp too old: {ts}");
    assert!(
        ts < TIMESTAMP_MAX_BOUNDARY,
        "timestamp too far in future: {ts}"
    );
    Ok(())
}
