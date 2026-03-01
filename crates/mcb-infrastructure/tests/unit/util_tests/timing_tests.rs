//! Timing Utility Tests

use std::thread::sleep;
use std::time::Duration;

use mcb_infrastructure::utils::TimedOperation;
use rstest::rstest;

#[rstest]
#[test]
fn test_timed_operation() {
    let timer = TimedOperation::start();
    sleep(Duration::from_millis(10));
    assert!(timer.elapsed_ms() >= 10);
}

#[rstest]
#[test]
fn test_elapsed_secs() {
    let timer = TimedOperation::start();
    sleep(Duration::from_millis(100));
    assert!(timer.elapsed_secs() >= 0.1);
}
