// Rx -- Reactive programming for Rust
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

extern crate rx;

use rx::{Never, Observable, Observer, Subject};
use std::cell::RefCell;
use std::rc::Rc;

// Generator tests

#[test]
fn never() {
    let mut never = Never::new();
    let _subscription = never.subscribe_error(
        |_x: u8| panic!("never observable should not produce a value"),
        || panic!("never observable should not complete"),
        |_err: ()| panic!("never observable should not fail")
    );

    // Without something like a message loop, the observable cannot suddenly
    // start pushing values, so if it did not produce anything here, it never
    // will.
}

// Option tests

#[test]
fn option_subscribe_next() {
    let mut received = None;

    // Subscribing to `Some` should push the value.
    Some(19).subscribe_next(|x| received = Some(x));
    assert_eq!(Some(19), received);

    None.subscribe_next(|_x: u32| panic!("none should not push a value"));
}

#[test]
fn option_subscribe_completed() {
    let mut received = None;
    let mut completed = false;

    // Subscribing to `Some` should complete after pushing the value.
    Some(19).subscribe_completed(|x| received = Some(x), || completed = true);
    assert_eq!(Some(19), received);
    assert!(completed);

    // Subscribing to `None` should complete without pushing a value.
    completed = false;
    None.subscribe_completed(
        |_x: u32| panic!("none should not push a value"),
        || completed = true
    );
    assert!(completed);
}

#[test]
fn option_subscribe_error() {
    let mut received = None;
    let mut completed = false;

    Some(23).subscribe_error(
        |x| received = Some(x),
        || completed = true,
        |_err| panic!("some observable should not fail")
    );
    assert_eq!(Some(23), received);
    assert!(completed);

    completed = false;
    None.subscribe_error(
        |_x: u32| panic!("none should not push a value"),
        || completed = true,
        |_err| panic!("none observable should not fail")
    );
    assert!(completed);
}

// Result tests

#[test]
fn result_subscribe_next_ok() {
    let mut result: Result<u32, ()> = Ok(13);
    let mut received = None;
    result.subscribe_next(|x| received = Some(x));
    assert_eq!(Some(13), received);
}

#[test]
#[should_panic]
fn result_subscribe_next_err() {
    let mut result: Result<u32, ()> = Err(());
    let mut received = None;

    // This should panic, because we did not provide an error handler.
    result.subscribe_next(|x| received = Some(x));
}

#[test]
fn result_subscribe_completed_ok() {
    let mut result: Result<u32, ()> = Ok(13);
    let mut received = None;
    let mut completed = false;
    result.subscribe_completed(|x| received = Some(x), || completed = true);
    assert_eq!(Some(13), received);
    assert!(completed);
}

#[test]
fn result_subscribe_error_ok() {
    let mut result: Result<u32, ()> = Ok(13);
    let mut received = None;
    let mut completed = false;
    result.subscribe_error(
        |x| received = Some(x),
        || completed = true,
        |_err| panic!("ok result should not be a failing observable")
    );
    assert_eq!(Some(13), received);
    assert!(completed);
}

#[test]
fn result_subscribe_error_err() {
    let mut result: Result<(), u32> = Err(17);
    let mut error = None;
    result.subscribe_error(
        |_x| panic!("err result should not push a value"),
        || panic!("err result should not complete"),
        |err| error = Some(err)
    );
    assert_eq!(Some(17), error);
}

// Slice tests

#[test]
fn slice_subscribe_next() {
    let mut values = &[2u8, 3, 5, 7, 11, 13];
    let mut received = Vec::new();
    values.subscribe_next(|&x| received.push(x));
    assert_eq!(&values[..], &received[..]);
}

#[test]
fn slice_subscribe_completed() {
    let mut values = &[2u8, 3, 5, 7, 11, 13];
    let mut received = Vec::new();
    let mut completed = false;
    values.subscribe_completed(|&x| received.push(x), || completed = true);
    assert_eq!(&values[..], &received[..]);
    assert!(completed);
}

#[test]
fn slice_subscribe_error() {
    let mut values = &[2u8, 3, 5, 7, 11, 13];
    let mut received = Vec::new();
    let mut completed = false;
    let mut failed = false;
    values.subscribe_error(|&x| received.push(x), || completed = true, |_err| failed = true);
    assert_eq!(&values[..], &received[..]);
    assert!(completed);
    assert!(!failed);
}

