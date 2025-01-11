#![allow(missing_docs)]
#![no_std]

use libyml::loader::allocate_yaml_scalar;
use libyml::loader::free_yaml_memory;
use libyml::{libc, loader::YamlScalarValue};

/// Tests for YamlScalarValue initialization and basic operations
mod scalar_init_tests {
    use super::*;

    /// Test creation of new empty scalar value
    #[test]
    fn test_new() {
        let scalar = YamlScalarValue::new();
        assert_eq!(scalar.len(), 0);
        assert!(scalar.is_empty());
        assert!(scalar.is_inline());
        assert_eq!(scalar.as_slice(), &[]);
    }

    /// Test creation from small slice (inline storage)
    #[test]
    fn test_from_slice_inline() {
        let data = b"test data";
        let scalar = YamlScalarValue::from_slice(data);
        assert_eq!(scalar.len(), data.len());
        assert!(!scalar.is_empty());
        assert!(scalar.is_inline());
        assert_eq!(scalar.as_slice(), data);
    }

    /// Test creation from large slice (heap storage)
    #[test]
    fn test_from_slice_heap() {
        let data = [0u8; 30]; // Larger than INLINE_CAPACITY
        let scalar = YamlScalarValue::from_slice(&data);
        assert_eq!(scalar.len(), data.len());
        assert!(!scalar.is_empty());
        assert!(!scalar.is_inline());
        assert_eq!(scalar.as_slice(), &data);
    }
}

/// Tests for YamlScalarValue comparison and equality
mod scalar_comparison_tests {
    use super::*;

    /// Test cloning and equality
    #[test]
    fn test_clone_and_eq() {
        let data = b"test data for cloning";
        let scalar = YamlScalarValue::from_slice(data);
        let cloned = scalar.clone();
        assert_eq!(scalar, cloned);
        assert_eq!(scalar.as_slice(), cloned.as_slice());
    }

    /// Test ordering between scalar values
    #[test]
    fn test_ordering() {
        let scalar1 = YamlScalarValue::from_slice(b"aaa");
        let scalar2 = YamlScalarValue::from_slice(b"bbb");
        assert!(scalar1 < scalar2);
    }

    /// Test equality between different storage types
    #[test]
    fn test_mixed_equality() {
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different";

        let scalar1 = YamlScalarValue::from_slice(data1);
        let scalar2 = YamlScalarValue::from_slice(data2);
        let scalar3 = YamlScalarValue::from_slice(data3);

        assert_eq!(scalar1, scalar2);
        assert_ne!(scalar1, scalar3);
    }

    /// Test comparison between inline and heap storage
    #[test]
    fn test_mixed_comparison() {
        let inline = YamlScalarValue::from_slice(b"zzz");
        let heap = YamlScalarValue::from_slice(&[b'a'; 30]);

        assert!(heap < inline);
        assert!(inline > heap);
    }
}

/// Tests for boundary conditions and edge cases
mod scalar_boundary_tests {
    use super::*;

    /// Test various edge cases for scalar values
    #[test]
    fn test_edge_cases() {
        // Empty slice
        let scalar = YamlScalarValue::from_slice(&[]);
        assert!(scalar.is_empty());
        assert_eq!(scalar.len(), 0);
        assert!(scalar.is_inline());

        // Exactly at INLINE_CAPACITY
        let data = [1u8; YamlScalarValue::INLINE_CAPACITY];
        let scalar = YamlScalarValue::from_slice(&data);
        assert!(scalar.is_inline());
        assert_eq!(scalar.len(), YamlScalarValue::INLINE_CAPACITY);

        // Just over INLINE_CAPACITY
        let data = [1u8; YamlScalarValue::INLINE_CAPACITY + 1];
        let scalar = YamlScalarValue::from_slice(&data);
        assert!(!scalar.is_inline());
        assert_eq!(scalar.len(), YamlScalarValue::INLINE_CAPACITY + 1);
    }

    /// Test very large data handling
    #[test]
    fn test_large_data() {
        let large_data = [0xFFu8; 1024];
        let scalar = YamlScalarValue::from_slice(&large_data);

        assert!(!scalar.is_inline());
        assert_eq!(scalar.len(), 1024);
        assert_eq!(scalar.as_slice(), &large_data);
    }

    /// Test storage transitions
    #[test]
    fn test_storage_transitions() {
        let mut scalar = YamlScalarValue::from_slice(b"initial");
        assert!(scalar.is_inline());

        scalar = YamlScalarValue::from_slice(&[0u8; 30]);
        assert!(!scalar.is_inline());

        scalar = YamlScalarValue::from_slice(b"back to inline");
        assert!(scalar.is_inline());
    }
}

/// Tests for special data handling
mod scalar_special_data_tests {
    use super::*;

    /// Test binary data handling
    #[test]
    fn test_binary_data() {
        let binary = [0u8, 255u8, 128u8, 64u8];
        let scalar = YamlScalarValue::from_slice(&binary);
        assert_eq!(scalar.as_slice(), &binary);
    }

    /// Test null byte handling
    #[test]
    fn test_null_byte_handling() {
        let with_null = b"test\0data";
        let scalar = YamlScalarValue::from_slice(with_null);
        assert_eq!(scalar.as_slice(), with_null);
    }
}

/// Tests for memory allocation functions
mod memory_allocation_tests {
    use super::*;

    /// Test scalar allocation and deallocation
    #[test]
    fn test_allocate_scalar() {
        unsafe {
            assert!(allocate_yaml_scalar(core::ptr::null()).is_null());

            let test_str =
                b"test\0" as *const u8 as *const libc::c_char;
            let ptr = allocate_yaml_scalar(test_str);
            assert!(!ptr.is_null());

            free_yaml_memory(ptr as *mut libc::c_void);
        }
    }

    /// Test multiple allocations
    #[test]
    fn test_multiple_allocations() {
        unsafe {
            let test_str1 =
                b"first\0" as *const u8 as *const libc::c_char;
            let test_str2 =
                b"second\0" as *const u8 as *const libc::c_char;

            let ptr1 = allocate_yaml_scalar(test_str1);
            let ptr2 = allocate_yaml_scalar(test_str2);

            assert!(!ptr1.is_null());
            assert!(!ptr2.is_null());

            free_yaml_memory(ptr1 as *mut libc::c_void);
            free_yaml_memory(ptr2 as *mut libc::c_void);
        }
    }

    /// Test repeated allocation/deallocation cycles
    #[test]
    fn test_repeated_allocation() {
        unsafe {
            let test_str =
                b"test\0" as *const u8 as *const libc::c_char;

            for _ in 0..10 {
                let ptr = allocate_yaml_scalar(test_str);
                assert!(!ptr.is_null());
                free_yaml_memory(ptr as *mut libc::c_void);
            }
        }
    }
}
