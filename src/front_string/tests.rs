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

#[test]
fn char_to_bytes() {
    // From https://www.compart.com/en/unicode/U+306C
    let expected: [u8; 3] = [0xE3, 0x81, 0xAC];

    let ch = 'ぬ';
    let mut buf = [0; 4];
    let bytes = ch.encode_utf8(&mut buf);
    assert_eq!(bytes.as_bytes(), &expected);
}

#[test]
fn unicode_characters() {
    // Example taken from here: https://www.cl.cam.ac.uk/~mgk25/ucs/examples/quickbrown.txt
    let full = "いろはにほへとちりぬるを";
    let first = 'い';
    let middle = "ろはにほ";
    let ch = 'へ';
    let end = "とちりぬるを";

    let mut s = FrontString::from(end);
    s.push_char_front(ch);
    s.push_str_front(middle);
    s.push_char_front(first);

    assert_eq!(s, full);
}
