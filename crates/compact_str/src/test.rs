/*========================================================================*\
** NotVeryMoe BevyUtil | Copyright 2021 NotVeryMoe (projects@notvery.moe) **
\*========================================================================*/

use super::*;

#[test]
fn empty() {
    let v = CompactStr8::new("");
    assert_eq!(v.len(),    0);
    assert_eq!(v.to_raw(), 0);
}

#[test]
fn overflow() {
    assert!(CompactStr8::try_new(""   ).is_ok());
    assert!(CompactStr8::try_new("a"  ).is_ok());
    assert!(CompactStr8::try_new("aa" ).is_err());
    assert!(CompactStr8::try_new("aaa").is_err());
    assert!(CompactStr8::try_new("a1a").is_err());
}

#[test]
fn chars() {
    fn character(ch: char) {
        let v = CompactStr8::try_new(&ch.to_string()).unwrap();
        let ch = ch as u8;
        assert_eq!(v.len(), 1);
        assert_eq!(v.to_raw(), match ch {
            b'a'..=b'z' => ch-95,
            b'A'..=b'Z' => ch-63,
            _           => 1
        }, "{}", ch as char);
    }

    let chars = "_abcdefghijklmnopqrstuvwxyz ";
    chars.chars()
        .chain(chars.to_uppercase().chars())
        .for_each(character);
}

#[test]
#[allow(clippy::unusual_byte_groupings)]
fn short() {
    let str = "abc";
    let v = CompactStr16::new(str);
    let str = sanitize_str(str);

    assert_eq!(v.len(),    str.len());
    assert_eq!(v.to_str(), str);

    // 5-bit groupings across byte boundaries
    assert_eq!(v.to_raw(), 0b_00100_00011_00010, "{}", v);
}

#[test]
#[allow(clippy::unusual_byte_groupings)]
fn long() {
    let v = [
        CompactStr64::new("ABCDEFGH"),
        CompactStr64::new("IJKLMNOP"),
        CompactStr64::new("QRSTUVWX"),
        CompactStr64::new("YZ_ "),
    ];

    // 5-bit groupings across byte boundaries
    for (i, &raw) in [
        0b01001_01000_00111_00110_00101_00100_00011_00010u64,
        0b10001_10000_01111_01110_01101_01100_01011_01010u64,
        0b11001_11000_10111_10110_10101_10100_10011_10010u64,
        0b00000_00000_00000_00000_00001_00001_11011_11010u64,
    ].iter().enumerate() {
        assert_eq!(v[i].to_raw(), raw, "{}", i); 
    }
}


fn sanitize_str(s: &str) -> String {
    s.chars().map(|c| match c {
        'a'..='z' | 'A'..='Z' => c.to_ascii_uppercase(),
        _ => '_',
    }).collect()
}