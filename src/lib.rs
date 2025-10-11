#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod silksong_memory;
pub mod splits;
mod timer;
mod unstable;

use alloc::{boxed::Box, format, string::String, vec::Vec};
use asr::{
    future::{next_tick, retry},
    settings::{
        gui::{Title, Widget},
        Gui,
    },
    timer::TimerState,
    Address64, Process,
};
use core::cmp;
use ugly_widget::{
    radio_button::{options_normalize, options_str},
    store::{StoreGui, StoreWidget},
    ugly_list::UglyList,
};

use crate::{
    silksong_memory::{
        attach_silksong, GameManagerPointers, Memory, PlayerDataPointers, SceneStore,
        GAME_STATE_CUTSCENE, GAME_STATE_ENTERING_LEVEL, GAME_STATE_EXITING_LEVEL,
        GAME_STATE_INACTIVE, GAME_STATE_LOADING, GAME_STATE_MAIN_MENU, GAME_STATE_PLAYING,
        HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL, MENU_TITLE, NON_MENU_GAME_STATES,
        OPENING_SCENES, QUIT_TO_MENU, UI_STATE_CUTSCENE, UI_STATE_PAUSED, UI_STATE_PLAYING,
    },
    timer::SplitterAction,
};

asr::async_main!(stable);
asr::panic_handler!();

// --------------------------------------------------------

static MODULE_PATH: &str = module_path!();

fn this_script_name() -> &'static str {
    MODULE_PATH
        .split_once("::")
        .map_or(MODULE_PATH, |(base, _)| base)
}

static CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The dash symbol to use for generic dashes in text.
pub const DASH: &str = "—";
/// The minus symbol to use for negative numbers.
const MINUS: &str = "−";
/// The plus symbol to use for positive numbers.
const PLUS: &str = "+";

// --------------------------------------------------------

struct AutoSplitterState {
    /// The timer state.
    timer_state: TimerState,
    /// The last observed asr::timer::state.
    /// Just in case asr::timer::state is a tad out-of-date.
    last_timer_state: TimerState,
    /// The split index.
    /// None: NotRunning
    /// Some: Running, Paused, or Ended
    split_index: Option<u64>,
    /// The last observed split index.
    /// Just in case asr::timer::current_split_index is a tad out-of-date.
    last_split_index: Option<u64>,
    segments_splitted: Vec<bool>,
    look_for_teleporting: bool,
    #[cfg(debug_assertions)]
    last_ui_state: i32,
    last_game_state: i32,
    #[cfg(debug_assertions)]
    last_hero_transition_state: i32,
    hits: i64,
    segment_hits: Vec<i64>,
    cumulative_hits: Vec<i64>,
    comparison_hits: Vec<i64>,
    last_recoil: bool,
    last_hazard: bool,
    last_health_0: bool,
    mms_room_dupe: bool,
    #[cfg(debug_assertions)]
    last_health: Option<i32>,
    #[cfg(debug_assertions)]
    last_paused: bool,
}

impl AutoSplitterState {
    fn new() -> AutoSplitterState {
        let timer_state = asr::timer::state();
        let split_index = unstable::timer_current_split_index();
        let mut segments_splitted = Vec::new();
        segments_splitted.resize(split_index.unwrap_or_default() as usize, false);
        let comparison_hits = Settings::get_comparison_hits().unwrap_or_default();
        AutoSplitterState {
            timer_state,
            last_timer_state: timer_state,
            split_index,
            last_split_index: split_index,
            segments_splitted,
            look_for_teleporting: false,
            #[cfg(debug_assertions)]
            last_ui_state: 0,
            last_game_state: GAME_STATE_INACTIVE,
            #[cfg(debug_assertions)]
            last_hero_transition_state: 0,
            hits: 0,
            segment_hits: Vec::new(),
            cumulative_hits: Vec::new(),
            comparison_hits,
            last_recoil: false,
            last_hazard: false,
            last_health_0: false,
            mms_room_dupe: false,
            #[cfg(debug_assertions)]
            last_health: None,
            #[cfg(debug_assertions)]
            last_paused: false,
        }
    }

