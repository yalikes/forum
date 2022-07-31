use openssl::sha::sha256;
use rand::{self, Rng};

pub fn check(password: &str, salt: &str, password_hash: [u8; 32]) -> bool {
    // TODO: optimize here
    openssl::sha::sha256((salt.to_owned() + password).as_bytes()) == password_hash
}

pub fn generate_salt_and_hash(password: &str) -> ([u8; 32], [char; 32]) {
    let mut rng = rand::thread_rng();
    let mut salt_buf: [char; 32] = [0 as char; 32];
    for x in &mut salt_buf {
        let n: u8 = rng.gen_range(0..16);
        *x = utochar(n);
    }
    (
        sha256((salt_buf.iter().collect::<String>() + password).as_bytes()),
        salt_buf,
    )
}

fn utochar(n: u8) -> char {
    match n {
        0..=9 => (n + ('0' as u8)) as char,
        _ => ((n - 10) + ('A' as u8)) as char,
    }
}

#[test]
fn test_check() {
    let password = "314159265TESTpassword";
    let salt = "12345678123456781234567812345678";
    let password_hash = sha256((salt.to_owned() + password).as_bytes());
    assert!(check(password, salt, password_hash));
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
