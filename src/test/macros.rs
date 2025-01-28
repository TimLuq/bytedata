#[test]
fn test_macros_bytes() {
    static HW: &[u8] = crate::concat_bytes_static!(b"hello", b" ", b"world");
    assert_eq!(HW, b"hello world");
}

#[test]
fn test_macros_str() {
    static HW: &str = crate::concat_str_static!("hello", " ", "world");
    assert_eq!(HW, "hello world");
}

#[cfg(feature = "alloc")]
#[test]
fn test_macros_format_shared() {
    static CHOICES: &[&str] = &[" you", " world", "... hello... hello", "oooooooo!"];
    static CHOICE_ATOM: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    let choice = CHOICES
        [CHOICE_ATOM.fetch_add(1, core::sync::atomic::Ordering::Relaxed) as usize % CHOICES.len()];
    let hw = crate::format_shared!("{}{}", "hello", choice);
    assert_eq!(hw.len(), 5 + choice.len());
    assert!(hw.starts_with("hello"));
    assert!(hw.ends_with(choice));
}

#[cfg(all(feature = "alloc", feature = "queue"))]
#[test]
fn test_macros_format_queue() {
    static CHOICES: &[&str] = &[" you", " world", "... hello... hello", "oooooooo!"];
    static CHOICE_ATOM: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    let choice = CHOICES
        [CHOICE_ATOM.fetch_add(1, core::sync::atomic::Ordering::Relaxed) as usize % CHOICES.len()];
    let hw = crate::format_queue!("{}{}", "hello", choice);
    assert_eq!(hw.len(), 5 + choice.len());
    assert!(hw.starts_with("hello"));
    assert!(hw.ends_with(choice));
}