#[test]
fn slice_subscribe_option() {
    let mut values = &[2u8, 3, 5, 7, 11, 13];
    let expected = &[Some(2u8), Some(3), Some(5), Some(7), Some(11), Some(13), None];
    let mut received = Vec::new();
    values.subscribe_option(|x| received.push(x.cloned()));
    assert_eq!(&received[..], &expected[..]);
}

#[test]
fn slice_subscribe_result() {
    let mut values = &[2u8, 3, 5, 7];
    let expected = &[Ok(Some(2u8)), Ok(Some(3)), Ok(Some(5)), Ok(Some(7)), Ok(None)];
    let mut received = Vec::new();
    values.subscribe_result(|x| received.push(x.map(|y| y.cloned())));
    assert_eq!(&received[..], &expected[..]);
}

// Subject tests

#[test]
fn subject_on_next() {
    let mut subject = Subject::<u8, ()>::new();
    let mut received = Vec::new();
    let _subscription = subject.observable().subscribe_next(|x| received.push(x));

    // Subject should not push anything upon subscription.
    assert_eq!(0, received.len());

    let values = &[2u8, 3, 5, 7, 11, 13];
    for i in 0..values.len() {
        subject.on_next(values[i]);
        assert_eq!(&values[..i + 1], &received[..]);
    }
}

#[test]
fn subject_on_completed() {
    let mut subject = Subject::<u8, ()>::new();
    let mut completed = false;
    let _subscription = subject.observable().subscribe_completed(
        |_x| panic!("no value should be pushed"),
        || completed = true
    );

    // Subject should not push anything upon subscription.
    assert!(!completed);

    subject.on_completed();
    assert!(completed);
}

#[test]
fn subject_on_error() {
    let mut subject = Subject::<u8, u8>::new();
    let mut error = 0;
    let _subscription = subject.observable().subscribe_error(
        |_x| panic!("no value should be pushed"),
        || panic!("subject should not complete"),
        |err| error = err
    );

    // Subject should not fail upon subscription.
    assert_eq!(0, error);

    subject.on_error(41);
    assert_eq!(41, error);
}

/// Helper for the `subject_clones_once_per_observer()` test.
struct CloneCounter {
    counter: Rc<RefCell<u32>>,
}

impl Clone for CloneCounter {
    fn clone(&self) -> CloneCounter {
        let count: u32 = *self.counter.borrow();
        *self.counter.borrow_mut() = count + 1;
        CloneCounter {
            counter: self.counter.clone(),
        }
    }
}

#[test]
fn subject_clones_once_per_observer() {
    let mut subject = Subject::<CloneCounter, ()>::new();
    let mut first_called = false;
    let mut second_called = false;
    let counter = CloneCounter {
        counter: Rc::new(RefCell::new(0)),
    };

    // Subscribe twice.
    let _s1 = subject.observable().subscribe_next(|_x| first_called = true);
    let _s2 = subject.observable().subscribe_next(|_x| second_called = true);

    // Nothing should have been cloned yet.
    assert_eq!(0, *counter.counter.borrow());

    subject.on_next(counter.clone());

    // We cloned once, and the subject should have cloned once per subscription.
    assert_eq!(3, *counter.counter.borrow());
    assert!(first_called);
    assert!(second_called);
}

#[test]
fn subject_drop_subscription() {
    let mut subject = Subject::<u8, ()>::new();
    let mut received = Vec::new();
    let subscription = subject.observable().subscribe_next(|x| received.push(x));

    subject.on_next(2);
    subject.on_next(3);
    subject.on_next(5);

    assert_eq!(&[2u8, 3, 5], &received[..]);

    drop(subscription);

    subject.on_next(7);
    subject.on_next(11);

    // Values pushed after drop should not have been invoked.
    assert_eq!(&[2u8, 3, 5], &received[..]);
}

#[test]
fn subject_drop_subscription_multi() {
    let mut subject = Subject::<u8, ()>::new();
    let mut received = Vec::new();
    let _s1 = subject.observable().subscribe_next(|x| received.push(x));
    let s2 = subject.observable().subscribe_error(
        |_x| panic!("no value should be pushed after dropping subscription"),
        || panic!("completion should not be signalled after dropping subscription"),
        |_err| panic!("failure should not be signalled after dropping subscription")
    );

    drop(s2);

    subject.on_next(2);
    subject.on_next(3);
    subject.on_next(5);
    subject.on_completed();

    assert_eq!(&[2u8, 3, 5], &received[..]);
}

