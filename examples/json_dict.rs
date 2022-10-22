#![feature(cow_is_borrowed)]

use serde::Deserialize;
use serde_with::{serde_as, BorrowCow, Same};
use smallvec_map::VecMap;
use std::borrow::Cow;
use std::fmt::Debug;

struct CowMap<'a, const N: usize> {
    inner: VecMap<Cow<'a, str>, Cow<'a, str>, N>,
}

impl<'de: 'a, 'a: 'de, const N: usize> Deserialize<'de> for CowMap<'a, N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner: VecMap<Cow<'_, str>, Cow<'_, str>, N> = serde_with::As::<
            VecMap<serde_with::BorrowCow, serde_with::BorrowCow, N>,
        >::deserialize(deserializer)?;
        Ok(CowMap { inner })
    }
}

impl<'a, const N: usize> Debug for CowMap<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.inner.iter() {
            writeln!(
                f,
                "k borrowed: {}, v borrowed: {}",
                k.is_borrowed(),
                v.is_borrowed()
            );
        }
        writeln!(f, "is heap allocated? {}", self.inner.spilled());
        self.inner.fmt(f)
    }
}

fn main() {
    let data = r#"
        {
            "owo": "uwu",
            "one": "two"
        }
    "#;

    let map: CowMap<'_, 2> = serde_json::from_str(data).unwrap();
    println!("{map:#?}");
}
