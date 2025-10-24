use alloc::collections::BTreeMap;
use asr::{
    timer::TimerState,
    watcher::{Pair, Watcher},
};

#[cfg(feature = "split-index")]
use crate::silksong_memory::get_timer_current_split_index;
use crate::silksong_memory::{get_timer_state, Env};

struct StoreValue<A: 'static> {
    watcher: Watcher<A>,
    interested: bool,
    get: &'static dyn Fn(Option<&Env>) -> Option<A>,
}

impl<A: Clone> StoreValue<A> {
    fn new(get: &'static dyn Fn(Option<&Env>) -> Option<A>, env: Option<&Env>) -> Self {
        let mut watcher = Watcher::new();
        if let Some(value) = get(env) {
            watcher.update_infallible(value);
        }
        StoreValue {
            watcher,
            interested: true,
            get,
        }
    }

    fn update(&mut self, env: Option<&Env>) {
        if let Some(value) = (self.get)(env) {
            self.watcher.update_infallible(value);
        }
    }
}

pub struct Store {
    timer_state: StoreValue<TimerState>,
    #[cfg(feature = "split-index")]
    split_index: StoreValue<Option<u64>>,
    bools: BTreeMap<&'static str, StoreValue<bool>>,
    i32s: BTreeMap<&'static str, StoreValue<i32>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            timer_state: StoreValue::new(&get_timer_state, None),
            #[cfg(feature = "split-index")]
            split_index: StoreValue::new(&get_timer_current_split_index, None),
            bools: BTreeMap::new(),
            i32s: BTreeMap::new(),
        }
    }

    pub fn get_timer_state_pair(&mut self) -> Option<Pair<TimerState>> {
        self.timer_state.watcher.pair
    }

    pub fn get_timer_state_current(&mut self) -> Option<TimerState> {
        Some(self.timer_state.watcher.pair?.current)
    }

    pub fn get_split_index_pair(&mut self) -> Option<Pair<Option<u64>>> {
        #[cfg(feature = "split-index")]
        return self.split_index.watcher.pair;
        #[allow(unreachable_code)]
        None
    }

    pub fn get_split_index_current(&mut self) -> Option<u64> {
        #[cfg(feature = "split-index")]
        return self.split_index.watcher.pair?.current;
        #[allow(unreachable_code)]
        None
    }

    pub fn get_bool_pair(&mut self, key: &str) -> Option<Pair<bool>> {
        let v = self.bools.get_mut(key)?;
        v.interested = true;
        v.watcher.pair
    }

    pub fn get_i32_pair(&mut self, key: &str) -> Option<Pair<i32>> {
        let v = self.i32s.get_mut(key)?;
        v.interested = true;
        v.watcher.pair
    }

    pub fn get_bool_pair_bang(
        &mut self,
        key: &'static str,
        get: &'static dyn Fn(Option<&Env>) -> Option<bool>,
        env: Option<&Env>,
    ) -> Option<Pair<bool>> {
        if !self.bools.contains_key(key) {
            self.bools.insert(key, StoreValue::new(get, env));
        }
        self.get_bool_pair(key)
    }

    pub fn get_i32_pair_bang(
        &mut self,
        key: &'static str,
        get: &'static dyn Fn(Option<&Env>) -> Option<i32>,
        env: Option<&Env>,
    ) -> Option<Pair<i32>> {
        if !self.i32s.contains_key(key) {
            self.i32s.insert(key, StoreValue::new(get, env));
        }
        self.get_i32_pair(key)
    }

    pub fn update_all(&mut self, env: Option<&Env>) {
        self.bools.retain(|_, v| v.interested);
        self.i32s.retain(|_, v| v.interested);
        self.timer_state.update(env);
        #[cfg(feature = "split-index")]
        self.split_index.update(env);
        for v in self.bools.values_mut() {
            v.update(env);
            v.interested = false;
        }
        for v in self.i32s.values_mut() {
            v.update(env);
            v.interested = false;
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}
