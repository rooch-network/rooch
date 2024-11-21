module rooch_fish::utils {
    use rooch_fish::simple_rng;

    /// Generate a random position within the given bounds
    public fun random_position(nonce: u64, max_x: u64, max_y: u64): (u64, u64) {
        (random_u64(nonce, max_x), random_u64(nonce + 1, max_y))
    }

    /// Generate a random u64 number between 0 and max (inclusive)
    public fun random_u64(nonce: u64, max: u64): u64 {
        simple_rng::rand_u64_range(nonce, 0, max + 1)
    }

    /// Calculate Manhattan distance between two points
    public fun calculate_distance(x1: u64, y1: u64, x2: u64, y2: u64): u64 {
        let dx = if (x1 > x2) { x1 - x2 } else { x2 - x1 };
        let dy = if (y1 > y2) { y1 - y2 } else { y2 - y1 };
        dx + dy
    }

    /// Clamp a value between min and max
    public fun clamp(value: u64, min: u64, max: u64): u64 {
        if (value < min) min
        else if (value > max) max
        else value
    }

    /// Check if a point is within a rectangle
    public fun is_point_in_rect(x: u64, y: u64, rect_x: u64, rect_y: u64, rect_width: u64, rect_height: u64): bool {
        x >= rect_x && x < rect_x + rect_width && y >= rect_y && y < rect_y + rect_height
    }

    /// Performs saturating subtraction
    public fun saturating_sub(x: u64, y: u64): u64 {
        if (x < y) {
            0
        } else {
            x - y
        }
    }

    #[test]
    fun test_random_position() {
        let (x, y) = random_position(0, 100, 100);
        assert!(x <= 100 && y <= 100, 0);
    }

    #[test]
    fun test_random_u64() {
        let r = random_u64(0, 10);
        assert!(r <= 10, 0);
    }

    #[test]
    fun test_calculate_distance() {
        assert!(calculate_distance(0, 0, 3, 4) == 7, 0);
    }

    #[test]
    fun test_clamp() {
        assert!(clamp(5, 0, 10) == 5, 0);
        assert!(clamp(15, 0, 10) == 10, 0);
        assert!(clamp(0, 5, 10) == 5, 0);
    }

    #[test]
    fun test_is_point_in_rect() {
        assert!(is_point_in_rect(5, 5, 0, 0, 10, 10), 0);
        assert!(!is_point_in_rect(15, 15, 0, 0, 10, 10), 0);
    }

    #[test]
    fun test_saturating_sub() {
        assert!(saturating_sub(10, 5) == 5, 0);
        assert!(saturating_sub(5, 10) == 0, 1);
        assert!(saturating_sub(0, 1) == 0, 2);
        assert!(saturating_sub(18446744073709551615, 1) == 18446744073709551614, 3); // u64::MAX - 1
    }
}
