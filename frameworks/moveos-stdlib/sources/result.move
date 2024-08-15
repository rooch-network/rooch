// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module moveos_std::result {

    use std::string::{Self, String};
    use std::option::{Self, Option};

    /// Expected the result is ok but the result is err.
    const ErrorExpectOk: u64 = 1;
    /// Expected the result is err but the result is ok.
    const ErrorExpectErr: u64 = 2;

    /// The same as Rust's Result type.
    /// Most of the time, we do not need the Result type in smart contract, we can directly abort the transaction.
    /// But in some cases, we need to return a result to ensure the caller can handle the error.
    struct Result<T, E> has copy, drop{
        value: Option<T>,
        err: Option<E>,
    }


    public fun ok<T, E>(value: T): Result<T, E> {
        Result {
            value: option::some(value),
            err: option::none(),
        }
    }

    public fun is_ok<T, E>(result: &Result<T, E>): bool {
        option::is_some(&result.value)
    }

    public fun get<T, E>(result: &Result<T, E>): &Option<T> {
        &result.value
    }

    public fun err<T, E>(err: E): Result<T, E> {
        Result {
            value: option::none(),
            err: option::some(err),
        }
    }

    /// A shortcut to create a Result<T, String> with an error String with
    /// err_str(b"msg").
    public fun err_str<T>(err: vector<u8>): Result<T, String> {
        Result {
            value: option::none(),
            err: option::some(string::utf8(err)),
        }
    }

    public fun is_err<T, E>(result: &Result<T, E>): bool {
        option::is_some(&result.err)
    }

    public fun get_err<T, E>(result: &Result<T, E>): &Option<E> {
        &result.err
    }

    /// Convert an error Result<T, String> to error Result<U, String>.
    public fun as_err<U, T>(self: Result<T, String>): Result<U, String> {
        let Result {
            value,
            err,
        } = self;
        assert!(option::is_none(&value), ErrorExpectErr);
        option::destroy_none(value);
        err(std::option::destroy_some(err))
    }

    public fun unpack<T, E>(result: Result<T, E>): (Option<T>, Option<E>) {
        let Result {
            value,
            err,
        } = result;
        (value, err)
    }

    public inline fun and_then<U, T, E>(result: Result<U, E>, f: |U|Result<T, E>): Result<T, E> {
        let (value, err) = moveos_std::result::unpack(result);
        if (std::option::is_some(&value)) {
            f(std::option::destroy_some(value))
        } else {
            moveos_std::result::err(std::option::destroy_some(err))   
        }
    }

    public fun unwrap<T, E: drop>(result: Result<T, E>): T {
        let Result {
            value,
            err:_,
        } = result;
        assert!(option::is_some(&value), ErrorExpectOk);
        option::destroy_some(value)
    }

    public fun unwrap_err<T, E>(result: Result<T, E>): E {
        let Result {
            value,
            err,
        } = result;
        assert!(option::is_some(&err), ErrorExpectErr);
        std::option::destroy_none(value);
        option::destroy_some(err)
    }

    /// Assert the result is ok, and return the value.
    /// Otherwise, abort with the abort_code.
    /// This function is inline, so it will be expanded in the caller.
    /// This ensures the abort_code is the caller's location.
    public inline fun assert_ok<T, E>(result: Result<T, E>, abort_code: u64): T{
        let (value, err) = moveos_std::result::unpack(result);
        assert!(std::option::is_some(&value), abort_code);
        std::option::destroy_none(err);
        std::option::destroy_some(value)
    }

    public inline fun assert_err<T, E>(result: Result<T, E>, abort_code: u64): E{
        let (value, err) = moveos_std::result::unpack(result);
        assert!(std::option::is_some(&err), abort_code);
        std::option::destroy_none(value);
        std::option::destroy_some(err)
    }

}