// TODO: Add a better test to test internal removal of observer from the list.

#[test]
fn subject_drop_in_handler() {
    let mut subject = Subject::<u8, ()>::new();
    let mut received = Vec::new();
    let mut subscription_opt: Option<Box<Drop>> = None;
    let subscription = subject.observable().subscribe_next(|x| {
        received.push(x);
        assert!(subscription_opt.is_some());
        drop(subscription_opt.take().unwrap());
    });
    // TODO: this assignment is not unused, or this code doing something illegal.
    // Either way, this is a bug in rustc.
    subscription_opt = Some(Box::new(subscription));

    subject.on_next(2);
    assert_eq!(&[2u8], &received[..]);

    subject.on_next(3);
    assert_eq!(&[2u8], &received[..]);
}

#[test]
fn subject_continue_with() {
    use std::mem;
    let mut first = Subject::<u8, ()>::new();
    let mut second = Subject::<u8, ()>::new();
    let mut received = Vec::new();
    let mut completed = false;
    {
        let subscription = first.observable()
            .continue_with(&mut second.observable())
            .subscribe_completed(|x| received.push(x), || completed = true);

        // TODO: How can I keep this alive without the compiler complaining about borrows?
        mem::forget(subscription);
    }

    first.on_next(2);
    assert_eq!(&[2u8][..], &received[..]);

    // If `second` produces a value, it should not be pushed,
    // because `first` has not yet completed.
    second.on_next(3);
    assert_eq!(&[2u8][..], &received[..]);

    first.on_next(5);
    assert_eq!(&[2u8, 5][..], &received[..]);

    // Completing `first` should not complete the continuation,
    // nor should it push a value.
    first.on_completed();
    assert_eq!(&[2u8, 5][..], &received[..]);
    assert!(!completed);

    // Now pushing to `second` should have an effect, because `first` completed.
    second.on_next(7);
    assert_eq!(&[2u8, 5, 7][..], &received[..]);

    second.on_completed();
    assert!(completed);
}

// TODO: Test multiple subscriptions and combinations of values and completed/error.
// TODO: Add better tests for dropping the subject subscription.

// Transform tests

#[test]
fn map() {
    let mut values = &[2u8, 3, 5, 7, 11, 13];
    let expected = &[4u8, 6, 10, 14, 22, 26];
    let mut received = Vec::new();
    let mut mapped = values.map(|x| x * 2);
    mapped.subscribe_next(|x| received.push(x));
    assert_eq!(&expected[..], &received[..]);
}

#[test]
fn map_does_not_change_error() {
    let mut error = None;
    let mut mapped = Err(23_u32).map(|x: u32| format!("{}", x));
    mapped.subscribe_error(
        |_x: String| panic!("mapped error should not produce a value"),
        || panic!("mapped error should not complete"),
        |err| error = Some(err)
    );
    assert_eq!(Some(23), error);
}

#[test]
fn map_error() {
    let mut error = None;
    let mut observable = Err(23_u32);
    let mut mapped = observable.map_error(|x| x * 2);
    mapped.subscribe_error(
        |_x: u32| panic!("mapped error should not produce a value"),
        || panic!("mapped error should not complete"),
        |err| error = Some(err)
    );
    assert_eq!(Some(46), error);
}

#[test]
fn map_error_does_not_change_values() {
    let mut values = &[2u8, 3, 5, 7, 11, 13];
    let mut received = Vec::new();
    let mut mapped = values.map_error(|_unit| 17u8);
    mapped.subscribe_next(|&x| received.push(x));
    assert_eq!(&values[..], &received[..]);
}

#[test]
fn continue_with() {
    let (mut first, mut second) = (&[2u8, 3, 5, 7], &[11u8, 13, 17, 19]);
    let expected = &[2u8, 3, 5, 7, 11, 13, 17, 19];
    let mut received = Vec::new();
    let mut continued = first.continue_with(&mut second);
    continued.subscribe_next(|&x| received.push(x));
    assert_eq!(&expected[..], &received[..]);
}
