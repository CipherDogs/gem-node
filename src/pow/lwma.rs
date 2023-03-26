use crate::{block::Header, constants::*, primitive::*};
use anyhow::{anyhow, Result};

const T: u64 = 15000;
const N: u64 = LWMA_NUMBER_BLOCKS;

pub struct Lwma1 {
    k: u64,
    pow_limit: U256,
    target: U256,
}

impl Default for Lwma1 {
    fn default() -> Self {
        Self {
            k: N * (N + 1) * T / 2,
            pow_limit: Default::default(),
            target: Default::default(),
        }
    }
}

impl Lwma1 {
    pub fn set_pow_limit(&mut self, pow_limit: U256) {
        self.pow_limit = pow_limit;
    }

    pub fn get_target(&self) -> U256 {
        self.target
    }

    pub fn get_target_u32(&self) -> u32 {
        Self::u256_to_u32(self.target)
    }

    pub fn calculate(&mut self, headers: Vec<Header>) -> Result<()> {
        if headers.len() as u64 <= N {
            self.target = self.pow_limit;
            return Ok(());
        }

        let mut avg_target: U256 = Default::default();

        let mut sum_weighted_solvetimes: u128 = 0;
        let mut j: u128 = 0;

        let mut previous_timestamp = headers[0].timestamp;
        let mut this_timestamp;

        for header in headers.iter().skip(1) {
            this_timestamp = if header.timestamp > previous_timestamp {
                header.timestamp
            } else {
                previous_timestamp + 1
            };

            let solve_time = std::cmp::min((6 * T) as u128, this_timestamp - previous_timestamp);
            previous_timestamp = this_timestamp;

            j += 1;
            sum_weighted_solvetimes += solve_time * j;

            let target = Self::u32_to_u256(header.n_bits)?;
            avg_target += target / N / self.k;
        }

        self.target = avg_target * U256::from(sum_weighted_solvetimes);

        if self.target > self.pow_limit {
            self.target = self.pow_limit;
        }

        Ok(())
    }

    fn u32_to_u256(value: u32) -> Result<U256> {
        let size = value >> 24;
        let mut word = value & 0x007fffff;

        let result = if size <= 3 {
            word >>= 8 * (3 - size as usize);
            word.into()
        } else {
            U256::from(word) << (8 * (size as usize - 3))
        };

        let is_negative = word != 0 && (value & 0x00800000) != 0;
        let is_overflow =
            (word != 0 && size > 34) || (word > 0xff && size > 33) || (word > 0xffff && size > 32);

        if is_negative || is_overflow {
            Err(anyhow!(
                "Result is either a negative number or an overflow number"
            ))
        } else {
            Ok(result)
        }
    }

    fn u256_to_u32(value: U256) -> u32 {
        let mut size = (value.bits() + 7) / 8;
        let mut compact = if size <= 3 {
            (value.low_u64() << (8 * (3 - size))) as u32
        } else {
            let bn = value >> (8 * (size - 3));
            bn.low_u32()
        };

        if (compact & 0x00800000) != 0 {
            compact >>= 8;
            size += 1;
        }

        assert!((compact & !0x007fffff) == 0);
        assert!(size < 256);

        compact | (size << 24) as u32
    }
}
