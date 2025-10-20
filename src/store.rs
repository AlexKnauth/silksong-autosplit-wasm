use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
};
use asr::watcher::{Pair, Watcher};

struct StoreValue<A> {
    watcher: Watcher<A>,
    interested: bool,
    get: Box<dyn Fn() -> Option<A>>,
}

impl<A: Clone> StoreValue<A> {
    fn new(get: Box<dyn Fn() -> Option<A>>) -> Self {
        let mut watcher = Watcher::new();
        if let Some(value) = get() {
            watcher.update_infallible(value);
        }
        StoreValue {
            watcher,
            interested: true,
            get,
        }
    }

    fn update(&mut self) {
        if let Some(value) = (self.get)() {
            self.watcher.update_infallible(value);
        }
    }
}

pub struct Store {
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

    pub fn get_bool_bang(
        &mut self,
        key: &str,
        get: Box<dyn Fn() -> Option<bool>>,
    ) -> Option<&Pair<bool>> {
        if !self.bools.contains_key(key) {
            self.bools.insert(key.to_string(), StoreValue::new(get));
        }
        self.get_bool(key)
    }

    pub fn get_i32_bang(
        &mut self,
        key: &str,
        get: Box<dyn Fn() -> Option<i32>>,
    ) -> Option<&Pair<i32>> {
        if !self.i32s.contains_key(key) {
            self.i32s.insert(key.to_string(), StoreValue::new(get));
        }
        self.get_i32(key)
    }

    pub fn update_all(&mut self) {
        self.bools.retain(|_, v| v.interested);
        self.i32s.retain(|_, v| v.interested);
        for v in self.bools.values_mut() {
            v.update();
            v.interested = false;
        }
        for v in self.i32s.values_mut() {
            v.update();
            v.interested = false;
        }
    }
}
