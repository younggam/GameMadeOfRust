#[macro_export]
macro_rules! unreachable_release {
    ($info:literal) => {
        unsafe {
            debug_assert!(false, $info);
            std::hint::unreachable_unchecked();
        }
    };
}
