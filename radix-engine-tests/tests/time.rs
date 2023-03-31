use radix_engine_interface::{blueprints::clock::TimePrecision, time::UtcDateTime};
use scrypto_unit::*;

#[test]
fn setting_single_time_succeeds() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    let time_rounded_to_minutes = UtcDateTime::new(2022, 1, 1, 0, 0, 0)
        .unwrap()
        .to_instant()
        .seconds_since_unix_epoch;

    let time_in_ms = time_rounded_to_minutes * 1000;

    // Act
    test_runner.set_current_time(time_in_ms);

    // Assert
    assert_eq!(
        test_runner
            .get_current_time(TimePrecision::Minute)
            .seconds_since_unix_epoch,
        time_rounded_to_minutes
    );
}

#[test]
fn setting_multiple_time_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let times = vec![
        UtcDateTime::new(2022, 1, 1, 0, 0, 0),
        UtcDateTime::new(2022, 2, 1, 0, 0, 0),
        UtcDateTime::new(2022, 3, 1, 0, 0, 0),
        UtcDateTime::new(2022, 4, 1, 0, 0, 0),
        UtcDateTime::new(2022, 5, 1, 0, 0, 0),
        UtcDateTime::new(2022, 6, 1, 0, 0, 0),
    ];

    for time in times.into_iter() {
        // Act
        let time_rounded_to_minutes = time.unwrap().to_instant().seconds_since_unix_epoch;

        let time_in_ms = time_rounded_to_minutes * 1000;

        test_runner.set_current_time(time_in_ms);

        // Assert
        assert_eq!(
            test_runner
                .get_current_time(TimePrecision::Minute)
                .seconds_since_unix_epoch,
            time_rounded_to_minutes
        );
    }
}
