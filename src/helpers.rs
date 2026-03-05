#[cfg(debug_assertions)]
pub(crate) fn missing_tag_warning(s: &str) {
    panic!("{}", s);
}

#[cfg(not(debug_assertions))]
pub(crate) fn missing_tag_warning(_s: &str) {}
