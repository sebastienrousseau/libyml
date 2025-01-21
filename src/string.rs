// string.rs

use crate::{
    externs::memset,
    libc,
    memory::{yaml_realloc, yaml_strdup},
    yaml::{size_t, yaml_char_t},
};

/// Extend a string buffer by reallocating and copying the existing data.
///
/// # Safety
///
/// - This function is unsafe because it directly calls the system's `realloc` and
///   `memset` functions, which can lead to undefined behaviour if misused.
/// - The caller must ensure that `start`, `pointer`, and `end` are valid pointers
///   into the same allocated memory block.
/// - The caller must ensure that the memory block being extended is large enough
///   to accommodate the new size.
/// - The caller is responsible for properly freeing the extended memory block using
///   the corresponding `yaml_free` function when it is no longer needed.
///
pub unsafe fn yaml_string_extend(
    start: *mut *mut yaml_char_t,
    pointer: *mut *mut yaml_char_t,
    end: *mut *mut yaml_char_t,
) {
    // 1) Compute the current capacity (before reallocation), storing as `size_t`.
    let current_size: size_t = (*end).offset_from(*start) as size_t;

    // 2) Record the old offset between `pointer` and `start`, also as `size_t`.
    let old_offset: size_t = (*pointer).offset_from(*start) as size_t;

    // 3) Decide the new size (for example, double).
    let new_size: size_t = current_size * 2;

    // 4) Reallocate. This may move (and free) the old pointer.
    //    It's crucial we computed `old_offset` first.
    let new_start =
        yaml_realloc((*start).cast::<libc::c_void>(), new_size)
            as *mut yaml_char_t;
    if new_start.is_null() {
        // handle out-of-memory, or let it panic
        panic!("yaml_string_extend: reallocation failed");
    }

    // 5) Optionally zero out the newly allocated region:
    //    from `current_size` up to `new_size`.
    memset(
        new_start.add(current_size as usize).cast::<libc::c_void>(),
        0,
        current_size,
    );

    // 6) Update pointers:
    //    - `*start` => new pointer
    //    - `*pointer` => offset from new_start by old_offset
    //    - `*end` => new_start + new_size
    *start = new_start;
    *pointer = new_start.add(old_offset as usize);
    *end = new_start.add(new_size as usize);
}

/// Duplicate a null-terminated string.
/// # Safety
/// - This function is unsafe because it involves memory allocation.
pub unsafe fn yaml_string_duplicate(
    str: *const yaml_char_t,
) -> *mut yaml_char_t {
    yaml_strdup(str)
}

/// Join two string buffers by copying data from one to the other.
///
/// This function is used to concatenate two string buffers.
///
/// # Safety
///
/// - This function is unsafe because it directly calls the system's `memcpy` function,
///   which can lead to undefined behaviour if misused.
/// - The caller must ensure that `a_start`, `a_pointer`, `a_end`, `b_start`, `b_pointer`,
///   and `b_end` are valid pointers into their respective allocated memory blocks.
/// - The caller must ensure that the memory blocks being joined are large enough to
///   accommodate the combined data.
/// - The caller is responsible for properly freeing the joined memory block using
///   the corresponding `yaml_free` function when it is no longer needed.
///
pub unsafe fn yaml_string_join(
    a_start: *mut *mut yaml_char_t,
    a_pointer: *mut *mut yaml_char_t,
    a_end: *mut *mut yaml_char_t,
    b_start: *mut *mut yaml_char_t,
    b_pointer: *mut *mut yaml_char_t,
    b_end: *mut *mut yaml_char_t,
) {
    // If b_start is equal to b_pointer, there's nothing to join
    if *b_start == *b_pointer {
        return;
    }

    // Calculate the length of the data in b
    let b_length = ((*b_pointer).offset_from(*b_start))
        .min((*b_end).offset_from(*b_start))
        as usize;

    // If the length of b is 0, there's nothing to copy
    if b_length == 0 {
        return;
    }

    // Ensure there's enough space in a to hold b's content
    while ((*a_end).offset_from(*a_pointer) as usize) < b_length {
        yaml_string_extend(a_start, a_pointer, a_end);
    }

    // Copy b's content to a
    core::ptr::copy_nonoverlapping(*b_start, *a_pointer, b_length);

    // Move a's pointer forward by the length of the copied data
    *a_pointer = (*a_pointer).add(b_length);
}