    fn update(&mut self, settings: &Settings) {
        let new_state = asr::timer::state();
        let new_index = unstable::timer_current_split_index();
        if (new_state == self.timer_state && new_index == self.split_index)
            || (new_state == self.last_timer_state && new_index == self.last_split_index)
        {
            self.last_timer_state = new_state;
            self.last_split_index = new_index;
            return;
        }

        match new_state {
            TimerState::NotRunning
                if self.timer_state == TimerState::Running
                    || self.timer_state == TimerState::Paused
                    || self.timer_state == TimerState::Ended =>
            {
                // Reset
                if settings.get_hit_counter() {
                    Settings::update_comparison_hits(
                        &mut self.comparison_hits,
                        &self.cumulative_hits,
                    );
                    if self.timer_state == TimerState::Ended {
                        if let Some(pb_hits) = self.comparison_hits.last() {
                            asr::timer::set_variable_int("pb hits", *pb_hits);
                        }
                    }
                }
                #[cfg(not(feature = "split-index"))]
                {
                    self.split_index = None;
                }
                self.segments_splitted.clear();
                self.hits = 0;
                self.segment_hits.clear();
                self.cumulative_hits.clear();
                if settings.get_hit_counter() {
                    asr::timer::set_variable_int("hits", self.hits);
                    asr::timer::set_variable_int("segment hits", 0);
                } else {
                    asr::timer::set_variable("hits", DASH);
                    asr::timer::set_variable("segment hits", DASH);
                }
                asr::timer::set_variable("comparison hits", DASH);
                asr::timer::set_variable("delta hits", DASH);
                asr::timer::set_variable("percent", DASH);
                self.look_for_teleporting = false;
                self.last_game_state = GAME_STATE_INACTIVE;
                #[cfg(debug_assertions)]
                {
                    self.last_paused = false;
                }
            }
            TimerState::Running if is_timer_state_between_runs(self.timer_state) => {
                // Start
                let new_i = new_index.unwrap_or_default() as usize;
                self.segment_hits.resize(new_i + 1, 0);
                if settings.get_hit_counter() {
                    asr::timer::set_variable_int("segment hits", self.segment_hits[new_i]);
                    if let Some(c) = self.comparison_hits.get(new_i) {
                        asr::timer::set_variable_int("comparison hits", *c);
                        asr::timer::set_variable("delta hits", &delta_string(self.hits - c));
                    } else {
                        asr::timer::set_variable("comparison hits", DASH);
                        asr::timer::set_variable("delta hits", DASH);
                    }
                }
            }
            TimerState::Paused if self.timer_state == TimerState::Running => {
                // Pause
            }
            TimerState::Running if self.timer_state == TimerState::Paused => {
                // Resume
            }
            TimerState::Ended
                if self.timer_state == TimerState::Running
                    || self.timer_state == TimerState::Paused =>
            {
                // End
                #[cfg(not(feature = "split-index"))]
                {
                    // TODO: use last_split_index here but also other places on legacy / not-split-index
                    self.split_index = Some(self.split_index.unwrap_or_default() + 1);
                }
                #[cfg(feature = "split-index")]
                match new_index {
                    Some(new_idx) if self.last_split_index.unwrap_or_default() < new_idx => {
                        self.split_index = Some(new_idx)
                    }
                    _ => {
                        self.split_index = Some(self.last_split_index.unwrap_or_default() + 1);
                    }
                }
                if settings.get_hit_counter() {
                    if let Some(index) = self.split_index {
                        let i = index as usize;
                        self.cumulative_hits.resize(i, self.hits);
                        let cmp_len = self.comparison_hits.len();
                        if i < cmp_len {
                            self.comparison_hits.drain(0..(cmp_len - i));
                        }
                    }
                }
            }
            _ => {
                #[cfg(feature = "split-index")]
                if let (Some(new_index), Some(old_index)) = (&new_index, &self.split_index) {
                    let new_i = *new_index as usize;
                    if new_index < old_index {
                        // Undo
                        self.segment_hits[new_i] +=
                            self.segment_hits.drain((new_i + 1)..).sum::<i64>();
                        if new_i < self.cumulative_hits.len() {
                            let mut i = new_i;
                            // go back through skipped splits
                            while 1 <= i && !self.segments_splitted[i - 1] {
                                i -= 1;
                            }
                            // segment [i - 1] was not skipped, but segment [i] was skipped or undone,
                            // so remove cumulative_hits from there on
                            self.cumulative_hits.truncate(i);
                        }
                        self.segments_splitted.truncate(new_i);
                    } else if new_index > old_index {
                        for old_idx in (*old_index)..(*new_index) {
                            let o_i = old_idx as usize;
                            let n_i = o_i + 1;
                            let splitted =
                                unstable::timer_segment_splitted(old_idx).unwrap_or_default();
                            self.segments_splitted.push(splitted);
                            if splitted {
                                // Split
                                self.segment_hits.push(0);
                                self.cumulative_hits.resize(n_i, self.hits);
                            } else {
                                // Skip
                                self.segment_hits.insert(o_i, 0);
                            }
                        }
                    }

                    if settings.get_hit_counter() && new_index != old_index {
                        asr::timer::set_variable_int("segment hits", self.segment_hits[new_i]);
                        if let Some(c) = self.comparison_hits.get(new_i) {
                            asr::timer::set_variable_int("comparison hits", *c);
                            asr::timer::set_variable("delta hits", &delta_string(self.hits - c));
                        } else {
                            asr::timer::set_variable("comparison hits", DASH);
                            asr::timer::set_variable("delta hits", DASH);
                        }
                    }
                }
            }
        }

        self.timer_state = new_state;
        self.last_timer_state = new_state;
        self.last_split_index = new_index;
        #[cfg(feature = "split-index")]
        {
            self.split_index = new_index;
        }
    }
}

