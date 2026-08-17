use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};
use asr::{
    timer::TimerState,
    watcher::{Pair, Watcher},
};

#[cfg(feature = "split-index")]
use crate::silksong_memory::get_timer_current_split_index;
use crate::silksong_memory::{
    find_collectable, find_tool, get_collectables_version, get_timer_state, get_tools_version,
    read_collectable, read_tool, Env,
};

struct StoreValue<A: 'static> {
    watcher: Watcher<A>,
    interested: bool,
    get: &'static dyn Fn(Option<&Env>) -> Option<A>,
}

impl<A: Clone + Eq> StoreValue<A> {
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

    /// Produces true if the value changed, false otherwise
    fn update(&mut self, env: Option<&Env>) -> bool {
        if let Some(value) = (self.get)(env) {
            self.watcher.update_infallible(value).changed()
        } else {
            false
        }
    }
}

pub struct ToolCache {
    version: Option<i32>,
    tool: &'static [u16],
    i: i32,
    found: bool,
}

impl ToolCache {
    fn new() -> Self {
        ToolCache {
            version: None,
            tool: &[],
            i: -1,
            found: false,
        }
    }

    fn update_version(&mut self, e: Option<&Env>) {
        match e {
            None => {
                self.version = None;
                self.tool = &[]
            }
            Some(Env { pd, mem, .. }) => {
                let new = get_tools_version(mem, pd);
                if self.version != new {
                    self.version = new;
                    self.tool = &[]
                }
            }
        }
    }

    pub fn update_validity(&mut self, e: Option<&Env>) {
        if !self.tool.is_empty() {
            self.update_version(e)
        }
    }

    pub fn has_tool(&mut self, tool_utf16: &'static [u16], e: &Env) -> bool {
        self.update_version(Some(e));
        if self.version.is_none() {
            return false;
        }
        if self.tool != tool_utf16 {
            if let Some((i, is_unlocked)) = find_tool(tool_utf16, e.mem, e.pd) {
                self.i = i;
                self.found = is_unlocked;
            } else {
                self.i = -1;
                self.found = false;
            }
            self.tool = tool_utf16
        } else if !self.i.is_negative() {
            if let Some(is_unlocked) = read_tool(self.i, e.mem, e.pd) {
                self.found = is_unlocked;
            }
        }
        self.found
    }
}

/// Caches the lookup of a single collectable at a time: calls `find_collectable`
/// only when the Collectables version changes or a different item is asked for,
/// otherwise re-reads just the cached index.
pub struct CollectableCache {
    version: Option<i32>,
    item: &'static [u16],
    i: i32,
    amount: Watcher<i32>,
    interested: bool,
}

impl CollectableCache {
    fn new() -> Self {
        CollectableCache {
            version: None,
            item: &[],
            i: -1,
            amount: Watcher::new(),
            interested: false,
        }
    }

    fn update_amount(&mut self, e: &Env) {
        let new = get_collectables_version(e.mem, e.pd);
        if new.is_none() {
            self.version = None;
            self.i = -1;
            self.amount.pair = None;
        } else if self.version != new {
            self.version = new;
            if let Some((i, amount)) = find_collectable(self.item, e.mem, e.pd) {
                self.i = i;
                self.amount.update_infallible(amount);
            } else {
                // entry does not exist, assume amount 0
                self.i = -1;
                self.amount.update_infallible(0);
            }
        } else {
            if !self.i.is_negative() {
                self.amount.update(read_collectable(self.i, e.mem, e.pd));
            } else {
                // entry does not exist, assume amount 0
                self.amount.update_infallible(0);
            }
        }
    }

    pub fn update_validity(&mut self, e: Option<&Env>) {
        if self.interested {
            if let Some(e) = e {
                self.update_amount(e);
            } else {
                self.version = None;
                self.i = -1;
                self.amount.pair = None;
            }
        } else {
            self.amount.pair = None;
        }
        self.interested = false;
    }

    pub fn get_amount_pair(&mut self, item_utf16: &'static [u16], e: &Env) -> Option<&Pair<i32>> {
        self.interested = true;
        if self.item != item_utf16 {
            self.i = -1;
            self.amount.pair = None;
            self.item = item_utf16;
            self.update_amount(e);
        } else if self.amount.pair.is_none() {
            self.update_amount(e);
        }
        self.amount.pair.as_ref()
    }

    pub fn get_amount(&mut self, item_utf16: &'static [u16], e: &Env) -> Option<i32> {
        Some(self.get_amount_pair(item_utf16, e)?.current)
    }
}

pub struct Store {
    timer_state: StoreValue<TimerState>,
    #[cfg(feature = "split-index")]
    split_index: StoreValue<Option<u64>>,
    bools: BTreeMap<&'static str, StoreValue<bool>>,
    i32s: BTreeMap<&'static str, StoreValue<i32>>,
    strings: BTreeMap<&'static str, StoreValue<String>>,
    tools: ToolCache,
    collectables: CollectableCache,
}

impl Store {
    pub fn new() -> Self {
        Self {
            timer_state: StoreValue::new(&get_timer_state, None),
            #[cfg(feature = "split-index")]
            split_index: StoreValue::new(&get_timer_current_split_index, None),
            bools: BTreeMap::new(),
            i32s: BTreeMap::new(),
            strings: BTreeMap::new(),
            tools: ToolCache::new(),
            collectables: CollectableCache::new(),
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

    pub fn has_tool(&mut self, tool_utf16: &'static [u16], e: &Env) -> bool {
        self.tools.has_tool(tool_utf16, e)
    }

    pub fn get_collectable_amount(&mut self, item_utf16: &'static [u16], e: &Env) -> Option<i32> {
        self.collectables.get_amount(item_utf16, e)
    }

    pub fn get_collectable_pair(
        &mut self,
        item_utf16: &'static [u16],
        e: &Env,
    ) -> Option<&Pair<i32>> {
        self.collectables.get_amount_pair(item_utf16, e)
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

    pub fn get_string(&mut self, key: &str) -> Option<String> {
        let v = self.strings.get_mut(key)?;
        v.interested = true;
        Some(v.watcher.pair.as_ref()?.current.to_string())
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

    pub fn get_string_bang(
        &mut self,
        key: &'static str,
        get: &'static dyn Fn(Option<&Env>) -> Option<String>,
        env: Option<&Env>,
    ) -> Option<String> {
        if !self.strings.contains_key(key) {
            self.strings.insert(key, StoreValue::new(get, env));
        }
        self.get_string(key)
    }

    pub fn update_all(&mut self, env: Option<&Env>) {
        self.bools.retain(|_, v| v.interested);
        self.i32s.retain(|_, v| v.interested);
        self.strings.retain(|_, v| v.interested);
        self.timer_state.update(env);
        #[cfg(feature = "split-index")]
        self.split_index.update(env);
        self.tools.update_validity(env);
        self.collectables.update_validity(env);
        for v in self.bools.values_mut() {
            if v.update(env) {
                v.interested = false;
            }
        }
        for v in self.i32s.values_mut() {
            if v.update(env) {
                v.interested = false;
            }
        }
        for v in self.strings.values_mut() {
            if v.update(env) {
                v.interested = false;
            }
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}
