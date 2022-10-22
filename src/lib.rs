#[cfg(feature = "serde")]
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use serde::{ser::SerializeMap, Serialize};

#[cfg(feature = "serde_with")]
use serde::Deserializer;
#[cfg(feature = "serde_with")]
use serde_with::{de::DeserializeAsWrap, DeserializeAs};

use smallvec::SmallVec;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use std::{marker::PhantomData, mem};

#[repr(transparent)]
pub struct VecMap<K, V, const N: usize> {
    inner: SmallVec<[(K, V); N]>,
}

impl<K, V, const N: usize> VecMap<K, V, N> {
    #[inline]
    pub fn new() -> VecMap<K, V, N> {
        VecMap {
            inner: SmallVec::new_const(),
        }
    }

    #[inline]
    pub fn with_capacity(n: usize) -> VecMap<K, V, N> {
        VecMap {
            inner: SmallVec::with_capacity(n),
        }
    }
}

impl<K: Ord, V, const N: usize> VecMap<K, V, N> {
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        match self.inner.binary_search_by(|v| k.cmp(&v.0)) {
            Ok(idx) => Some(mem::replace(&mut self.inner[idx].1, v)),
            Err(idx) => {
                self.inner.insert(idx, (k, v));
                None
            }
        }
    }

    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .binary_search_by(|v| k.cmp(v.0.borrow()))
            .ok()
            .and_then(|idx| self.inner.get(idx))
            .map(|v| &v.1)
    }

    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner
            .binary_search_by(|v| k.cmp(v.0.borrow()))
            .ok()
            .and_then(|idx| self.inner.get_mut(idx))
            .map(|v| &mut v.1)
    }

    #[inline]
    pub fn contains<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.inner.binary_search_by(|v| k.cmp(v.0.borrow())).is_ok()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn spilled(&self) -> bool {
        self.inner.spilled()
    }

    #[inline]
    pub fn iter(&self) -> VecMapIter<'_, K, V> {
        VecMapIter {
            inner: self.inner.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> VecMapIterMut<'_, K, V> {
        VecMapIterMut {
            inner: self.inner.iter_mut(),
        }
    }
}

impl<K, V, const N: usize> Default for VecMap<K, V, N> {
    #[inline]
    fn default() -> VecMap<K, V, N> {
        Self::new()
    }
}

/* index */

impl<K, Q: ?Sized, V, const N: usize> Index<&Q> for VecMap<K, V, N>
where
    K: Ord + Borrow<Q>,
    Q: Ord,
{
    type Output = V;

    #[inline]
    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<K, Q: ?Sized, V, const N: usize> IndexMut<&Q> for VecMap<K, V, N>
where
    K: Ord + Borrow<Q>,
    Q: Ord,
{
    #[inline]
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

/* iter */

impl<K, V, const N: usize> IntoIterator for VecMap<K, V, N> {
    type Item = (K, V);

    type IntoIter = <SmallVec<[(K, V); N]> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[repr(transparent)]
pub struct VecMapIter<'a, K, V> {
    inner: std::slice::Iter<'a, (K, V)>,
}

impl<'a, K, V> Iterator for VecMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (k, v))
    }
}

#[repr(transparent)]
pub struct VecMapIterMut<'a, K, V> {
    inner: std::slice::IterMut<'a, (K, V)>,
}

impl<'a, K, V> Iterator for VecMapIterMut<'a, K, V> {
    type Item = (&'a mut K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (k, v))
    }
}

/* out of iter land! */

impl<K, V, const N: usize> Debug for VecMap<K, V, N>
where
    K: Debug + Ord,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

/* serde */

#[cfg(feature = "serde")]
struct VecMapVisitor<K, V, const N: usize> {
    _spooky: PhantomData<fn() -> VecMap<K, V, N>>,
}

#[cfg(feature = "serde")]
impl<K, V, const N: usize> VecMapVisitor<K, V, N> {
    fn new() -> Self {
        VecMapVisitor {
            _spooky: PhantomData,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V, const N: usize> Visitor<'de> for VecMapVisitor<K, V, N>
where
    K: Ord + Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = VecMap<K, V, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a hashmap/dictionary")
    }

    fn visit_map<M>(self, mut acc: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: VecMap<K, V, N> = VecMap::with_capacity(acc.size_hint().unwrap_or(0));

        while let Some(tuple) = acc.next_entry()? {
            map.inner.push(tuple);
        }

        map.inner.sort_unstable_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        map.inner.dedup_by(|lhs, rhs| lhs.0 == rhs.0);

        Ok(map)
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V, const N: usize> Deserialize<'de> for VecMap<K, V, N>
where
    K: Ord + Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(VecMapVisitor::new())
    }
}

#[cfg(feature = "serde")]
impl<K, V, const N: usize> Serialize for VecMap<K, V, N>
where
    K: Ord + Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, v)?;
        }

        map.end()
    }
}

#[cfg(feature = "serde_with")]
struct AsVecMapVisitor<K, V, Q, I, const N: usize> {
    _spooky: PhantomData<fn() -> VecMap<K, V, N>>,
    _spookier: PhantomData<fn() -> VecMap<Q, I, N>>,
}

#[cfg(feature = "serde_with")]
impl<K, V, Q, I, const N: usize> AsVecMapVisitor<K, V, Q, I, N> {
    fn new() -> Self {
        AsVecMapVisitor {
            _spooky: PhantomData,
            _spookier: PhantomData,
        }
    }
}

#[cfg(feature = "serde_with")]
impl<'de, K, V, Q, I, const N: usize> Visitor<'de> for AsVecMapVisitor<K, V, Q, I, N>
where
    K: Ord,
    Q: DeserializeAs<'de, K>,
    I: DeserializeAs<'de, V>,
{
    type Value = VecMap<K, V, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a hashmap/dictionary")
    }

    fn visit_map<M>(self, mut acc: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: VecMap<K, V, N> = VecMap::with_capacity(acc.size_hint().unwrap_or(0));

        while let Some((k, v)) =
            acc.next_entry::<DeserializeAsWrap<K, Q>, DeserializeAsWrap<V, I>>()?
        {
            map.inner.push((k.into_inner(), v.into_inner()));
        }

        map.inner.sort_unstable_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        map.inner.dedup_by(|lhs, rhs| lhs.0 == rhs.0);

        Ok(map)
    }
}

#[cfg(feature = "serde_with")]
impl<'de, K, V, Q, I, const N: usize> DeserializeAs<'de, VecMap<K, V, N>> for VecMap<Q, I, N>
where
    K: Ord,
    Q: DeserializeAs<'de, K>,
    I: DeserializeAs<'de, V>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<VecMap<K, V, N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(AsVecMapVisitor::<K, V, Q, I, N>::new())
    }
}
