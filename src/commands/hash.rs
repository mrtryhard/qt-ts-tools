pub struct SysVHasher {
    computed: u32,
}

impl SysVHasher {
    pub fn new() -> SysVHasher {
        SysVHasher { computed: 0 }
    }

    /*
        This function is based on the [elf crate](https://docs.rs/elf/).
        As of 2025-12-28, it is compatible license-wise (MIT or Apache-2.0).
    */
    pub fn hash(&mut self, value: &[u8]) -> &SysVHasher {
        for byte in value {
            self.computed = self.computed.wrapping_mul(16).wrapping_add(*byte as u32);
            self.computed ^= (self.computed >> 24) & 0xf0;
        }
        self.computed &= 0xfffffff;

        self
    }

    pub fn compute(&self) -> u32 {
        if self.computed != 0 { self.computed } else { 1 }
    }
}

#[cfg(test)]
mod hash_tests {
    use crate::commands::hash::SysVHasher;

    #[test]
    fn basic_test() {
        // This matches Qt's encoding
        let expected_raw: [u8; 4] = [0x07, 0xa6, 0xc8, 0x95];
        let expected = u32::from_be_bytes(expected_raw);
        let actual = SysVHasher::new().hash("source".as_bytes()).compute();

        assert_eq!(actual, expected);
    }
}
