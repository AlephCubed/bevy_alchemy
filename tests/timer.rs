//! Tests the behaviour of timers for each [`TimerMergeMode`].

use bevy_status_effects::*;

#[test]
fn merge_replace() {
    let first = Lifetime::from_seconds(1.0).with_mode(TimerMergeMode::Replace);
    let second = Lifetime::from_seconds(2.0).with_mode(TimerMergeMode::Replace);
    let mut result = second.clone();
    result.merge(&first);

    assert_eq!(result, second);
}

#[test]
fn merge_inherit() {
    let first = Lifetime::from_seconds(1.0).with_mode(TimerMergeMode::Inherit);
    let second = Lifetime::from_seconds(2.0).with_mode(TimerMergeMode::Inherit);
    let mut result = second.clone();
    result.merge(&first);

    assert_eq!(result, first);
}

#[test]
fn merge_fraction() {
    let first = Lifetime::from_seconds(1.0).with_mode(TimerMergeMode::Fraction);
    let second = Lifetime::from_seconds(2.0).with_mode(TimerMergeMode::Fraction);
    let mut result = second.clone();
    result.merge(&first);

    assert_eq!(result, second);
}

#[test]
fn merge_max() {
    let first = Lifetime::from_seconds(1.0).with_mode(TimerMergeMode::Max);
    let mut second = Lifetime::from_seconds(3.0).with_mode(TimerMergeMode::Max);
    second.merge(&first);
    let third = Lifetime::from_seconds(2.0).with_mode(TimerMergeMode::Max);

    let mut result = third.clone();
    result.merge(&second);

    assert_eq!(result, second);
}
