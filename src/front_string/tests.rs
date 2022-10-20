use super::FrontString;

#[test]
fn basic_string_ops() {
    let mut s = FrontString::new();
    s.push_str_front("cadabra");
    s.push_str_front("abra");
    assert_eq!(s, "abracadabra");
}

#[test]
fn fmt_debug() {
    let s1 = format!("{:?}", FrontString::from("asdf"));
    let s2 = format!("{:?}", String::from("asdf"));
    assert_eq!(s1, s2);
}

#[test]
fn fmt_display() {
    let s1 = format!("{}", FrontString::from("asdf"));
    let s2 = format!("{}", String::from("asdf"));
    assert_eq!(s1, s2);
}
