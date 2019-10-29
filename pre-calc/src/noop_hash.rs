use std::collections::HashMap;
use std::default::Default;
use std::hash::{Hasher, BuildHasherDefault};

//use crate::noop_hash::PreHashedMap;

#[derive(Debug)]
pub struct NoopHasher(u64);

impl Default for NoopHasher {

    #[inline]
    fn default() -> NoopHasher {
        // "empty" Hasher
        NoopHasher(0x0)
    }
}

impl Hasher for NoopHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
    #[inline]
    fn write(&mut self, _bytes: &[u8]) {}
}

pub type NoopBuildHasher = BuildHasherDefault<NoopHasher>;
pub type PreHashedMap<K, V> = HashMap<K, V, NoopBuildHasher>;

#[cfg(test)]
mod test {
    use super::*;
    use std::hash::{BuildHasher, Hash};

    // this is the fn that https://github.com/rust-lang/hashbrown/blob/master/src/map.rs#L200 uses
    // which is the guts that std lib uses so this should test what happens in the hashmap
    fn make_hash<K: Hash + ?Sized>(hash_builder: &impl BuildHasher, val: &K) -> u64 {
        let mut state = hash_builder.build_hasher();
        val.hash(&mut state);
        state.finish()
    }

    #[test]
    fn test_noop_hash() {
        // hash of "devin"
        let cmp_done1 = 12638194897137039473_u64;
        // "abc"
        let cmp_done2 = 12638189399578898418_u64;
        // "chongo"
        let cmp_done3 = 12638193797625411262_u64;

        let cmp_done4 = 0_u64;

        let hasher = NoopBuildHasher::default();

        let hash1 = make_hash(&hasher, &cmp_done1);
        assert_eq!(hash1, cmp_done1);

        let hash2 = make_hash(&hasher, &cmp_done2);
        assert_eq!(hash2, cmp_done2);

        let hash3 = make_hash(&hasher, &cmp_done3);
        assert_eq!(hash3, cmp_done3);

        let hash4 = make_hash(&hasher, &cmp_done4);
        assert_eq!(hash4, cmp_done4);
    }
}
