use std::collections::HashMap;
use std::hash::Hash;

use std::sync::Mutex;

pub struct TrackedMethod {
    calls_exact: Option<i64>,
    name: String
}

impl TrackedMethod {
    fn new(name: String) -> Self {
        TrackedMethod {
            calls_exact: None,
            name 
        }
    }

    fn was_called(&mut self) {
        if let Some(calls) = self.calls_exact {
            self.calls_exact = Some(calls - 1);
        }
    }

    fn verify(&self) {
        match self.calls_exact {
            Some(x) if x < 0 => panic!("{} was called {} times more than expected", self.name, x.abs()),
            Some(x) if x > 0 => panic!("{} was called {} times fewer than expected", self.name, x),
            _ => { }
        };
    }
}

macro_rules! get_tracked_method {
    ($target:ident, $key:ident, $name:ident) => {
        $target.0.lock().unwrap().entry($key).or_insert_with(|| TrackedMethod::new($name.into()))
    }
}

pub struct TrackedMethodGuard<'a, K>(&'a mut ExpectationStoreInner<K>, Option<K>, Option<String>) where
    K: 'a + Eq + Hash;

impl<'a, K> TrackedMethodGuard<'a, K> where
    K: 'a + Eq + Hash
{

    fn new(hash: &'a mut ExpectationStoreInner<K>, key: K, name: String) -> Self {
        TrackedMethodGuard(hash, Some(key), Some(name))
    }

    pub fn called_never(&mut self) {
        self.called_times(0);
    }

    pub fn called_once(&mut self) {
        self.called_times(1);
    }

    pub fn called_times(&mut self, calls: i64) {
        let key = self.1.take().unwrap();
        let name = self.2.take().unwrap();
        get_tracked_method!(self, key, name).calls_exact = Some(calls);
    }
}

type ExpectationStoreInner<K> = Mutex<HashMap<K, TrackedMethod>>;

pub struct ExpectationStore<K> where
    K: Eq + Hash
{
    inner: ExpectationStoreInner<K>
}

impl<K> ExpectationStore<K> where
    K: Eq + Hash
{
    /// Create a new `ExpectationStore` instance. Call this when your mock object is created,
    /// and store the `ExpectaionStore` object in it.
    pub fn new() -> Self {
        ExpectationStore {
            inner: Mutex::new(HashMap::new())
        }
    }

    /// When a tracked method is called on the mock object, call this with the method's key
    /// in order to tell the `ExpectationStore` that the method was called.
    pub fn was_called(&self, key: K) {
        if self.is_tracked(&key) {
            self.inner.lock().unwrap().get_mut(&key).unwrap().was_called();
        }
    }

    /// Signify that you'd like the `ExpectationStore` to track a method with the given key and name.
    pub fn track_method<'a, S: Into<String>>(&'a mut self, key: K, name: S) -> TrackedMethodGuard<'a, K> {
        TrackedMethodGuard::new(&mut self.inner, key, name.into())
    }

    fn is_tracked(&self, key: &K) -> bool {
        self.inner.lock().unwrap().contains_key(key)
    }

    fn verify(&self) {
        for (_, exp) in self.inner.lock().unwrap().iter() {
            exp.verify();
        }
    }
}

impl<K> Drop for ExpectationStore<K> where
    K: Eq + Hash
{
    /// All expectations will be verified when the mock object is dropped.
    fn drop(&mut self) {
        self.verify();
    }
}
