#[cfg(test)]
mod tests {
    use core::ffi::c_void;
    use libyml::api::yaml_malloc;
    use libyml::externs::free;

    #[test]
    fn test_yaml_malloc() {
        unsafe {
            // Test allocation of zero bytes
            let ptr = yaml_malloc(0);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory

            // Test allocation of non-zero bytes
            let ptr = yaml_malloc(10);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory
        }
    }

    // Helper function to free memory
    unsafe fn yaml_free(ptr: *mut c_void) {
        free(ptr);
    }
}
