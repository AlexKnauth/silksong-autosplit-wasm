#[cfg(debug_assertions)]
use alloc::format;
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
#[cfg(debug_assertions)]
use crate::silksong_memory::scan_all_collectables;
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

pub struct CollectableCache {
    version: Option<i32>,
    // The item name this cache's amount/prev_amount history belongs to. Only changes when the
    // caller queries a different item - a Collectables version bump does NOT clear this, so
    // history survives across re-scans (see `stale`).
    item: &'static [u16],
    // Set on a Collectables version bump (or construction): the cached index `i` may no longer
    // be valid and must be re-derived via a full find_collectable scan before it's trusted again.
    stale: bool,
    i: i32,
    prev_amount: i32,
    amount: i32,
}

impl CollectableCache {
    fn new() -> Self {
        CollectableCache {
            version: None,
            item: &[],
            stale: true,
            i: -1,
            prev_amount: 0,
            amount: 0,
        }
    }

    fn update_version(&mut self, e: Option<&Env>) {
        match e {
            None => {
                self.version = None;
                self.stale = true;
            }
            Some(Env { pd, mem, .. }) => {
                let new = get_collectables_version(mem, pd);
                if self.version != new {
                    self.version = new;
                    self.stale = true;
                }
            }
        }
    }

    pub fn update_validity(&mut self, e: Option<&Env>) {
        if !self.item.is_empty() {
            self.update_version(e)
        }
    }

    /// Returns the current amount owned for `item_utf16`. Also updates the previous-amount
    /// history used by `increased`, so calling this directly instead of through `increased`
    /// still keeps that history correct.
    pub fn get_amount(&mut self, item_utf16: &'static [u16], e: &Env) -> i32 {
        self.update_version(Some(e));
        let item_switched = self.item != item_utf16;
        self.prev_amount = self.amount;
        if item_switched {
            self.item = item_utf16;
            self.stale = true;
        }
        if self.version.is_none() {
            self.i = -1;
            self.amount = 0;
        } else if self.stale {
            if let Some((i, amount)) = find_collectable(item_utf16, e.mem, e.pd) {
                self.i = i;
                self.amount = amount;
            } else {
                self.i = -1;
                self.amount = 0;
            }
            self.stale = false;
        } else if !self.i.is_negative() {
            if let Some(amount) = read_collectable(self.i, e.mem, e.pd) {
                self.amount = amount;
            }
        }
        if item_switched {
            // No meaningful history when comparing against a different item's amount.
            self.prev_amount = self.amount;
        }
        #[cfg(debug_assertions)]
        if self.amount != self.prev_amount {
            let name = String::from_utf16_lossy(item_utf16);
            let verb = if self.amount > self.prev_amount {
                "gained"
            } else {
                "lost"
            };
            asr::print_message(&format!(
                "Collectable \"{}\" {}: {} -> {}",
                name, verb, self.prev_amount, self.amount
            ));
        }
        self.amount
    }

    /// True on the tick the amount owned for `item_utf16` goes up (e.g. picking up a
    /// Collectable), false otherwise - mirrors `Pair::increased()` for a value that isn't
    /// otherwise tracked via `Store`'s generic `i32s` watcher map.
    pub fn increased(&mut self, item_utf16: &'static [u16], e: &Env) -> bool {
        self.get_amount(item_utf16, e);
        self.amount > self.prev_amount
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
    // DEBUG-ONLY: last-seen (name -> amount) for every Collectables entry, so any change can be
    // logged regardless of which Split (if any) is currently active. See `log_collectable_changes`.
    #[cfg(debug_assertions)]
    collectables_log_version: Option<i32>,
    #[cfg(debug_assertions)]
    collectables_log_snapshot: BTreeMap<String, i32>,
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
            #[cfg(debug_assertions)]
            collectables_log_version: None,
            #[cfg(debug_assertions)]
            collectables_log_snapshot: BTreeMap::new(),
        }
    }

    // DEBUG-ONLY: on a Collectables version bump, scans every entry and prints one line per
    // item whose amount changed since the last scan - independent of any active Split.
    #[cfg(debug_assertions)]
    fn log_collectable_changes(&mut self, e: Option<&Env>) {
        let Some(Env { pd, mem, .. }) = e else {
            return;
        };
        let new_version: Option<i32> = mem.deref(&pd.collectables_version).ok();
        if self.collectables_log_version == new_version {
            return;
        }
        self.collectables_log_version = new_version;

        let Some(entries) = scan_all_collectables(mem, pd) else {
            return;
        };

        for (name, amount) in &entries {
            let prev = self
                .collectables_log_snapshot
                .get(name)
                .copied()
                .unwrap_or(0);
            if amount != &prev {
                let verb = if *amount > prev { "gained" } else { "lost" };
                asr::print_message(&format!(
                    "Collectable \"{}\" {}: {} -> {}",
                    name, verb, prev, amount
                ));
            }
        }

        self.collectables_log_snapshot = entries.into_iter().collect();
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

    pub fn get_collectable_amount(&mut self, item_utf16: &'static [u16], e: &Env) -> i32 {
        self.collectables.get_amount(item_utf16, e)
    }

    pub fn has_collectable(&mut self, item_utf16: &'static [u16], e: &Env) -> bool {
        self.collectables.get_amount(item_utf16, e) > 0
    }

    pub fn collectable_increased(&mut self, item_utf16: &'static [u16], e: &Env) -> bool {
        self.collectables.increased(item_utf16, e)
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
        #[cfg(debug_assertions)]
        self.log_collectable_changes(env);
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
