// success.rs

use core::hash::Hash;
use core::ops::Deref;

/// Constant representing a successful operation.
pub const OK: Success = Success { ok: true };

/// Constant representing a failed operation.
pub const FAIL: Success = Success { ok: false };

/// Structure representing the success state of an operation.
#[must_use]
#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash,
)]
pub struct Success {
    /// Boolean indicating whether the operation was successful.
    pub ok: bool,
}

/// Structure representing the failure state of an operation.
#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash,
)]
pub struct Failure {
    /// Boolean indicating whether the operation failed.
    pub fail: bool,
}

impl Deref for Success {
    type Target = Failure;

    /// Returns a reference to the corresponding `Failure` instance based on the success state.
    fn deref(&self) -> &Self::Target {
        if self.ok {
            &Failure { fail: false }
        } else {
            &Failure { fail: true }
        }
    }
}

/// Checks if the given `Success` result indicates a successful operation.
///
/// # Arguments
///
/// * `result` - A `Success` struct representing the result of an operation.
///
/// # Returns
///
/// * `true` if the operation was successful.
/// * `false` if the operation failed.
#[must_use]
pub const fn is_success(result: Success) -> bool {
    result.ok
}

/// Checks if the given `Success` result indicates a failure operation.
///
/// # Arguments
///
/// * `result` - A `Success` struct representing the result of an operation.
///
/// # Returns
///
/// * `true` if the operation failed.
/// * `false` if the operation was successful.
#[must_use]
pub const fn is_failure(result: Success) -> bool {
    !result.ok
}

#[cfg(test)]
mod tests {
    use crate::success::{
        is_failure, is_success, Failure, Success, FAIL, OK,
    };
    use alloc::format;
    use core::{
        hash::{Hash, Hasher},
        ops::Deref,
    };

    #[derive(Default)]
    struct TestHasher {
        state: u64,
    }

    impl Hasher for TestHasher {
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes {
                self.state = self.state.wrapping_add(*byte as u64);
            }
        }

        fn finish(&self) -> u64 {
            self.state
        }
    }

    #[test]
    fn test_is_success() {
        assert!(is_success(OK));
        assert!(!is_success(FAIL));
    }

    #[test]
    fn test_is_failure() {
        assert!(is_failure(FAIL));
        assert!(!is_failure(OK));
    }

    #[test]
    fn test_deref() {
        let success = OK;
        let failure = FAIL;
        assert!(!success.deref().fail);
        assert!(failure.deref().fail);
    }

    #[test]
    fn test_success_equality() {
        let success1 = Success { ok: true };
        let success2 = Success { ok: true };
        assert_eq!(success1, success2);
    }

    #[test]
    fn test_failure_equality() {
        let failure1 = Success { ok: false };
        let failure2 = Success { ok: false };
        assert_eq!(failure1, failure2);
    }

    #[test]
    fn test_success_inequality() {
        let success = Success { ok: true };
        let failure = Success { ok: false };
        assert_ne!(success, failure);
    }

    #[test]
    fn test_default_success() {
        let default_success: Success = Default::default();
        assert_eq!(default_success, Success { ok: false });
    }

    #[test]
    fn test_default_failure() {
        let default_failure: Failure = Default::default();
        assert_eq!(default_failure, Failure { fail: false });
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn test_clone_success() {
        let success = OK;
        let cloned = success.clone();
        assert_eq!(success, cloned);
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn test_clone_failure() {
        let failure = FAIL;
        let cloned = failure.clone();
        assert_eq!(failure, cloned);
    }

    #[test]
    fn test_debug_success() {
        let debug_output = format!("{:?}", OK);
        assert_eq!(debug_output, "Success { ok: true }");
    }

    #[test]
    fn test_debug_failure() {
        let debug_output = format!("{:?}", FAIL);
        assert_eq!(debug_output, "Success { ok: false }");
    }

    #[test]
    fn test_hash_success() {
        let mut hasher = TestHasher::default();
        OK.hash(&mut hasher);
        let success_hash = hasher.finish();

        let mut hasher2 = TestHasher::default();
        Success { ok: true }.hash(&mut hasher2);
        assert_eq!(success_hash, hasher2.finish());
    }

    #[test]
    fn test_hash_failure() {
        let mut hasher = TestHasher::default();
        FAIL.hash(&mut hasher);
        let failure_hash = hasher.finish();

        let mut hasher2 = TestHasher::default();
        Success { ok: false }.hash(&mut hasher2);
        assert_eq!(failure_hash, hasher2.finish());
    }
}
