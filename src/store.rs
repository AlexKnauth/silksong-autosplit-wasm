use alloc::{collections::BTreeMap, string::{String, ToString}};
use asr::watcher::{Pair, Watcher};

struct StoreValue<A> {
    watcher: Watcher<A>,
    interested: bool,
}

impl<A: Clone> StoreValue<A> {
    fn new(value: A) -> Self {
        let mut watcher = Watcher::new();
        watcher.update_infallible(value);
        StoreValue { watcher, interested: true }
    }
}

struct Store {
    bools: BTreeMap<String, StoreValue<bool>>,
    i32s: BTreeMap<String, StoreValue<i32>>,
}

impl Store {
    fn get_bool(&mut self, key: &str) -> Option<&Pair<bool>> {
        let v = self.bools.get_mut(key)?;
        v.interested = true;
        v.watcher.pair.as_ref()
    }

    fn get_i32(&mut self, key: &str) -> Option<&Pair<i32>> {
        let v = self.i32s.get_mut(key)?;
        v.interested = true;
        v.watcher.pair.as_ref()
    }

    fn update_bool(&mut self, key: &str, value: bool) {
        if let Some(v) = self.bools.get_mut(key) {
            v.watcher.update_infallible(value);
        } else {
            self.bools.insert(key.to_string(), StoreValue::new(value));
        }
    }

    fn update_i32(&mut self, key: &str, value: i32) {
        if let Some(v) = self.i32s.get_mut(key) {
            v.watcher.update_infallible(value);
        } else {
            self.i32s.insert(key.to_string(), StoreValue::new(value));
        }
    }
}
