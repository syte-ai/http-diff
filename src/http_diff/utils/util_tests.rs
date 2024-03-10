use std::time::Duration;

use super::utils::prettify_duration;

#[test]
pub fn test_less_than_a_second() {
    let input = Duration::from_millis(400);

    let actual = prettify_duration(input);

    let expected = "0.4 seconds".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_second() {
    let input = Duration::from_secs(1);

    let actual = prettify_duration(input);

    let expected = "1 second".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_less_than_a_minute() {
    let input = Duration::from_secs(29);

    let actual = prettify_duration(input);

    let expected = "29 seconds".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_a_minute() {
    let input = Duration::from_secs(60);

    let actual = prettify_duration(input);

    let expected = "1 minute".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_10_minutes() {
    let input = Duration::from_secs(60 * 10);

    let actual = prettify_duration(input);

    let expected = "10 minutes".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_minutes_and_seconds() {
    let input = Duration::from_secs((60 * 20) + 35);

    let actual = prettify_duration(input);

    let expected = "20 minutes and 35 seconds".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_minutes_and_one_second() {
    let input = Duration::from_secs((60 * 45) + 1);

    let actual = prettify_duration(input);

    let expected = "45 minutes and 1 second".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_one_hour() {
    let input = Duration::from_secs(60 * 60);

    let actual = prettify_duration(input);

    let expected = "1 hour".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_one_hour_and_minutes() {
    let input = Duration::from_secs(60 * 64);

    let actual = prettify_duration(input);

    let expected = "1 hour and 4 minutes".to_owned();

    assert_eq!(actual, expected)
}

#[test]
pub fn test_hours_minutes_and_seconds() {
    let input = Duration::from_secs(((60 * 60) * 12) + (60 * 37) + 43);

    let actual = prettify_duration(input);

    let expected = "12 hours, 37 minutes and 43 seconds".to_owned();

    assert_eq!(actual, expected)
}
