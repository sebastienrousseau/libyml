// internals.rs

use crate::{
    externs::memmove,
    libc,
    memory::yaml_realloc,
    ops::{ForceAdd as _, ForceMul as _},
    success::{Success, FAIL, OK},
    yaml::{size_t, yaml_char_t},
    PointerExt,
};

/// Extend a stack by reallocating and copying the existing data.
///
/// This function is used to grow a stack when more space is needed.
///
/// # Safety
///
/// - This function is unsafe because it directly calls the system's `realloc` function,
///   which can lead to undefined behaviour if misused.
/// - The caller must ensure that `start`, `top`, and `end` are valid pointers into the
///   same allocated memory block.
/// - The caller must ensure that the memory block being extended is large enough to
///   accommodate the new size.
/// - The caller is responsible for properly freeing the extended memory block using
///   the corresponding `yaml_free` function when it is no longer needed.
///
pub unsafe fn yaml_stack_extend(
    start: *mut *mut libc::c_void,
    top: *mut *mut libc::c_void,
    end: *mut *mut libc::c_void,
) {
    let new_start: *mut libc::c_void = yaml_realloc(
        *start,
        (((*end as *mut libc::c_char)
            .c_offset_from(*start as *mut libc::c_char)
            as libc::c_long)
            .force_mul(2_i64)) as size_t,
    );
    *top = (new_start as *mut libc::c_char).wrapping_offset(
        (*top as *mut libc::c_char)
            .c_offset_from(*start as *mut libc::c_char)
            as libc::c_long as isize,
    ) as *mut libc::c_void;
    *end = (new_start as *mut libc::c_char).wrapping_offset(
        (((*end as *mut libc::c_char)
            .c_offset_from(*start as *mut libc::c_char)
            as libc::c_long)
            .force_mul(2_i64)) as isize,
    ) as *mut libc::c_void;
    *start = new_start;
}

/// Extend a queue by reallocating (doubling) if necessary or moving existing data.
///
/// # Safety
///
/// - The caller must ensure `start`, `head`, `tail`, and `end` all point into
///   the same allocated memory block.
/// - If `tail == end`, this function will attempt to move data (if there is front space),
///   or reallocate with double the current capacity.
/// - The caller is responsible for eventually freeing the block with `yaml_free`.
///
pub unsafe fn yaml_queue_extend(
    start: *mut *mut libc::c_void,
    head: *mut *mut libc::c_void,
    tail: *mut *mut libc::c_void,
    end: *mut *mut libc::c_void,
) {
    use crate::libc;
    use crate::memory::yaml_realloc;

    // 1) If `tail` has reached `end`, we either move data to the front or reallocate.
    if *tail == *end {
        // Calculate how many bytes are currently used in the queue
        let used_bytes = (*tail as *mut libc::c_char)
            .c_offset_from(*head as *mut libc::c_char);

        // Calculate the total capacity
        let capacity = (*end as *mut libc::c_char)
            .c_offset_from(*start as *mut libc::c_char);

        // 2) If there's space at the front (i.e., head != start), move data down to index 0
        if *head != *start {
            // Move the used portion to the front
            let _ = memmove(*start, *head, used_bytes as libc::c_ulong);

            // Update `tail` to be right after the moved bytes
            *tail = (*start as *mut libc::c_char)
                .add(used_bytes as usize)
                as *mut libc::c_void;

            // `head` now points to the start of the block
            *head = *start;
        } else {
            // 3) Otherwise, there's no free space in the front, so we must reallocate (double capacity)
            let new_capacity = capacity * 2;

            let new_start = yaml_realloc(
                *start,
                (new_capacity as usize).try_into().unwrap(),
            );
            if new_start.is_null() {
                // Handle error: out-of-memory or panic, etc.
                // For now, just return or panic
                panic!("yaml_queue_extend: reallocation failed");
            }

            // Recalculate the old offsets
            let old_start_char = *start as *mut libc::c_char;
            let head_offset = (*head as *mut libc::c_char)
                .c_offset_from(old_start_char);
            let tail_offset = (*tail as *mut libc::c_char)
                .c_offset_from(old_start_char);

            // Store the new base pointer
            *start = new_start;
            let new_start_char = new_start as *mut libc::c_char;

            // Update head/tail pointers to new positions
            *head = new_start_char.add(head_offset as usize)
                as *mut libc::c_void;
            *tail = new_start_char.add(tail_offset as usize)
                as *mut libc::c_void;

            // `end` is now start + new_capacity
            *end = new_start_char.add(new_capacity as usize)
                as *mut libc::c_void;
        }
    }

    // If `*tail` hasn't reached `*end`, no action is required.
}

/// Checks if the provided UTF-8 encoded string is valid according to the UTF-8 specification.
///
/// # Parameters
///
/// * `start`: A pointer to the start of the UTF-8 encoded string.
/// * `length`: The length of the UTF-8 encoded string in bytes.
///
/// # Return
///
/// Returns `Success::OK` if the string is valid UTF-8, otherwise returns `Success::FAIL`.
///
/// # Safety
///
/// - `start` must be a valid, non-null pointer to a null-terminated UTF-8 string.
/// - The UTF-8 encoded string must be properly formatted and not contain any invalid characters.
/// - The string must be properly null-terminated.
/// - The string must not contain any invalid characters or sequences.
///
pub unsafe fn yaml_check_utf8(
    start: *const yaml_char_t,
    length: size_t,
) -> Success {
    let end: *const yaml_char_t =
        start.wrapping_offset(length as isize);
    let mut pointer: *const yaml_char_t = start;

    while pointer < end {
        let mut octet: libc::c_uchar;
        let mut value: libc::c_uint;
        let mut k: size_t;

        octet = *pointer;
        let width: libc::c_uint = if octet & 0x80 == 0 {
            1
        } else if octet & 0xE0 == 0xC0 {
            2
        } else if octet & 0xF0 == 0xE0 {
            3
        } else if octet & 0xF8 == 0xF0 {
            4
        } else {
            0
        } as libc::c_uint;

        value = if octet & 0x80 == 0 {
            octet & 0x7F
        } else if octet & 0xE0 == 0xC0 {
            octet & 0x1F
        } else if octet & 0xF0 == 0xE0 {
            octet & 0xF
        } else if octet & 0xF8 == 0xF0 {
            octet & 0x7
        } else {
            0
        } as libc::c_uint;

        if width == 0 {
            return FAIL;
        }

        if pointer.wrapping_offset(width as isize) > end {
            return FAIL;
        }

        k = 1_u64;
        while k < width as libc::c_ulong {
            octet = *pointer.wrapping_offset(k as isize);
            if octet & 0xC0 != 0x80 {
                return FAIL;
            }
            value =
                (value << 6).force_add((octet & 0x3F) as libc::c_uint);
            k = k.force_add(1);
        }

        if !(width == 1
            || width == 2 && value >= 0x80
            || width == 3 && value >= 0x800
            || width == 4 && value >= 0x10000)
        {
            return FAIL;
        }

        pointer = pointer.wrapping_offset(width as isize);
    }

    OK
}