// --------------------------------------------------------

const TICKS_PER_GUI: usize = 0x100;

struct PkgVersion(Title);

impl Widget for PkgVersion {
    type Args = <Title as Widget>::Args;

    fn register(key: &str, _: &str, args: Self::Args) -> Self {
        let desc = format!("{} {}", this_script_name(), CARGO_PKG_VERSION);
        Self(Title::register(key, &desc, args))
    }

    fn update_from(&mut self, settings_map: &asr::settings::Map, key: &str, args: Self::Args) {
        self.0.update_from(settings_map, key, args);
    }
}

#[derive(Gui)]
struct Settings {
    _pkg_version: PkgVersion,
    /// Hit Counter
    #[default = true]
    hit_counter: bool,
    /// Splits
    #[heading_level = 1]
    splits: UglyList<splits::Split>,
}

impl StoreGui for Settings {
    fn insert_into(&self, settings_map: &asr::settings::Map) -> bool {
        let a = self.hit_counter.insert_into(settings_map, "hit_counter");
        let b = self.splits.insert_into(settings_map, "splits");
        a || b
    }
}

impl Settings {
    pub fn get_hit_counter(&self) -> bool {
        self.hit_counter
    }
    pub fn get_splits_len(&self) -> usize {
        self.splits.get_list().len()
    }
    pub fn get_splits(&self) -> Vec<splits::Split> {
        self.splits.get_list().into_iter().cloned().collect()
    }
    pub fn get_split(&self, i: u64) -> Option<splits::Split> {
        self.splits.get_list().get(i as usize).cloned().cloned()
    }

    pub fn default_init_register() -> Settings {
        default_splits_init();
        let mut gui = Settings::register();
        gui.loop_load_update_store();
        gui
    }

    pub fn get_comparison_hits() -> Option<Vec<i64>> {
        let c = asr::settings::Map::load().get("comparison_hits")?;
        Some(c.get_list()?.iter().filter_map(|i| i.get_i64()).collect())
    }

