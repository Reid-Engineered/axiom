pub(crate) struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub(crate) fn sample_integer(&mut self, min: i64, max: i64) -> Option<i64> {
        if min > max {
            return None;
        }

        let range = (i128::from(max) - i128::from(min) + 1) as u128;
        let offset = u128::from(self.next_u64()) % range;
        Some((i128::from(min) + offset as i128) as i64)
    }

    pub(crate) fn sample_float(&mut self, min: f64, max: f64) -> Option<f64> {
        if min > max {
            return None;
        }

        Some(min + (self.next_u64() as f64 / u64::MAX as f64) * (max - min))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_the_same_sequence() {
        let mut a = DeterministicRng::new(42);
        let mut b = DeterministicRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = DeterministicRng::new(1);
        let mut b = DeterministicRng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn sample_integer_stays_within_bounds_and_reaches_both_endpoints() {
        let mut rng = DeterministicRng::new(7);
        let mut saw_min = false;
        let mut saw_max = false;
        for _ in 0..10_000 {
            let value = rng.sample_integer(3, 5).unwrap();
            assert!((3..=5).contains(&value));
            saw_min |= value == 3;
            saw_max |= value == 5;
        }
        assert!(saw_min, "never sampled the minimum in 10,000 draws");
        assert!(saw_max, "never sampled the maximum in 10,000 draws");
    }

    #[test]
    fn sample_float_stays_within_bounds() {
        let mut rng = DeterministicRng::new(99);
        for _ in 0..10_000 {
            let value = rng.sample_float(-2.5, 4.5).unwrap();
            assert!((-2.5..=4.5).contains(&value));
        }
    }

    #[test]
    fn reversed_ranges_are_rejected_without_consuming_rng_state() {
        let mut baseline = DeterministicRng::new(11);
        let expected_next = baseline.next_u64();

        let mut integer_rng = DeterministicRng::new(11);
        assert_eq!(integer_rng.sample_integer(2, 1), None);
        assert_eq!(integer_rng.next_u64(), expected_next);

        let mut float_rng = DeterministicRng::new(11);
        assert_eq!(float_rng.sample_float(2.0, 1.0), None);
        assert_eq!(float_rng.next_u64(), expected_next);
    }

    #[test]
    fn full_width_integer_range_does_not_overflow() {
        let mut rng = DeterministicRng::new(13);
        for _ in 0..100 {
            assert!(rng.sample_integer(i64::MIN, i64::MAX).is_some());
        }
    }
}
