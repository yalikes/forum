#![allow(unused)]
use openssl::sha::sha256;
use rand::{self, Rng};

pub fn check(password: &str, salt: &[u8], password_hash: &[u8]) -> bool {
    if salt.len() != 32{
        return false;
    }
    if password_hash.len() != 32{
        return false;
    }
    openssl::sha::sha256(&[salt, password.as_bytes()].concat()) == password_hash
}

pub fn generate_salt_and_hash(password: &str) -> ([u8; 32], [u8; 32]) {
    let mut rng = rand::thread_rng();
    let mut salt_buf: [u8; 32] = rng.gen();
    (
        sha256(&[&salt_buf, password.as_bytes()].concat()),
        salt_buf,
    )
}

fn utochar(n: u8) -> char {
    match n {
        0..=9 => (n + (b'0')) as char,
        _ => ((n - 10) + (b'A')) as char,
    }
}

#[test]
fn test_check() {
    let password = "314159265TESTpassword";
    let salt = &[0,1,2,3,4,5,6,7,0,1,2,3,4,5,6,7,0,1,2,3,4,5,6,7,0,1,2,3,4,5,6,7,];
    let password_hash = sha256(&[salt, password.as_bytes()].concat());
    assert!(check(password, salt, &password_hash));
}

#[test]
fn test_utochar() {
    let testchars = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    ];
    for (i, c) in testchars.iter().enumerate() {
        assert_eq!(&utochar(i as u8), c);
    }
}
