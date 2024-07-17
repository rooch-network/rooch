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
    struct Result<T> has copy, drop{
        value: Option<T>,
        err: Option<String>,
    }


    public fun ok<T>(value: T): Result<T> {
        Result {
            value: option::some(value),
            err: option::none(),
        }
    }

    public fun is_ok<T>(result: &Result<T>): bool {
        option::is_some(&result.value)
    }

    public fun get<T>(result: &Result<T>): &Option<T> {
        &result.value
    }

    public fun err<T>(err: vector<u8>): Result<T> {
        Result {
            value: option::none(),
            err: option::some(string::utf8(err)),
        }
    }

    public fun err_string<T>(err: String): Result<T> {
        Result {
            value: option::none(),
            err: option::some(err),
        }
    }

    public fun is_err<T>(result: &Result<T>): bool {
        option::is_some(&result.err)
    }

    public fun get_err<T>(result: &Result<T>): Option<String> {
        result.err
    }

    /// Convert an error Result<T> to error Result<U>.
    public fun as_err<U, T>(self: Result<T>): Result<U> {
        let Result {
            value,
            err,
        } = self;
        assert!(option::is_none(&value), ErrorExpectErr);
        option::destroy_none(value);
        err_string(std::option::destroy_some(err))
    }

    public fun unpack<T>(result: Result<T>): (Option<T>, Option<String>) {
        let Result {
            value,
            err,
        } = result;
        (value, err)
    }

    public inline fun and_then<U, T>(result: Result<U>, f: |U|Result<T>): Result<T> {
        let (value, err) = moveos_std::result::unpack(result);
        if (std::option::is_some(&value)) {
            f(std::option::destroy_some(value))
        } else {
            moveos_std::result::err_string(std::option::destroy_some(err))   
        }
    }

    public fun unwrap<T>(result: Result<T>): T {
        let Result {
            value,
            err:_,
        } = result;
        assert!(option::is_some(&value), ErrorExpectOk);
        option::destroy_some(value)
    }

    public fun unwrap_err<T>(result: Result<T>): String {
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
    public inline fun assert_ok<T>(result: Result<T>, abort_code: u64): T{
        let (value, _err) = moveos_std::result::unpack(result);
        assert!(std::option::is_some(&value), abort_code);
        std::option::destroy_some(value)
    }

    public inline fun assert_err<T>(result: Result<T>, abort_code: u64): String{
        let (value, err) = moveos_std::result::unpack(result);
        assert!(std::option::is_some(&err), abort_code);
        std::option::destroy_none(value);
        std::option::destroy_some(err)
    }

}