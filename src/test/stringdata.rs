use alloc::borrow::ToOwned;

use crate::StringData;

#[test]
fn test_stringdata() {
    let s0 = StringData::from_cow("hello world".into());
    assert_eq!(s0, "hello world");
    assert_eq!(s0.as_str(), "hello world");
    assert_eq!(s0.as_bytes(), b"hello world");
    assert_eq!(s0.len(), 11);
    assert!(!s0.is_empty());
    assert_eq!(&s0[1..3], "el");

    let s1 = StringData::from_owned("hello world".to_owned());
    assert_eq!(s1, "hello world");
    assert_eq!(s1.as_str(), "hello world");
    assert_eq!(s1.as_bytes(), b"hello world");
    assert_eq!(s1.len(), 11);
    assert!(!s1.is_empty());
    assert_eq!(&s1[1..3], "el");
}
