//! Build probe only. No simulation semantics are defined in T00.

/// Adds unsigned 32-bit operands modulo 2^32 for the infrastructure smoke test.
pub fn build_probe(left: u32, right: u32) -> u32 {
    left.wrapping_add(right)
}

#[cfg(test)]
mod tests {
    use super::build_probe;

    #[test]
    fn independently_worked_probe_cases() {
        for (left, right, expected) in [
            (0, 0, 0),
            (17, 25, 42),
            (123, 456, 579),
            (u32::MAX, 1, 0),
            (u32::MAX, u32::MAX, 4_294_967_294),
        ] {
            assert_eq!(build_probe(left, right), expected);
        }
    }
}
