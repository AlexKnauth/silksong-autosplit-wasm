#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod silksong_memory;
pub mod splits;
mod timer;
mod unstable;

use alloc::{boxed::Box, format, vec::Vec};
use asr::{
    future::{next_tick, retry},
    settings::Gui,
    timer::TimerState,
    Address64, Process,
};
use ugly_widget::{
    radio_button::options_str,
    store::{StoreGui, StoreWidget},
    ugly_list::UglyList,
};

use crate::{
    silksong_memory::{
        attach_silksong, GameManagerPointers, Memory, PlayerDataPointers, SceneStore,
        GAME_STATE_CUTSCENE, GAME_STATE_ENTERING_LEVEL, GAME_STATE_EXITING_LEVEL,
        GAME_STATE_INACTIVE, GAME_STATE_LOADING, GAME_STATE_MAIN_MENU, GAME_STATE_PLAYING,
        HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL, MENU_TITLE, QUIT_TO_MENU, UI_STATE_CUTSCENE,
        UI_STATE_PAUSED, UI_STATE_PLAYING,
    },
    timer::SplitterAction,
};

asr::async_main!(stable);
asr::panic_handler!();

// --------------------------------------------------------

struct AutoSplitterState {
    timer_state: TimerState,
    split_index: Option<u64>,
    look_for_teleporting: bool,
    #[cfg(debug_assertions)]
    last_ui_state: i32,
    last_game_state: i32,
    #[cfg(debug_assertions)]
    last_hero_transition_state: i32,
    hits: i64,
    last_recoil: bool,
    last_hazard: bool,
    last_health_0: bool,
    #[cfg(debug_assertions)]
    last_health: Option<i32>,
    #[cfg(debug_assertions)]
    last_paused: bool,
    #[cfg(debug_assertions)]
    last_reasons: Vec<&'static str>,
}

impl AutoSplitterState {
    fn new() -> AutoSplitterState {
        let timer_state = asr::timer::state();
        let split_index = unstable::timer_current_split_index();
        AutoSplitterState {
            timer_state,
            split_index,
            look_for_teleporting: false,
            #[cfg(debug_assertions)]
            last_ui_state: 0,
            last_game_state: GAME_STATE_INACTIVE,
            #[cfg(debug_assertions)]
            last_hero_transition_state: 0,
            hits: 0,
            last_recoil: false,
            last_hazard: false,
            last_health_0: false,
            #[cfg(debug_assertions)]
            last_health: None,
            #[cfg(debug_assertions)]
            last_paused: false,
            #[cfg(debug_assertions)]
            last_reasons: Vec::new(),
        }
    }

    fn update(&mut self) {
        let new_state = asr::timer::state();
        let new_index = unstable::timer_current_split_index();
        if new_state == self.timer_state && new_index == self.split_index {
            return;
        }

        match new_state {
            TimerState::NotRunning
                if self.timer_state == TimerState::Running
                    || self.timer_state == TimerState::Paused
                    || self.timer_state == TimerState::Ended =>
            {
                // Reset
                #[cfg(not(feature = "unstable"))]
                {
                    self.split_index = None;
                }
                self.hits = 0;
                asr::timer::set_variable_int("hits", self.hits);
                self.look_for_teleporting = false;
                self.last_game_state = GAME_STATE_INACTIVE;
                #[cfg(debug_assertions)]
                {
                    self.last_paused = false;
                    self.last_reasons = Vec::new();
                }
            }
            TimerState::Running if is_timer_state_between_runs(self.timer_state) => {
                // Start
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
                #[cfg(not(feature = "unstable"))]
                {
                    self.split_index = Some(self.split_index.unwrap_or_default() + 1);
                }
            }
            _ => {
                #[cfg(feature = "unstable")]
                if let (Some(new_index), Some(old_index)) = (&new_index, &self.split_index) {
                    if new_index < old_index {
                        // Undo
                    } else if new_index > old_index {
                        if unstable::timer_segment_splitted(new_index - 1).unwrap_or_default() {
                            // Split
                        } else {
                            // Skip
                        }
                    }
                }
            }
        }

        self.timer_state = new_state;
        #[cfg(feature = "unstable")]
        {
            self.split_index = new_index;
        }
    }
}

// --------------------------------------------------------

const TICKS_PER_GUI: usize = 0x100;

#[derive(Gui)]
struct Settings {
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
}

fn default_splits_init() -> asr::settings::Map {
    let settings1 = asr::settings::Map::load();
    if settings1
        .get("splits")
        .is_some_and(|v| v.get_list().is_some_and(|l| !l.is_empty()))
    {
        asr::print_message("Settings from asr::settings::Map::load");
        return settings1;
    }
    let l = asr::settings::List::new();
    l.push(options_str(&splits::Split::StartNewGame));
    l.push(options_str(&splits::Split::EndingSplit));
    loop {
        let old = asr::settings::Map::load();
        let new = old.clone();
        new.insert("splits", &l);
        if new.store_if_unchanged(&old) {
            asr::print_message("No settings found: default splits initialized");
            return new;
        }
    }
}

// --------------------------------------------------------

