#[cfg(debug_assertions)]
pub(crate) fn missing_tag_warning(_s: &str) {
    panic!("{}", _s);
}

#[cfg(not(debug_assertions))]
pub(crate) fn missing_tag_warning(_s: &str) {}
