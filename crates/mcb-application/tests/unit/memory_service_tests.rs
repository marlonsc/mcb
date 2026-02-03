use mcb_application::use_cases::memory_service::MemoryServiceImpl;

#[test]
fn test_current_timestamp_reports_recent_time() {
    let ts = MemoryServiceImpl::current_timestamp();
    assert!(ts > 1_700_000_000, "Timestamp should be after 2023");
    assert!(ts < 2_000_000_000, "Timestamp should be before 2033");
}
