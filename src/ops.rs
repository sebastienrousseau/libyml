// ops.rs

//! # Checked Arithmetic Operations (no_std version)
//!
//! This module provides checked arithmetic traits that **abort** on overflow (or invalid conversions) in non-test builds, and **panic** in tests. This is suitable for `#![no_std]` environments where we can’t rely on normal unwinding or `std::process::abort`.
//!
//! ## Notes on Performance
//!
//! Each checked operation introduces a small runtime overhead compared to direct, unchecked arithmetic. In tight loops or performance-critical sections, consider whether you need these overflow checks. If performance is paramount and you can guarantee no overflow occurs, using regular arithmetic may be preferable.

#[cfg(all(test, feature = "std"))]
extern crate std;

use core::convert::TryInto;

/// Trait for performing addition that aborts on overflow instead of wrapping.
///
/// # Examples
///
/// ```ignore
/// use your_crate::ops::ForceAdd;
///
/// let x: u8 = 10;
/// let y: u8 = 20;
/// let z = x.force_add(y); // 30
/// assert_eq!(z, 30);
/// ```
pub trait ForceAdd: Sized {
    /// Adds `rhs` to `self`, aborting (or panicking in tests) on overflow.
    ///
    /// This returns a new value with the result of the addition.
    ///
    /// # Performance
    ///
    /// This method performs a checked add, which adds a small runtime overhead.
    #[must_use = "Checked add returns a new value and should not be ignored."]
    fn force_add(self, rhs: Self) -> Self;
}

/// Trait for performing multiplication that aborts on overflow instead of wrapping.
///
/// # Examples
///
/// ```ignore
/// use your_crate::ops::ForceMul;
///
/// let x: i32 = 10;
/// let y: i32 = 20;
/// let z = x.force_mul(y); // 200
/// assert_eq!(z, 200);
/// ```
pub trait ForceMul: Sized {
    /// Multiplies `rhs` with `self`, aborting (or panicking in tests) on overflow.
    ///
    /// This returns a new value with the result of the multiplication.
    ///
    /// # Performance
    ///
    /// This method performs a checked mul, which adds a small runtime overhead.
    #[must_use = "Checked mul returns a new value and should not be ignored."]
    fn force_mul(self, rhs: Self) -> Self;
}

/// Trait for performing checked type conversions that abort on failure.
///
/// # Examples
///
/// ```ignore
/// use your_crate::ops::ForceInto;
///
/// let small: u8 = 100;
/// let bigger: u16 = small.force_into(); // 100
/// ```
pub trait ForceInto {
    /// Converts `self` to type `U`, aborting (or panicking in tests) on conversion failure.
    ///
    /// This returns a new value of type `U`.
    ///
    /// # Performance
    ///
    /// This method performs a checked conversion, which adds a small runtime overhead.
    #[must_use = "Forcing a conversion returns a new value and should not be ignored."]
    fn force_into<U>(self) -> U
    where
        Self: TryInto<U>;
}

/// Manually aborts (via UD2 or infinite loop) or panics depending on the build.
///
/// - **Non-test builds**: use inline assembly or an infinite loop to abort.
/// - **Test builds**: `panic!()` so that `#[should_panic]` tests can detect it.
///
/// # Safety
///
/// The inline assembly block uses `ud2` on x86/x86_64, which will produce an illegal
/// instruction and terminate the process immediately on most platforms. On other
/// architectures, this falls back to an infinite loop.
#[inline(never)]
#[cold]
pub(crate) fn die<T>() -> T {
    #[cfg(not(test))]
    {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("ud2", options(noreturn));
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        loop {
            // Fallback to an infinite loop on other architectures
            core::hint::spin_loop();
        }
    }

    #[cfg(test)]
    panic!("arithmetic overflow or invalid conversion");
}

/// Helper to avoid `.unwrap()` for `Option`.
#[inline]
fn or_die<T>(val: Option<T>) -> T {
    match val {
        Some(v) => v,
        None => die(),
    }
}

/// Helper to avoid `.unwrap()` for `Result`.
#[inline]
fn or_die_result<T, E>(val: Result<T, E>) -> T {
    match val {
        Ok(v) => v,
        Err(_) => die(),
    }
}