    pub fn update_comparison_hits(comparison_hits: &mut Vec<i64>, cumulative_hits: &[i64]) {
        // save cumulative_hits to comparison_hits
        for i in 0..cumulative_hits.len() {
            if i < comparison_hits.len() {
                comparison_hits[i] = cmp::min(comparison_hits[i], cumulative_hits[i]);
            } else {
                comparison_hits.push(cumulative_hits[i]);
            }
        }
        Settings::set_comparison_hits(comparison_hits);
    }

    fn set_comparison_hits(comparison_hits: &[i64]) {
        let l = asr::settings::List::new();
        for i in comparison_hits {
            l.push(*i);
        }
        loop {
            let old = asr::settings::Map::load();
            let new = old.clone();
            new.insert("comparison_hits", &l);
            if new.store_if_unchanged(&old) {
                return;
            }
        }
    }
}

fn default_splits_init() -> asr::settings::Map {
    let settings1 = asr::settings::Map::load();
    let this_script = this_script_name();
    if let Some(script_name) = settings1.get("script_name") {
        if script_name.get_string().unwrap_or_default() != this_script {
            asr::print_message(&format!(
                "error: settings for wrong script_name: {:?} vs {}",
                script_name, this_script
            ));
            panic!(
                "error: settings for wrong script_name: {:?} vs {}",
                script_name, this_script
            );
        }
    }
    if settings1
        .get("splits")
        .is_some_and(|v| v.get_list().is_some_and(|l| !l.is_empty()))
    {
        asr::print_message("Settings from asr::settings::Map::load");
        if asr_settings_normalize(&settings1).is_some() {
            asr::print_message("Settings normalized");
            settings1.store();
        }
        return settings1;
    }
    let l = asr::settings::List::new();
    l.push(options_str(&splits::Split::StartNewGame));
    l.push(options_str(&splits::Split::EndingSplit));
    loop {
        let old = asr::settings::Map::load();
        let new = old.clone();
        new.insert("script_name", this_script);
        new.insert("splits", &l);
        if new.store_if_unchanged(&old) {
            asr::print_message("No settings found: default splits initialized");
            return new;
        }
    }
}

fn asr_settings_normalize(m: &asr::settings::Map) -> Option<()> {
    let old_splits = m.get("splits")?.get_list()?;
    let new_splits = asr::settings::List::new();
    let mut changed = false;
    let this_script = this_script_name();
    if !m
        .get("script_name")
        .is_some_and(|v| v.get_string().unwrap_or_default() == this_script)
    {
        changed = true;
        m.insert("script_name", this_script);
    }
    for (i, old_split) in old_splits.iter().enumerate() {
        let old_string = old_split.get_string()?;
        let new_string = options_normalize::<splits::Split>(&old_string);
        new_splits.push(new_string.as_str());
        if old_string != new_string {
            changed = true;
            m.insert(&format!("splits_{}_item", i), new_string.as_str());
        }
    }
    if changed {
        m.insert("splits", new_splits);
        Some(())
    } else {
        None
    }
}

// --------------------------------------------------------

