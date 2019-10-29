use std::hash::{Hash, Hasher};

use fnv::FnvHasher;

pub(crate) fn fnv_key<T: Hash, U: Hash>(to_hash: (&T, &U)) -> u64 {
    let mut hasher = FnvHasher::default();
    to_hash.hash(&mut hasher);
    hasher.finish()
}