// -----------------------------------------------------------------------------
// ForceAdd Implementations
// -----------------------------------------------------------------------------

impl ForceAdd for u8 {
    #[inline]
    fn force_add(self, rhs: Self) -> Self {
        or_die(self.checked_add(rhs))
    }
}

impl ForceAdd for i32 {
    #[inline]
    fn force_add(self, rhs: Self) -> Self {
        or_die(self.checked_add(rhs))
    }
}

impl ForceAdd for i64 {
    #[inline]
    fn force_add(self, rhs: Self) -> Self {
        or_die(self.checked_add(rhs))
    }
}

impl ForceAdd for u32 {
    #[inline]
    fn force_add(self, rhs: Self) -> Self {
        or_die(self.checked_add(rhs))
    }
}

impl ForceAdd for u64 {
    #[inline]
    fn force_add(self, rhs: Self) -> Self {
        or_die(self.checked_add(rhs))
    }
}

impl ForceAdd for usize {
    #[inline]
    fn force_add(self, rhs: Self) -> Self {
        or_die(self.checked_add(rhs))
    }
}

// -----------------------------------------------------------------------------
// ForceMul Implementations
// -----------------------------------------------------------------------------

impl ForceMul for i32 {
    #[inline]
    fn force_mul(self, rhs: Self) -> Self {
        or_die(self.checked_mul(rhs))
    }
}

impl ForceMul for i64 {
    #[inline]
    fn force_mul(self, rhs: Self) -> Self {
        or_die(self.checked_mul(rhs))
    }
}

impl ForceMul for u8 {
    #[inline]
    fn force_mul(self, rhs: Self) -> Self {
        or_die(self.checked_mul(rhs))
    }
}

impl ForceMul for u32 {
    #[inline]
    fn force_mul(self, rhs: Self) -> Self {
        or_die(self.checked_mul(rhs))
    }
}

impl ForceMul for u64 {
    #[inline]
    fn force_mul(self, rhs: Self) -> Self {
        or_die(self.checked_mul(rhs))
    }
}

impl ForceMul for usize {
    #[inline]
    fn force_mul(self, rhs: Self) -> Self {
        or_die(self.checked_mul(rhs))
    }
}

// -----------------------------------------------------------------------------
// ForceInto Implementation
// -----------------------------------------------------------------------------

impl<T> ForceInto for T {
    #[inline]
    fn force_into<U>(self) -> U
    where
        Self: TryInto<U>,
    {
        or_die_result(self.try_into())
    }
}

