use equivalent::Comparable;

#[derive(Debug)]
pub struct SortedVec<K, V> {
    inner: Vec<(K, V)>,
}

impl<K, V> Default for SortedVec<K, V> {
    fn default() -> Self {
        Self { inner: vec![] }
    }
}

pub enum Entry<'a, K, V, Q> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V, Q>),
}

pub struct OccupiedEntry<'a, K, V> {
    pos: usize,
    vec: &'a mut Vec<(K, V)>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    pub fn key_value(&'a self) -> (&'a K, &'a V) {
        let (k, v) = &self.vec[self.pos];
        (k, v)
    }

    pub fn key(&'a self) -> &'a K {
        self.key_value().0
    }

    pub fn value(&'a self) -> &'a V {
        self.key_value().1
    }

    pub fn into_mut_value(self) -> &'a mut V {
        &mut self.vec[self.pos].1
    }
}

pub struct VacantEntry<'a, K, V, Q> {
    pos: usize,
    key: Q,
    vec: &'a mut Vec<(K, V)>,
}

impl<'a, K, V, Q: Into<K>> VacantEntry<'a, K, V, Q> {
    pub fn insert(self, element: V) -> OccupiedEntry<'a, K, V> {
        let Self { pos, key, vec } = self;
        vec.insert(pos, (key.into(), element));
        OccupiedEntry { pos, vec }
    }
}

impl<K: Ord, V> SortedVec<K, V> {
    pub fn get_mut_or_insert_default<Q>(&mut self, key: Q) -> &mut V
    where
        V: Default,
        Q: Into<K> + Comparable<K>,
    {
        match self.entry(key) {
            Entry::Occupied(entry) => entry.into_mut_value(),
            Entry::Vacant(entry) => entry.insert(Default::default()).into_mut_value(),
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        Q: ?Sized + Comparable<K>,
    {
        self.entry_inner(key).get(&self.inner)
    }

    pub fn entry_ref<'q, Q>(&mut self, key: &'q Q) -> Entry<K, V, &'q Q>
    where
        Q: ?Sized + Comparable<K>,
        &'q Q: Into<K>,
    {
        self.entry_inner(key).into_entry(&mut self.inner, key)
    }

    pub fn entry<Q>(&mut self, key: Q) -> Entry<K, V, Q>
    where
        Q: Into<K> + Comparable<K>,
    {
        self.entry_inner(&key).into_entry(&mut self.inner, key)
    }

    fn entry_inner<Q>(&self, key: &Q) -> EntryInner
    where
        Q: ?Sized + Comparable<K>,
    {
        match self
            .inner
            .binary_search_by(|(k, _)| key.compare(k).reverse())
        {
            Ok(pos) => EntryInner {
                pos,
                occupied: true,
            },
            Err(pos) => EntryInner {
                pos,
                occupied: false,
            },
        }
    }
}

struct EntryInner {
    pos: usize,
    occupied: bool,
}

impl EntryInner {
    fn into_entry<K, V, Q>(self, vec: &mut Vec<(K, V)>, key: Q) -> Entry<K, V, Q> {
        let EntryInner { pos, occupied } = self;
        if occupied {
            Entry::Occupied(OccupiedEntry { pos, vec })
        } else {
            Entry::Vacant(VacantEntry { pos, key, vec })
        }
    }

    fn get<K, V>(self, vec: &[(K, V)]) -> Option<&V> {
        let EntryInner { pos, occupied } = self;
        if occupied { Some(&vec[pos].1) } else { None }
    }
}
