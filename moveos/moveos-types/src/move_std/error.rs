// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
/// MoveStd's error module to rust mapping

/// Caller specified an invalid argument (http: 400)
pub const INVALID_ARGUMENT: u64 = 0x1;

/// An input or result of a computation is out of range (http: 400)
pub const OUT_OF_RANGE: u64 = 0x2;

/// The system is not in a state where the operation can be performed (http: 400)
pub const INVALID_STATE: u64 = 0x3;

/// Request not authenticated due to missing, invalid, or expired auth token (http: 401)
pub const UNAUTHENTICATED: u64 = 0x4;

/// client does not have sufficient permission (http: 403)
pub const PERMISSION_DENIED: u64 = 0x5;

/// A specified resource is not found (http: 404)
pub const NOT_FOUND: u64 = 0x6;

/// Concurrency conflict, such as read-modify-write conflict (http: 409)
pub const ABORTED: u64 = 0x7;

/// The resource that a client tried to create already exists (http: 409)
pub const ALREADY_EXISTS: u64 = 0x8;

/// Out of gas or other forms of quota (http: 429)
pub const RESOURCE_EXHAUSTED: u64 = 0x9;

/// Request cancelled by the client (http: 499)
pub const CANCELLED: u64 = 0xA;

/// Internal error (http: 500)
pub const INTERNAL: u64 = 0xB;

/// Feature not implemented (http: 501)
pub const NOT_IMPLEMENTED: u64 = 0xC;

/// The service is currently unavailable. Indicates that a retry could solve the issue (http: 503)
pub const UNAVAILABLE: u64 = 0xD;

/// Construct a canonical error code from a category and a reason.
pub const fn canonical(category: u64, reason: u64) -> u64 {
    (category << 16) + reason
}

pub fn invalid_argument(r: u64) -> u64 {
    canonical(INVALID_ARGUMENT, r)
}
pub fn out_of_range(r: u64) -> u64 {
    canonical(OUT_OF_RANGE, r)
}
pub fn invalid_state(r: u64) -> u64 {
    canonical(INVALID_STATE, r)
}
pub fn unauthenticated(r: u64) -> u64 {
    canonical(UNAUTHENTICATED, r)
}
pub fn permission_denied(r: u64) -> u64 {
    canonical(PERMISSION_DENIED, r)
}
pub fn not_found(r: u64) -> u64 {
    canonical(NOT_FOUND, r)
}
pub fn aborted(r: u64) -> u64 {
    canonical(ABORTED, r)
}
pub fn already_exists(r: u64) -> u64 {
    canonical(ALREADY_EXISTS, r)
}
pub fn resource_exhausted(r: u64) -> u64 {
    canonical(RESOURCE_EXHAUSTED, r)
}
pub fn internal(r: u64) -> u64 {
    canonical(INTERNAL, r)
}
pub fn not_implemented(r: u64) -> u64 {
    canonical(NOT_IMPLEMENTED, r)
}
pub fn unavailable(r: u64) -> u64 {
    canonical(UNAVAILABLE, r)
}

/// Explain a canonical error code into a category and a reason.
pub fn explain(abort_code: u64) -> (u64, u64) {
    if abort_code > 0xffff {
        let category = abort_code >> 16;
        let reason_code = abort_code & 0xffff;
        (category, reason_code)
    } else {
        (0, abort_code)
    }
}

pub fn explain_category(category: u64) -> Option<String> {
    match category {
        INVALID_ARGUMENT => Some("INVALID_ARGUMENT".to_string()),
        OUT_OF_RANGE => Some("OUT_OF_RANGE".to_string()),
        INVALID_STATE => Some("INVALID_STATE".to_string()),
        UNAUTHENTICATED => Some("UNAUTHENTICATED".to_string()),
        PERMISSION_DENIED => Some("PERMISSION_DENIED".to_string()),
        NOT_FOUND => Some("NOT_FOUND".to_string()),
        ABORTED => Some("ABORTED".to_string()),
        ALREADY_EXISTS => Some("ALREADY_EXISTS".to_string()),
        RESOURCE_EXHAUSTED => Some("RESOURCE_EXHAUSTED".to_string()),
        CANCELLED => Some("CANCELLED".to_string()),
        INTERNAL => Some("INTERNAL".to_string()),
        NOT_IMPLEMENTED => Some("NOT_IMPLEMENTED".to_string()),
        UNAVAILABLE => Some("UNAVAILABLE".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        let category = UNAVAILABLE;
        let reason = 0x101;
        let error_code = canonical(category, reason);
        let (explain_category, explain_reason) = explain(error_code);
        assert_eq!(category, explain_category);
        assert_eq!(reason, explain_reason);
    }
}