// -----------------------------------------------------------------------------
// Unit Tests
// -----------------------------------------------------------------------------

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // FORCE ADD TESTS
    // -------------------------------------------------------------------------
    mod force_add_tests {
        use super::*;

        // -------------------- u8 Tests --------------------
        mod u8_tests {
            use super::*;

            #[test]
            fn test_normal_addition() {
                assert_eq!(5u8.force_add(3), 8);
            }

            #[test]
            fn test_zero_addition() {
                assert_eq!(0u8.force_add(0), 0);
                assert_eq!(5u8.force_add(0), 5);
                assert_eq!(0u8.force_add(5), 5);
            }

            #[test]
            fn test_maximum_valid() {
                assert_eq!(254u8.force_add(1), 255);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = 255u8.force_add(1);
            }

            #[test]
            #[should_panic]
            fn test_large_addition_overflow() {
                let _ = 200u8.force_add(100);
            }
        }

        // -------------------- i32 Tests --------------------
        mod i32_tests {
            use super::*;

            #[test]
            fn test_normal_addition() {
                assert_eq!(5i32.force_add(3), 8);
                assert_eq!((-5).force_add(3), -2);
                assert_eq!(5.force_add(-3), 2);
                assert_eq!((-5).force_add(-3), -8);
            }

            #[test]
            fn test_zero_addition() {
                assert_eq!(0i32.force_add(0), 0);
                assert_eq!(5i32.force_add(0), 5);
                assert_eq!((-5).force_add(0), -5);
            }

            #[test]
            fn test_maximum_valid() {
                assert_eq!((i32::MAX - 1).force_add(1), i32::MAX);
            }

            #[test]
            fn test_minimum_valid() {
                assert_eq!((i32::MIN + 1).force_add(-1), i32::MIN);
            }

            #[test]
            #[should_panic]
            fn test_positive_overflow() {
                let _ = i32::MAX.force_add(1);
            }

            #[test]
            #[should_panic]
            fn test_negative_overflow() {
                let _ = i32::MIN.force_add(-1);
            }

            // Additional boundary checks
            #[test]
            fn test_min_plus_zero() {
                assert_eq!(i32::MIN.force_add(0), i32::MIN);
            }

            #[test]
            fn test_max_plus_negative() {
                // i32::MAX + (-1) = i32::MAX - 1
                assert_eq!(i32::MAX.force_add(-1), i32::MAX - 1);
            }
        }

        // -------------------- i64 Tests --------------------
        mod i64_tests {
            use super::*;

            #[test]
            fn test_normal_addition() {
                assert_eq!(5i64.force_add(3), 8);
                assert_eq!((-5).force_add(3), -2);
                assert_eq!(5.force_add(-3), 2);
                assert_eq!((-5).force_add(-3), -8);
            }

            #[test]
            fn test_zero_addition() {
                assert_eq!(0i64.force_add(0), 0);
                assert_eq!(5i64.force_add(0), 5);
                assert_eq!((-5).force_add(0), -5);
            }

            #[test]
            fn test_maximum_valid() {
                assert_eq!((i64::MAX - 1).force_add(1), i64::MAX);
            }

            #[test]
            fn test_minimum_valid() {
                assert_eq!((i64::MIN + 1).force_add(-1), i64::MIN);
            }

            #[test]
            #[should_panic]
            fn test_positive_overflow() {
                let _ = i64::MAX.force_add(1);
            }

            #[test]
            #[should_panic]
            fn test_negative_overflow() {
                let _ = i64::MIN.force_add(-1);
            }

            // Additional boundary checks
            #[test]
            fn test_min_plus_zero() {
                assert_eq!(i64::MIN.force_add(0), i64::MIN);
            }

            #[test]
            fn test_max_plus_negative() {
                assert_eq!(i64::MAX.force_add(-1), i64::MAX - 1);
            }
        }

        // -------------------- u32 Tests --------------------
        mod u32_tests {
            use super::*;

            #[test]
            fn test_normal_addition() {
                assert_eq!(5u32.force_add(3), 8);
            }

            #[test]
            fn test_zero_addition() {
                assert_eq!(0u32.force_add(0), 0);
                assert_eq!(5u32.force_add(0), 5);
                assert_eq!(0u32.force_add(5), 5);
            }

            #[test]
            fn test_maximum_valid() {
                assert_eq!((u32::MAX - 1).force_add(1), u32::MAX);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = u32::MAX.force_add(1);
            }
        }

        // -------------------- u64 Tests --------------------
        mod u64_tests {
            use super::*;

            #[test]
            fn test_normal_addition() {
                assert_eq!(5u64.force_add(3), 8);
            }

            #[test]
            fn test_zero_addition() {
                assert_eq!(0u64.force_add(0), 0);
                assert_eq!(5u64.force_add(0), 5);
                assert_eq!(0u64.force_add(5), 5);
            }

            #[test]
            fn test_maximum_valid() {
                assert_eq!((u64::MAX - 1).force_add(1), u64::MAX);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = u64::MAX.force_add(1);
            }
        }

        // -------------------- usize Tests --------------------
        mod usize_tests {
            use super::*;

            #[test]
            fn test_normal_addition() {
                assert_eq!(5usize.force_add(3), 8);
            }

            #[test]
            fn test_zero_addition() {
                assert_eq!(0usize.force_add(0), 0);
                assert_eq!(5usize.force_add(0), 5);
                assert_eq!(0usize.force_add(5), 5);
            }

            #[test]
            fn test_maximum_valid() {
                assert_eq!((usize::MAX - 1).force_add(1), usize::MAX);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = usize::MAX.force_add(1);
            }
        }
    }

    // -------------------------------------------------------------------------
    // FORCE MUL TESTS
    // -------------------------------------------------------------------------
    mod force_mul_tests {
        use super::*;

        // -------------------- i32 Tests --------------------
        mod i32_tests {
            use super::*;

            #[test]
            fn test_normal_multiplication() {
                assert_eq!(5i32.force_mul(3), 15);
                assert_eq!((-5).force_mul(3), -15);
                assert_eq!(5.force_mul(-3), -15);
                assert_eq!((-5).force_mul(-3), 15);
            }

            #[test]
            fn test_zero_multiplication() {
                assert_eq!(0i32.force_mul(0), 0);
                assert_eq!(5i32.force_mul(0), 0);
                assert_eq!(0i32.force_mul(5), 0);
                assert_eq!((-5).force_mul(0), 0);
                assert_eq!(0i32.force_mul(-5), 0);
            }

            #[test]
            fn test_one_multiplication() {
                assert_eq!(5i32.force_mul(1), 5);
                assert_eq!(1i32.force_mul(5), 5);
                assert_eq!((-5).force_mul(1), -5);
                assert_eq!(1i32.force_mul(-5), -5);
            }

            #[test]
            #[should_panic]
            fn test_positive_overflow() {
                let _ = i32::MAX.force_mul(2);
            }

            #[test]
            #[should_panic]
            fn test_negative_overflow() {
                let _ = i32::MIN.force_mul(2);
            }

            // Additional boundary checks
            #[test]
            fn test_min_times_one() {
                // i32::MIN * 1 = i32::MIN
                assert_eq!(i32::MIN.force_mul(1), i32::MIN);
            }

            #[test]
            fn test_max_times_minus_one() {
                // i32::MAX * -1 = -i32::MAX (valid)
                assert_eq!(i32::MAX.force_mul(-1), -i32::MAX);
            }
        }

        // -------------------- i64 Tests --------------------
        mod i64_tests {
            use super::*;

            #[test]
            fn test_normal_multiplication() {
                assert_eq!(5i64.force_mul(3), 15);
                assert_eq!((-5).force_mul(3), -15);
                assert_eq!(5.force_mul(-3), -15);
                assert_eq!((-5).force_mul(-3), 15);
            }

            #[test]
            fn test_zero_multiplication() {
                assert_eq!(0i64.force_mul(0), 0);
                assert_eq!(5i64.force_mul(0), 0);
                assert_eq!(0i64.force_mul(5), 0);
            }

            #[test]
            fn test_one_multiplication() {
                assert_eq!(5i64.force_mul(1), 5);
                assert_eq!(1i64.force_mul(5), 5);
                assert_eq!((-5).force_mul(1), -5);
                assert_eq!(1i64.force_mul(-5), -5);
            }

            #[test]
            #[should_panic]
            fn test_positive_overflow() {
                let _ = i64::MAX.force_mul(2);
            }

            #[test]
            #[should_panic]
            fn test_negative_overflow() {
                let _ = i64::MIN.force_mul(2);
            }

            // Additional boundary checks
            #[test]
            fn test_min_times_one() {
                // i64::MIN * 1 = i64::MIN
                assert_eq!(i64::MIN.force_mul(1), i64::MIN);
            }

            #[test]
            fn test_max_times_minus_one() {
                // i64::MAX * -1 = -i64::MAX (valid)
                assert_eq!(i64::MAX.force_mul(-1), -i64::MAX);
            }

            #[test]
            #[should_panic]
            fn test_min_times_minus_one_overflow() {
                // i64::MIN * -1 is out of range
                let _ = i64::MIN.force_mul(-1);
            }
        }

        // -------------------- u8 Tests --------------------
        mod u8_tests {
            use super::*;

            #[test]
            fn test_normal_multiplication() {
                assert_eq!(5u8.force_mul(3), 15);
            }

            #[test]
            fn test_zero_multiplication() {
                assert_eq!(0u8.force_mul(0), 0);
                assert_eq!(5u8.force_mul(0), 0);
                assert_eq!(0u8.force_mul(5), 0);
            }

            #[test]
            fn test_one_multiplication() {
                assert_eq!(5u8.force_mul(1), 5);
                assert_eq!(1u8.force_mul(5), 5);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = u8::MAX.force_mul(2);
            }

            // Additional boundary checks
            #[test]
            fn test_u8_min_times_max() {
                // 0 * 255 = 0
                assert_eq!(0u8.force_mul(255), 0);
            }

            #[test]
            fn test_u8_near_overflow() {
                // 128 * 2 = 256 (overflow for u8)
                let _ = 128u8.force_mul(2);
            }
        }

        // -------------------- u32 Tests --------------------
        mod u32_tests {
            use super::*;

            #[test]
            fn test_normal_multiplication() {
                assert_eq!(5u32.force_mul(3), 15);
            }

            #[test]
            fn test_zero_multiplication() {
                assert_eq!(0u32.force_mul(0), 0);
                assert_eq!(5u32.force_mul(0), 0);
            }

            #[test]
            fn test_one_multiplication() {
                assert_eq!(5u32.force_mul(1), 5);
                assert_eq!(1u32.force_mul(5), 5);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = u32::MAX.force_mul(2);
            }

            // Additional boundary checks
            #[test]
            fn test_u32_min_times_max() {
                // 0 * u32::MAX = 0
                assert_eq!(0u32.force_mul(u32::MAX), 0);
            }

            #[test]
            fn test_u32_near_overflow() {
                // (u32::MAX / 2) * 2 = Possibly safe, if it's exactly even
                let half_max = u32::MAX / 2;
                assert_eq!(half_max.force_mul(2), u32::MAX - 1); // for even MAX, (2^32 - 1) is odd
            }
        }

        // -------------------- u64 Tests --------------------
        mod u64_tests {
            use super::*;

            #[test]
            fn test_normal_multiplication() {
                assert_eq!(5u64.force_mul(3), 15);
            }

            #[test]
            fn test_zero_multiplication() {
                assert_eq!(0u64.force_mul(0), 0);
                assert_eq!(5u64.force_mul(0), 0);
                assert_eq!(0u64.force_mul(5), 0);
            }

            #[test]
            fn test_one_multiplication() {
                assert_eq!(5u64.force_mul(1), 5);
                assert_eq!(1u64.force_mul(5), 5);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = u64::MAX.force_mul(2);
            }

            // Additional boundary checks
            #[test]
            fn test_u64_min_times_max() {
                assert_eq!(0u64.force_mul(u64::MAX), 0);
            }

            #[test]
            fn test_u64_near_overflow() {
                // (u64::MAX / 2) * 2 = u64::MAX - 1
                let half_max = u64::MAX / 2;
                assert_eq!(half_max.force_mul(2), u64::MAX - 1);
            }
        }

        // -------------------- usize Tests --------------------
        mod usize_tests {
            use super::*;

            #[test]
            fn test_normal_multiplication() {
                assert_eq!(5usize.force_mul(3), 15);
            }

            #[test]
            fn test_zero_multiplication() {
                assert_eq!(0usize.force_mul(0), 0);
                assert_eq!(5usize.force_mul(0), 0);
            }

            #[test]
            fn test_one_multiplication() {
                assert_eq!(5usize.force_mul(1), 5);
                assert_eq!(1usize.force_mul(5), 5);
            }

            #[test]
            #[should_panic]
            fn test_overflow() {
                let _ = usize::MAX.force_mul(2);
            }

            // Additional boundary checks
            #[test]
            fn test_usize_min_times_max() {
                assert_eq!(0usize.force_mul(usize::MAX), 0);
            }

            #[test]
            fn test_usize_near_overflow() {
                // (usize::MAX / 2) * 2 == usize::MAX - (1 or 0) depending on alignment
                let half_max = usize::MAX / 2;
                let result = half_max.force_mul(2);
                // For 64-bit systems: half_max = 2^63-1 => half_max*2 = 2^64-2 = usize::MAX - 1
                // For 32-bit systems: half_max = 2^31-1 => half_max*2 = 2^32-2 = usize::MAX - 1
                assert!(
                    result == usize::MAX || result == usize::MAX - 1
                );
            }
        }
    }

    // -------------------------------------------------------------------------
    // FORCE INTO TESTS
    // -------------------------------------------------------------------------
    mod force_into_tests {
        use super::*;

        #[test]
        fn test_valid_conversions() {
            let x: u8 = 5;
            let _: u16 = x.force_into();
            let _: u32 = x.force_into();
            let _: u64 = x.force_into();
            let _: usize = x.force_into();

            let x: i32 = -5;
            let y: i64 = x.force_into();
            assert_eq!(y, -5_i64);
        }

        #[test]
        fn test_maximum_value_conversions() {
            let x: u8 = u8::MAX;
            let y: u16 = x.force_into();
            assert_eq!(y, u16::from(u8::MAX));
        }

        #[test]
        #[should_panic]
        fn test_overflow_u16_to_u8() {
            // 256 u16 cannot fit in u8
            let x: u16 = 256;
            let _: u8 = x.force_into();
        }

        #[test]
        #[should_panic]
        fn test_negative_i32_to_u32() {
            let x: i32 = -1;
            let _: u32 = x.force_into();
        }

        #[test]
        #[should_panic]
        fn test_overflow_i32_to_i16() {
            // 32768 i32 cannot fit in i16
            let x: i32 = 32768;
            let _: i16 = x.force_into();
        }

        // Additional coverage for i64 -> i32
        #[test]
        fn test_i64_within_i32_bounds() {
            // i64::MAX is out of range, but let's pick a value safely in i32 range
            let x: i64 = 123_456_789;
            let y: i32 = x.force_into();
            assert_eq!(y, 123_456_789);
        }

        #[test]
        #[should_panic]
        fn test_i64_above_i32_max() {
            // i64::MAX is definitely too large
            let x: i64 = i64::from(i32::MAX) + 1;
            let _: i32 = x.force_into();
        }

        #[test]
        #[should_panic]
        fn test_i64_below_i32_min() {
            let x: i64 = i64::from(i32::MIN) - 1;
            let _: i32 = x.force_into();
        }

        // Additional coverage for i64 -> u64 negative
        #[test]
        #[should_panic]
        fn test_negative_i64_to_u64() {
            let x: i64 = -42;
            let _: u64 = x.force_into();
        }
    }

    #[cfg(all(test, feature = "std"))]
    mod helper_fn_tests {
        use super::*;

        #[test]
        #[should_panic]
        fn test_or_die_none() {
            let some_val: Option<u8> = None;
            let _ = or_die(some_val);
        }

        #[test]
        #[should_panic]
        fn test_or_die_result_err() {
            let some_res: Result<u8, ()> = Err(());
            let _ = or_die_result(some_res);
        }

        #[test]
        fn test_or_die_some() {
            let some_val: Option<u8> = Some(42);
            assert_eq!(or_die(some_val), 42);
        }

        #[test]
        fn test_or_die_result_ok() {
            let some_res: Result<u8, ()> = Ok(99);
            assert_eq!(or_die_result(some_res), 99);
        }

        #[test]
        #[should_panic]
        fn test_overflow_i8_to_u8() {
            // -1 i8 => 255 in u8, which is valid in two's complement representation,
            // but `TryFrom` fails in Rust because negative can't fit in u8.
            let x: i8 = -1;
            let _: u8 = x.force_into();
        }

        #[test]
        fn test_valid_i8_to_u8() {
            let x: i8 = 100;
            let y: u8 = x.force_into();
            assert_eq!(y, 100);
        }

        #[test]
        #[should_panic]
        fn test_u64_to_i32_out_of_range() {
            // i32::MAX is about 2.147e9; let's exceed that
            let x: u64 = (i32::MAX as u64) + 1;
            let _: i32 = x.force_into();
        }

        #[test]
        #[should_panic]
        fn test_sequential_operations() {
            let x: i32 = 1;
            let y: i32 = 2;
            let z: i32 = 3;

            // (1 + 2) * 3 = 9
            let sum = x.force_add(y);
            let product = sum.force_mul(z);
            assert_eq!(product, 9);

            // Overflow check: (i32::MAX - 1 + 1) * 2 => i32::MAX * 2 => should fail
            {
                let max_minus_one = i32::MAX - 1;
                let sum = max_minus_one.force_add(1);
                let _over = sum.force_mul(2);
            }
        }
    }
}
