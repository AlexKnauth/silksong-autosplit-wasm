use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
};
use asr::{
    timer::TimerState,
    watcher::{Pair, Watcher},
};

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
    timer_state: StoreValue<TimerState>,
    #[cfg(feature = "split-index")]
    split_index: StoreValue<Option<u64>>,
    bools: BTreeMap<String, StoreValue<bool>>,
    i32s: BTreeMap<String, StoreValue<i32>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            timer_state: StoreValue::new(Box::new(|| Some(asr::timer::state()))),
            #[cfg(feature = "split-index")]
            split_index: StoreValue::new(Box::new(|| Some(asr::timer::current_split_index()))),
            bools: BTreeMap::new(),
            i32s: BTreeMap::new(),
        }
    }

    pub fn get_timer_state_pair(&mut self) -> Option<Pair<TimerState>> {
        self.timer_state.watcher.pair.clone()
    }

    pub fn get_timer_state_current(&mut self) -> Option<TimerState> {
        Some(self.timer_state.watcher.pair?.current)
    }

    pub fn get_split_index_pair(&mut self) -> Option<Pair<Option<u64>>> {
        #[cfg(feature = "split-index")]
        return self.split_index.watcher.pair.clone();
        #[allow(unreachable_code)]
        None
    }

    pub fn get_split_index_current(&mut self) -> Option<u64> {
        #[cfg(feature = "split-index")]
        return self.split_index.watcher.pair?.current;
        #[allow(unreachable_code)]
        None
    }

    fn get_bool_pair(&mut self, key: &str) -> Option<Pair<bool>> {
        let v = self.bools.get_mut(key)?;
        v.interested = true;
        v.watcher.pair.clone()
    }

    fn get_i32_pair(&mut self, key: &str) -> Option<Pair<i32>> {
        let v = self.i32s.get_mut(key)?;
        v.interested = true;
        v.watcher.pair.clone()
    }

    pub fn get_bool_pair_bang(
        &mut self,
        key: &str,
        get: Box<dyn Fn() -> Option<bool>>,
    ) -> Option<Pair<bool>> {
        if !self.bools.contains_key(key) {
            self.bools.insert(key.to_string(), StoreValue::new(get));
        }
        self.get_bool_pair(key)
    }

    pub fn get_i32_pair_bang(
        &mut self,
        key: &str,
        get: Box<dyn Fn() -> Option<i32>>,
    ) -> Option<Pair<i32>> {
        if !self.i32s.contains_key(key) {
            self.i32s.insert(key.to_string(), StoreValue::new(get));
        }
        self.get_i32_pair(key)
    }

    pub fn update_all(&mut self) {
        self.bools.retain(|_, v| v.interested);
        self.i32s.retain(|_, v| v.interested);
        self.timer_state.update();
        #[cfg(feature = "split-index")]
        self.split_index.update();
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

impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}