#[cfg(test)]
mod tests {
    use crate::{
        externs::free,
        memory::yaml_realloc,
        string::{
            yaml_string_duplicate, yaml_string_extend, yaml_string_join,
        },
        yaml::{size_t, yaml_char_t},
    };
    use core::ptr;

    #[test]
    fn test_yaml_string_extend() {
        unsafe {
            // Mock initial buffer
            let initial_size = 4;
            let mut start = yaml_realloc(ptr::null_mut(), initial_size)
                as *mut yaml_char_t;
            assert!(!start.is_null());

            let mut pointer =
                start.add(initial_size.try_into().unwrap());
            let mut end = start.add(initial_size.try_into().unwrap());

            // Write some initial data
            ptr::write_bytes(
                start,
                b'A',
                initial_size.try_into().unwrap(),
            );

            // Extend the buffer
            yaml_string_extend(&mut start, &mut pointer, &mut end);

            // Validate the new size and contents
            assert_eq!(
                end.offset_from(start),
                (initial_size * 2) as isize
            );
            for i in 0..initial_size {
                assert_eq!(*start.add(i.try_into().unwrap()), b'A');
            }
            for i in initial_size..(initial_size * 2) {
                assert_eq!(*start.add(i.try_into().unwrap()), 0); // New memory zeroed
            }
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_yaml_string_duplicate() {
        unsafe {
            // Create a source null-terminated string
            let src = b"Hello\0";
            let src_ptr =
                yaml_realloc(ptr::null_mut(), src.len() as size_t)
                    as *mut yaml_char_t;
            assert!(!src_ptr.is_null());
            ptr::copy_nonoverlapping(src.as_ptr(), src_ptr, src.len());

            // Duplicate the string
            let dup_ptr = yaml_string_duplicate(src_ptr);
            assert!(!dup_ptr.is_null());

            // Validate the duplicated string
            for i in 0..src.len() {
                assert_eq!(*dup_ptr.add(i), src[i]);
            }

            // Free allocated memory
            free(src_ptr.cast());
            free(dup_ptr.cast());
        }
    }

    #[test]
    fn test_yaml_string_join() {
        unsafe {
            // Create buffer A
            let a_size = 4;
            let mut a_start = yaml_realloc(ptr::null_mut(), a_size)
                as *mut yaml_char_t;
            assert!(!a_start.is_null());
            let mut a_pointer = a_start.add(a_size as usize);
            let mut a_end = a_start.add(a_size as usize);
            ptr::write_bytes(a_start, b'A', a_size as usize);

            // Create buffer B
            let b_size = 2;
            let mut b_start = yaml_realloc(ptr::null_mut(), b_size)
                as *mut yaml_char_t;
            assert!(!b_start.is_null());
            let mut b_pointer = b_start.add(b_size as usize);
            let mut b_end = b_start.add(b_size as usize);
            ptr::write_bytes(b_start, b'B', b_size as usize);

            // Join B into A
            yaml_string_join(
                &mut a_start,
                &mut a_pointer,
                &mut a_end,
                &mut b_start,
                &mut b_pointer,
                &mut b_end,
            );

            // Validate the joined buffer
            assert_eq!(a_pointer.offset_from(a_start), 6); // A (4) + B (2)
            for i in 0..4 {
                assert_eq!(*a_start.add(i), b'A');
            }
            for i in 4..6 {
                assert_eq!(*a_start.add(i), b'B');
            }

            // Free allocated memory
            free(a_start.cast());
            free(b_start.cast());
        }
    }
}