async fn main() {
    // register the variables on start
    asr::timer::set_variable_int("hits", 0);

    asr::print_message("Hello, World!");

    let mut ticks_since_gui = 0;
    let mut settings = Settings::default_init_register();
    asr::print_message(&format!("hit_counter: {:?}", settings.get_hit_counter()));
    asr::print_message(&format!("splits: {:?}", settings.get_splits()));

    let mut state = AutoSplitterState::new();

    loop {
        // TODO: replace this placeholder with the actual executables
        // for each operating system / platform once the game releases.
        let process = wait_attach_silksong(&mut settings, &mut state).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let mut scene_store = Box::new(SceneStore::new());
                let mem = Memory::wait_attach(&process).await;
                let gm = Box::new(GameManagerPointers::new());
                let pd = Box::new(PlayerDataPointers::new());
                let _: bool = mem.deref(&gm.accepting_input).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.entry_gate_name).unwrap_or_default();
                let _: i32 = mem.deref(&gm.game_state).unwrap_or_default();
                let _: bool = mem.deref(&gm.hazard_death).unwrap_or_default();
                let _: bool = mem.deref(&gm.hazard_respawning).unwrap_or_default();
                let _: bool = mem.deref(&gm.hero_recoil_frozen).unwrap_or_default();
                let _: i32 = mem.deref(&gm.hero_transition_state).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.next_scene_name).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.scene_name).unwrap_or_default();
                let _: i32 = mem.deref(&gm.ui_state_vanilla).unwrap_or_default();
                let _: i32 = mem.deref(&pd.health).unwrap_or_default();
                next_tick().await;
                loop {
                    ticks_since_gui += 1;
                    if TICKS_PER_GUI <= ticks_since_gui {
                        settings.load_update_store_if_unchanged();
                        ticks_since_gui = 0;
                    }
                    state.update();

                    // TODO: Do something on every tick.
                    handle_splits(&settings, &mut state, &mem, &gm, &pd, &mut scene_store).await;
                    load_removal(&mut state, &mem, &gm);
                    handle_hits(&settings, &mut state, &mem, &gm, &pd);
                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_attach_silksong(gui: &mut Settings, state: &mut AutoSplitterState) -> Process {
    retry(|| {
        gui.load_update_store_if_unchanged();
        state.update();
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
                        asr::timer::reset();
                        state.timer_state = TimerState::NotRunning;
                        state.split_index = None;
                        state.hits = 0;
                        asr::timer::set_variable_int("hits", state.hits);
                        state.look_for_teleporting = false;
                        state.last_game_state = GAME_STATE_INACTIVE;
                        #[cfg(debug_assertions)]
                        {
                            state.last_paused = false;
                            state.last_reasons = Vec::new();
                        }
                        // no break, allow other actions after a skip or reset
                    }
                    SplitterAction::Skip => {
                        asr::timer::skip_split();
                        state.split_index = Some(state.split_index.unwrap_or_default() + 1);
                        // no break, allow other actions after a skip or reset
                    }
                    SplitterAction::Split => {
                        asr::timer::split();
                        state.split_index = Some(state.split_index.unwrap_or_default() + 1);
                        break;
                    }
                    SplitterAction::ManualSplit => {
                        #[cfg(not(feature = "unstable"))]
                        {
                            state.split_index = Some(state.split_index.unwrap_or_default() + 1);
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
        asr::timer::pause_game_time();
        return;
    }

    let ui_state: i32 = mem.deref(&gm.ui_state_vanilla).unwrap_or_default();
    let scene_name = mem.read_string(&gm.scene_name).unwrap_or_default();
    let next_scene = mem.read_string(&gm.next_scene_name).unwrap_or_default();

    let loading_menu = (scene_name != MENU_TITLE && next_scene.is_empty())
        || (scene_name != MENU_TITLE && next_scene == MENU_TITLE || (scene_name == QUIT_TO_MENU));

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

    // TODO: hazard_respawning
    let accepting_input: bool = mem.deref(&gm.accepting_input).unwrap_or_default();
    let hero_transition_state: i32 = mem.deref(&gm.hero_transition_state).unwrap_or_default();
    // TODO: tile_map_dirty, uses_scene_transition_routine

    let is_game_time_paused = (state.look_for_teleporting)
        || ((game_state == GAME_STATE_PLAYING || game_state == GAME_STATE_ENTERING_LEVEL)
            && ui_state != UI_STATE_PLAYING)
        || (game_state != GAME_STATE_PLAYING
            && game_state != GAME_STATE_CUTSCENE
            && !accepting_input)
        || (game_state == GAME_STATE_EXITING_LEVEL || game_state == GAME_STATE_LOADING)
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
        let mut reasons = Vec::new();
        if state.look_for_teleporting {
            reasons.push("look_for_teleporting");
        }
        if (game_state == GAME_STATE_PLAYING || game_state == GAME_STATE_ENTERING_LEVEL)
            && ui_state != UI_STATE_PLAYING
        {
            reasons.push("ui_state");
        }
        if game_state != GAME_STATE_PLAYING && game_state != GAME_STATE_CUTSCENE && !accepting_input
        {
            reasons.push("accepting_input");
        }
        if game_state == GAME_STATE_EXITING_LEVEL || game_state == GAME_STATE_LOADING {
            reasons.push("game_state");
        }
        if hero_transition_state == HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL {
            reasons.push("hero_transition_state");
        }
        if ui_state != UI_STATE_PLAYING && (loading_menu) && next_scene != scene_name {
            reasons.push("next_scene_loading_menu");
        }
        if ui_state != UI_STATE_PLAYING
            && (ui_state != UI_STATE_PAUSED
                && ui_state != UI_STATE_CUTSCENE
                && (!next_scene.is_empty()))
            && next_scene != scene_name
        {
            reasons.push("next_scene_ui_state");
        }
        if reasons != state.last_reasons {
            asr::print_message(&format!("reasons: {:?}", reasons));
        }
        state.last_reasons = reasons;
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
}

// --------------------------------------------------------

pub fn is_timer_state_between_runs(s: TimerState) -> bool {
    s == TimerState::NotRunning || s == TimerState::Ended
}

pub fn str_take_right(s: &str, n: usize) -> &str {
    s.split_at(s.len().saturating_sub(n)).1
}