async fn main() {
    // register the variables on start
    asr::timer::set_variable("hits", DASH);
    asr::timer::set_variable("segment hits", DASH);
    asr::timer::set_variable("pb hits", DASH);
    asr::timer::set_variable("comparison hits", DASH);
    asr::timer::set_variable("delta hits", DASH);
    asr::timer::set_variable("percent", DASH);

    asr::print_message("Hello, World!");

    let mut ticks_since_gui = 0;
    let mut settings = Settings::default_init_register();
    asr::print_message(&format!("hit_counter: {:?}", settings.get_hit_counter()));
    asr::print_message(&format!("splits: {:?}", settings.get_splits()));

    let mut state = AutoSplitterState::new();

    if settings.get_hit_counter() {
        asr::timer::set_variable_int("hits", 0);
        asr::timer::set_variable_int("segment hits", 0);
    }

    if !state.comparison_hits.is_empty()
        && (state.comparison_hits.len() + 1 == settings.get_splits_len())
    {
        if let Some(pb_hits) = state.comparison_hits.last() {
            asr::timer::set_variable_int("pb hits", *pb_hits);
        }
    }

    loop {
        // TODO: replace this placeholder with the actual executables
        // for each operating system / platform once the game releases.
        let process = wait_attach_silksong(&mut settings, &mut state).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let mut scene_store = Box::new(SceneStore::new());
                next_tick().await;
                let mem = Memory::wait_attach(&process).await;
                next_tick().await;
                let gm = Box::new(GameManagerPointers::new());
                let pd = Box::new(PlayerDataPointers::new());
                let _: bool = mem.deref(&gm.accepting_input).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.entry_gate_name).unwrap_or_default();
                let _: i32 = mem.deref(&gm.game_state).unwrap_or_default();
                let _: bool = mem.deref(&gm.hazard_death).unwrap_or_default();
                let _: bool = mem.deref(&gm.hazard_respawning).unwrap_or_default();
                let _: bool = mem.deref(&gm.hero_recoil_frozen).unwrap_or_default();
                let _: i32 = mem.deref(&gm.hero_transition_state).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.scene_load).unwrap_or_default();
                let _: bool = mem
                    .deref(&gm.scene_load_activation_allowed)
                    .unwrap_or_default();
                let _: Address64 = mem.deref(&gm.next_scene_name).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.scene_name).unwrap_or_default();
                let _: i32 = mem.deref(&gm.ui_state_vanilla).unwrap_or_default();
                let _: i32 = mem.deref(&pd.health).unwrap_or_default();
                next_tick().await;
                asr::print_message("Initialized load removal pointers");
                next_tick().await;
                loop {
                    ticks_since_gui += 1;
                    if TICKS_PER_GUI <= ticks_since_gui
                        || (is_timer_state_between_runs(state.timer_state)
                            && scene_store.pair().current == MENU_TITLE)
                    {
                        settings.load_update_store_if_unchanged();
                        ticks_since_gui = 0;
                    }
                    state.update(&settings);

                    // TODO: Do something on every tick.
                    handle_splits(&settings, &mut state, &mem, &gm, &pd, &mut scene_store).await;
                    load_removal(&mut state, &mem, &gm);
                    handle_hits(&settings, &mut state, &mem, &gm, &pd);
                    handle_percent(&mem, &gm, &pd);
                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_attach_silksong(gui: &mut Settings, state: &mut AutoSplitterState) -> Process {
    retry(|| {
        gui.load_update_store_if_unchanged();
        state.update(gui);
        attach_silksong()
    })
    .await
}

// --------------------------------------------------------

async fn handle_splits(
    settings: &Settings,
    state: &mut AutoSplitterState,
    mem: &Memory<'_>,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
    ss: &mut SceneStore,
) {
    let trans_now = ss.transition_now(mem, gm);
    loop {
        match state.timer_state {
            TimerState::NotRunning => {
                // TODO: look up from settings
                let Some(split) = settings.get_split(0) else {
                    break;
                };
                let a = splits::splits(&split, mem, gm, pd, trans_now, ss);
                match a {
                    SplitterAction::Split => {
                        asr::timer::start();
                        state.timer_state = TimerState::Running;
                        state.split_index = Some(0);
                        state.segment_hits.resize(1, 0);
                        if settings.get_hit_counter() {
                            asr::timer::set_variable_int("segment hits", state.segment_hits[0]);
                            // .first() = .get(0)
                            if let Some(c) = state.comparison_hits.first() {
                                asr::timer::set_variable_int("comparison hits", *c);
                                asr::timer::set_variable(
                                    "delta hits",
                                    &delta_string(state.hits - c),
                                );
                            } else {
                                asr::timer::set_variable("comparison hits", DASH);
                                asr::timer::set_variable("delta hits", DASH);
                            }
                        }
                        break;
                    }
                    _ => break,
                }
            }
            TimerState::Running | TimerState::Paused => {
                // TODO: look up from settings
                let Some(split) = settings.get_split(state.split_index.unwrap_or_default() + 1)
                else {
                    break;
                };
                let a = splits::splits(&split, mem, gm, pd, trans_now, ss);
                match a {
                    SplitterAction::Reset => {
                        if settings.get_hit_counter() {
                            Settings::update_comparison_hits(
                                &mut state.comparison_hits,
                                &state.cumulative_hits,
                            );
                        }
                        asr::timer::reset();
                        state.timer_state = TimerState::NotRunning;
                        state.split_index = None;
                        state.segments_splitted.clear();
                        state.hits = 0;
                        state.segment_hits.clear();
                        state.cumulative_hits.clear();
                        if settings.get_hit_counter() {
                            asr::timer::set_variable_int("hits", state.hits);
                            asr::timer::set_variable_int("segment hits", 0);
                        } else {
                            asr::timer::set_variable("hits", DASH);
                            asr::timer::set_variable("segment hits", DASH);
                        }
                        asr::timer::set_variable("comparison hits", DASH);
                        asr::timer::set_variable("delta hits", DASH);
                        asr::timer::set_variable("percent", DASH);
                        state.look_for_teleporting = false;
                        state.last_game_state = GAME_STATE_INACTIVE;
                        #[cfg(debug_assertions)]
                        {
                            state.last_paused = false;
                        }
                        // no break, allow other actions after a skip or reset
                    }
                    SplitterAction::Skip => {
                        let old_index = state.split_index.unwrap_or_default();
                        let old_i = old_index as usize;
                        asr::timer::skip_split();
                        let new_i = old_i + 1;
                        state.split_index = Some(old_index + 1);
                        state.segments_splitted.push(false);
                        state.segment_hits.insert(old_i, 0);
                        if settings.get_hit_counter() {
                            asr::timer::set_variable_int("segment hits", state.segment_hits[new_i]);
                            if let Some(c) = state.comparison_hits.get(new_i) {
                                asr::timer::set_variable_int("comparison hits", *c);
                                asr::timer::set_variable(
                                    "delta hits",
                                    &delta_string(state.hits - c),
                                );
                            } else {
                                asr::timer::set_variable("comparison hits", DASH);
                                asr::timer::set_variable("delta hits", DASH);
                            }
                        }
                        // no break, allow other actions after a skip or reset
                    }
                    SplitterAction::Split => {
                        let old_index = state.split_index.unwrap_or_default();
                        asr::timer::split();
                        let new_i = old_index as usize + 1;
                        state.split_index = Some(old_index + 1);
                        state.segments_splitted.push(true);
                        state.segment_hits.push(0);
                        state.cumulative_hits.resize(new_i, state.hits);
                        if settings.get_hit_counter() {
                            asr::timer::set_variable_int("segment hits", state.segment_hits[new_i]);
                            if let Some(c) = state.comparison_hits.get(new_i) {
                                asr::timer::set_variable_int("comparison hits", *c);
                                asr::timer::set_variable(
                                    "delta hits",
                                    &delta_string(state.hits - c),
                                );
                            } else {
                                asr::timer::set_variable("comparison hits", DASH);
                                asr::timer::set_variable("delta hits", DASH);
                            }
                        }
                        break;
                    }
                    SplitterAction::ManualSplit => {
                        #[cfg(not(feature = "split-index"))]
                        {
                            let old_index = state.split_index.unwrap_or_default();
                            let old_i = old_index as usize;
                            let new_i = old_i + 1;
                            state.split_index = Some(old_index + 1);
                            state.segments_splitted.push(false);
                            state.segment_hits.insert(old_i, 0);
                            if settings.get_hit_counter() {
                                asr::timer::set_variable_int(
                                    "segment hits",
                                    state.segment_hits[new_i],
                                );
                                if let Some(c) = state.comparison_hits.get(new_i) {
                                    asr::timer::set_variable_int("comparison hits", *c);
                                    asr::timer::set_variable(
                                        "delta hits",
                                        &delta_string(state.hits - c),
                                    );
                                } else {
                                    asr::timer::set_variable("comparison hits", DASH);
                                    asr::timer::set_variable("delta hits", DASH);
                                }
                            }
                        }
                        break;
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }
}

fn load_removal(state: &mut AutoSplitterState, mem: &Memory, gm: &GameManagerPointers) {
    // only remove loads if timer is running
    if asr::timer::state() != TimerState::Running {
        return;
    }

    let ui_state: i32 = mem.deref(&gm.ui_state_vanilla).unwrap_or_default();
    let scene_name = mem.read_string(&gm.scene_name).unwrap_or_default();
    let next_scene = mem.read_string(&gm.next_scene_name).unwrap_or_default();

    let loading_menu = (scene_name != MENU_TITLE && next_scene.is_empty())
        || (scene_name != MENU_TITLE && next_scene == MENU_TITLE)
        || (scene_name == QUIT_TO_MENU);

    // TODO: teleporting, look_for_teleporting

    let game_state: i32 = mem.deref(&gm.game_state).unwrap_or_default();

    if game_state == GAME_STATE_PLAYING && state.last_game_state == GAME_STATE_MAIN_MENU {
        state.look_for_teleporting = true;
    }
    if state.look_for_teleporting
        && (game_state != GAME_STATE_PLAYING && game_state != GAME_STATE_ENTERING_LEVEL)
    {
        state.look_for_teleporting = false;
    }

    if game_state == GAME_STATE_LOADING
        && state.last_game_state == GAME_STATE_CUTSCENE
        && OPENING_SCENES.contains(&scene_name.as_str())
    {
        #[cfg(debug_assertions)]
        if !state.mms_room_dupe {
            asr::print_message("mms_room_dupe: true");
        }
        state.mms_room_dupe = true;
    } else if game_state == GAME_STATE_PLAYING {
        #[cfg(debug_assertions)]
        if state.mms_room_dupe {
            asr::print_message("mms_room_dupe: false");
        }
        state.mms_room_dupe = false;
    }

    // TODO: hazard_respawning
    let accepting_input: bool = mem.deref(&gm.accepting_input).unwrap_or_default();
    let hero_transition_state: i32 = mem.deref(&gm.hero_transition_state).unwrap_or_default();
    let scene_load_null: bool = mem
        .deref(&gm.scene_load)
        .is_ok_and(|a: Address64| a.is_null());
    let scene_load_activation_allowed: bool = mem
        .deref(&gm.scene_load_activation_allowed)
        .unwrap_or_default();
    // TODO: tile_map_dirty, uses_scene_transition_routine

    let is_game_time_paused = (state.look_for_teleporting)
        || ((game_state == GAME_STATE_PLAYING || game_state == GAME_STATE_ENTERING_LEVEL)
            && ui_state != UI_STATE_PLAYING)
        || (game_state != GAME_STATE_PLAYING
            && game_state != GAME_STATE_CUTSCENE
            && !accepting_input
            && !state.mms_room_dupe)
        || ((game_state == GAME_STATE_EXITING_LEVEL
            && (scene_load_null || scene_load_activation_allowed)
            && !state.mms_room_dupe)
            || game_state == GAME_STATE_LOADING)
        || (hero_transition_state == HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL)
        || (ui_state != UI_STATE_PLAYING
            && (loading_menu
                || (ui_state != UI_STATE_PAUSED
                    && ui_state != UI_STATE_CUTSCENE
                    && (!next_scene.is_empty())))
            && next_scene != scene_name);
    if is_game_time_paused {
        asr::timer::pause_game_time();
    } else {
        asr::timer::resume_game_time();
    }

    #[cfg(debug_assertions)]
    {
        if ui_state != state.last_ui_state {
            asr::print_message(&format!("ui_state: {}", ui_state));
        }
        state.last_ui_state = ui_state;
    }

    #[cfg(debug_assertions)]
    if game_state != state.last_game_state {
        asr::print_message(&format!("game_state: {}", game_state));
    }
    state.last_game_state = game_state;

    #[cfg(debug_assertions)]
    {
        if hero_transition_state != state.last_hero_transition_state {
            asr::print_message(&format!("hero_transition_state: {}", hero_transition_state));
        }
        state.last_hero_transition_state = hero_transition_state;
    }

    #[cfg(debug_assertions)]
    {
        if is_game_time_paused != state.last_paused {
            asr::print_message(&format!("is_game_time_paused: {}", is_game_time_paused));
        }
        state.last_paused = is_game_time_paused;
    }
}

fn handle_hits(
    settings: &Settings,
    state: &mut AutoSplitterState,
    mem: &Memory,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
) {
    // only count hits if hit counter is true
    if !settings.get_hit_counter() {
        return;
    }
    // only count hits if timer is running
    if asr::timer::state() != TimerState::Running {
        return;
    }

    let recoil: bool = mem.deref(&gm.hero_recoil_frozen).unwrap_or_default();
    if !state.last_recoil && recoil {
        add_hit(state);
        #[cfg(debug_assertions)]
        asr::print_message(&format!("hit: {}, from recoil", state.hits));
    }
    state.last_recoil = recoil;

    let hazard: bool = mem.deref(&gm.hazard_death).unwrap_or_default();
    if !state.last_hazard && hazard {
        add_hit(state);
        #[cfg(debug_assertions)]
        asr::print_message(&format!("hit: {}, from hazard", state.hits));
    }
    state.last_hazard = hazard;

    let maybe_health: Option<i32> = mem.deref(&pd.health).ok();
    let game_state: i32 = mem.deref(&gm.game_state).unwrap_or_default();
    let health_0 = maybe_health == Some(0) && game_state == GAME_STATE_PLAYING;
    if !state.last_health_0 && health_0 {
        add_hit(state);
        #[cfg(debug_assertions)]
        asr::print_message(&format!("hit: {}, from heath 0", state.hits));
    }
    state.last_health_0 = health_0;

    #[cfg(debug_assertions)]
    {
        if maybe_health != state.last_health {
            asr::print_message(&format!("health: {:?}", maybe_health));
        }
        state.last_health = maybe_health;
    }
}

fn add_hit(state: &mut AutoSplitterState) {
    state.hits += 1;
    asr::timer::set_variable_int("hits", state.hits);
    let i = state.split_index.unwrap_or_default() as usize;
    state.segment_hits.resize(i + 1, 0);
    state.segment_hits[i] += 1;
    asr::timer::set_variable_int("segment hits", state.segment_hits[i]);
    if let Some(c) = state.comparison_hits.get(i) {
        asr::timer::set_variable("delta hits", &delta_string(state.hits - c));
    } else {
        asr::timer::set_variable("delta hits", DASH);
    }
}

fn handle_percent(mem: &Memory, gm: &GameManagerPointers, pd: &PlayerDataPointers) {
    // only update percent if timer is running or paused
    if is_timer_state_between_runs(asr::timer::state()) {
        return;
    }

    // only update percent if game state is non-menu
    let game_state: i32 = mem.deref(&gm.game_state).unwrap_or_default();
    if !NON_MENU_GAME_STATES.contains(&game_state) {
        return;
    }

    let Ok(percent) = mem.deref::<f32, _>(&pd.completion_percentage) else {
        return;
    };

    asr::timer::set_variable("percent", &format!("{}%", percent));
}

// --------------------------------------------------------

pub fn is_timer_state_between_runs(s: TimerState) -> bool {
    s == TimerState::NotRunning || s == TimerState::Ended
}

pub fn str_take_right(s: &str, n: usize) -> &str {
    s.split_at(s.len().saturating_sub(n)).1
}

fn delta_string(i: i64) -> String {
    if i.is_positive() {
        format!("{}{}", PLUS, i)
    } else if i.is_negative() {
        format!("{}{}", MINUS, -i)
    } else {
        format!("{}", i)
    }
}
