use sha2::{Digest, Sha256};

pub mod permissions;

pub use permissions::Perm;

/// Hash password using SHA-256
pub fn hash_password(plain: &str) -> String {
    let mut h = Sha256::new();
    h.update(plain.as_bytes());
    let res = h.finalize();
    res.iter().map(|b| format!("{:02x}", b)).collect()
}

