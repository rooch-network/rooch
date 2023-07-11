// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use futures::future::BoxFuture;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

/// Simply evaluates expr.
#[macro_export]
macro_rules! nondeterministic {
    ($expr: expr) => {
        $expr
    };
}

type FpCallback = dyn Fn() -> Option<BoxFuture<'static, ()>> + Send + Sync + 'static;
type FpMap = HashMap<&'static str, Arc<FpCallback>>;

fn with_fp_map<T>(func: impl FnOnce(&mut FpMap) -> T) -> T {
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static MAP: Lazy<Mutex<FpMap>> = Lazy::new(Default::default);
    let mut map = MAP.lock().unwrap();
    func(&mut map)
}

fn get_callback(identifier: &'static str) -> Option<Arc<FpCallback>> {
    with_fp_map(|map| map.get(identifier).cloned())
}

pub fn handle_fail_point(identifier: &'static str) {
    if let Some(callback) = get_callback(identifier) {
        tracing::error!("hit failpoint {}", identifier);
        assert!(
            callback().is_none(),
            "sync failpoint must not return future"
        );
    }
}

pub async fn handle_fail_point_async(identifier: &'static str) {
    if let Some(callback) = get_callback(identifier) {
        tracing::error!("hit async failpoint {}", identifier);
        let fut = callback().expect("async callback must return future");
        fut.await;
    }
}

fn register_fail_point_impl(
    identifier: &'static str,
    callback: Arc<dyn Fn() -> Option<BoxFuture<'static, ()>> + Sync + Send + 'static>,
) {
    with_fp_map(move |map| {
        assert!(
            map.insert(identifier, callback).is_none(),
            "duplicate fail point registration"
        );
    })
}

pub fn register_fail_point(identifier: &'static str, callback: impl Fn() + Sync + Send + 'static) {
    register_fail_point_impl(
        identifier,
        Arc::new(move || {
            callback();
            None
        }),
    );
}

pub fn register_fail_point_async<F>(
    identifier: &'static str,
    callback: impl Fn() -> F + Sync + Send + 'static,
) where
    F: Future<Output = ()> + Sync + Send + 'static,
{
    register_fail_point_impl(identifier, Arc::new(move || Some(Box::pin(callback()))));
}

pub fn register_fail_points(
    identifiers: &[&'static str],
    callback: impl Fn() + Sync + Send + 'static,
) {
    let cb = Arc::new(move || {
        callback();
        None
    });
    for id in identifiers {
        register_fail_point_impl(id, cb.clone());
    }
}

#[cfg(not(any(fail_points)))]
#[macro_export]
macro_rules! fail_point {
    ($tag: expr) => {};
}

#[cfg(not(any(fail_points)))]
#[macro_export]
macro_rules! fail_point_async {
    ($tag: expr) => {};
}

// These tests need to be run in release mode, since debug mode does overflow checks by default!
#[cfg(test)]
mod test {
    // use super::*;

    // Uncomment to test error messages
    // #[with_checked_arithmetic]
    // struct TestStruct;

    macro_rules! pass_through {
        ($($tt:tt)*) => {
            $($tt)*
        }
    }

    #[test]
    fn test_skip_checked_arithmetic() {
        // comment out this attr to test the error message
        pass_through! {
            fn unchecked_add(a: i32, b: i32) -> i32 {
                a + b
            }
        }

        // this will not panic even if we pass in (i32::MAX, 1), because we skipped processing
        // the item macro, so we also need to make sure it doesn't panic in debug mode.
        unchecked_add(1, 2);
    }
}
