use sha2::{Digest, Sha256};

pub fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn hash_files(files: &[String]) -> String {
    let mut sorted = files.to_vec();
    sorted.sort();
    hash_bytes(sorted.join("\0").as_bytes())
}
