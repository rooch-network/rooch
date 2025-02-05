module twitter_binding::string_util {

    use std::vector;

    friend twitter_binding::tweet_fetcher;
    friend twitter_binding::twitter_account;

    //TODO migrate to std::string::starts_with
    public(friend) fun starts_with(haystack: &vector<u8>, needle: &vector<u8>): bool {
        let haystack_len = vector::length(haystack);
        let needle_len = vector::length(needle);

        if (needle_len > haystack_len) {
            return false
        };

        let i = 0;
        while (i < needle_len) {
            if (vector::borrow(haystack, i) != vector::borrow(needle, i)) {
                return false
            };
            i = i + 1;
        };

        true
    }

    const MOVE_PREFIX: vector<u8> = b"b'";
    // the ascii value of the single quote character "'"
    const MOVE_SUFFIX: u8 = 39;

    public(friend) fun try_remove_move_string_prefix(bytes: vector<u8>): vector<u8> {
        let len = vector::length(&bytes);
        if (starts_with(&bytes, &MOVE_PREFIX) && *vector::borrow(&bytes, len - 1) == MOVE_SUFFIX) {
            vector::slice(&bytes, 2, len - 1)
        } else {
            bytes
        }
    }

    #[test]
    fun test_try_remove_move_string_prefix() {
        let s2 = try_remove_move_string_prefix(b"b'hello'");
        assert!(s2 == b"hello", 1000);
    }

    #[test]
    fun test_try_remove_move_string_prefix_fail() {
        let s2 = try_remove_move_string_prefix(b"hello");
        assert!(s2 == b"hello", 1000);
    }
}