use core::mem;

#[cfg(debug_assertions)]
use alloc::format;
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use asr::{
    future::next_tick,
    game_engine::unity::mono::{self, UnityPointer},
    watcher::Pair,
    Address64, Process,
};
use bytemuck::CheckedBitPattern;

// --------------------------------------------------------

static SILKSONG_NAMES: [&str; 2] = [
    "Hollow Knight Silksong.exe", // Windows
    "Hollow Knight Silksong",     // Mac, Linux
];

// const PRE_MENU_INTRO: &str = "Pre_Menu_Intro";
pub const MENU_TITLE: &str = "Menu_Title";
pub const QUIT_TO_MENU: &str = "Quit_To_Menu";

pub const OPENING_SEQUENCE: &str = "Opening_Sequence";
pub static OPENING_SCENES: [&str; 1] = [OPENING_SEQUENCE];

// static NON_PLAY_SCENES: [&str; 4] = [PRE_MENU_INTRO, MENU_TITLE, QUIT_TO_MENU, OPENING_SEQUENCE];

static BAD_SCENE_NAMES: [&str; 11] = [
    "Untagged",
    "left1",
    "oncomplete",
    "Attack Range",
    "onstart",
    "position",
    "looptype",
    "integer1",
    "gameObject",
    "eventTarget",
    "material",
];

pub const GAME_STATE_INACTIVE: i32 = 0;
pub const GAME_STATE_MAIN_MENU: i32 = 1;
pub const GAME_STATE_LOADING: i32 = 2;
pub const GAME_STATE_ENTERING_LEVEL: i32 = 3;
pub const GAME_STATE_PLAYING: i32 = 4;
// pub const GAME_STATE_PAUSED: i32 = 5;
pub const GAME_STATE_EXITING_LEVEL: i32 = 6;
pub const GAME_STATE_CUTSCENE: i32 = 7;

pub static NON_MENU_GAME_STATES: [i32; 2] = [GAME_STATE_PLAYING, GAME_STATE_CUTSCENE];

// UI_STATE 1: Main Menu
pub const UI_STATE_PLAYING: i32 = 4;
pub const UI_STATE_PAUSED: i32 = 5;

// HERO_TRANSITION_STATE 0: N/A, not in transition
// HERO_TRANSITION_STATE 1: Exiting scene
// HERO_TRANSITION_STATE 2, 3: Waiting to enter, Entering?
pub const HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL: i32 = 2;

pub struct StringListOffsets {
    string_len: u64,
    string_contents: u64,
    /*
    list_array: u64,
    array_len: u64,
    array_contents: u64,
    */
}

impl StringListOffsets {
    fn new() -> StringListOffsets {
        StringListOffsets {
            string_len: 0x10,
            string_contents: 0x14,
            /*
            list_array: 0x10,
            array_len: 0x18,
            array_contents: 0x20,
            */
        }
    }
}

// --------------------------------------------------------

pub fn attach_silksong() -> Option<Process> {
    SILKSONG_NAMES.into_iter().find_map(Process::attach)
}

pub fn is_menu(s: &str) -> bool {
    s.is_empty() || s == MENU_TITLE || s == QUIT_TO_MENU // || s == PERMA_DEATH
}

// --------------------------------------------------------

macro_rules! declare_pointers {
    ( $g:ident { $( $f:ident : $t:ty = $e:expr ),*, } ) => {
        pub struct $g {
            $( pub $f : $t ),*,
        }

        impl $g {
            pub fn new() -> $g {
                $g {
                    $( $f : $e ),*,
                }
            }
        }
    };
}

declare_pointers!(GameManagerPointers {
    scene_name: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "sceneName"]),
    next_scene_name: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "nextSceneName"]),
    entry_gate_name: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "entryGateName"]),
    game_state: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "<GameState>k__BackingField"]),
    ui_state_vanilla: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &["_instance", "<ui>k__BackingField", "uiState"],
    ),
    accepting_input: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<inputHandler>k__BackingField",
            "acceptingInput",
        ],
    ),
    hazard_death: UnityPointer<4> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<hero_ctrl>k__BackingField",
            "cState",
            "hazardDeath",
        ],
    ),
    hazard_respawning: UnityPointer<4> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<hero_ctrl>k__BackingField",
            "cState",
            "hazardRespawning",
        ],
    ),
    hero_recoil_frozen: UnityPointer<4> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<hero_ctrl>k__BackingField",
            "cState",
            "recoilFrozen",
        ],
    ),
    hero_transition_state: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &["_instance", "<hero_ctrl>k__BackingField", "transitionState"],
    ),
});

declare_pointers!(PlayerDataPointers {
    health: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "health"]),
    defeated_moss_mother: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedMossMother"]),
    has_needle_throw: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasNeedleThrow"]),
    defeated_bell_beast: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedBellBeast"]),
    has_dash: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDash"]),
    defeated_lace1: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedLace1"]),
    has_brolly: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasBrolly"]),
    defeated_song_golem: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedSongGolem"]),
    defeated_vampire_gnat_boss: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedVampireGnatBoss"]),
    has_wall_jump: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasWalljump"]),
    spinner_defeated: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "spinnerDefeated"]),
    defeated_phantom: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedPhantom"]),
    act2_started: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "act2Started"]),
    defeated_cogwork_dancers: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedCogworkDancers"]),
    defeated_trobbio: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedTrobbio"]),
    has_harpoon_dash: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasHarpoonDash"]),
    hang04_battle: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hang04Battle"]),
    defeated_lace_tower: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedLaceTower"]),
    has_melody_librarian: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "HasMelodyLibrarian"]),
    has_melody_conductor: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "HasMelodyConductor"]),
    has_melody_architect: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "HasMelodyArchitect"]),
    unlocked_melody_lift: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedMelodyLift"]),
    nail_upgrades: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nailUpgrades"]),
});

// --------------------------------------------------------

pub struct Memory<'a> {
    pub process: &'a Process,
    pub module: Box<mono::Module>,
    pub image: mono::Image,
    pub string_list_offsets: Box<StringListOffsets>,
}

impl Memory<'_> {
    pub async fn wait_attach<'a>(process: &'a Process) -> Memory<'a> {
        asr::print_message("Memory wait_attach: Module wait_attach...");
        next_tick().await;
        let mut found_module = false;
        let mut needed_retry = false;
        loop {
            let module = mono::Module::wait_attach(&process, mono::Version::V3).await;
            if !found_module {
                found_module = true;
                asr::print_message("Memory wait_attach: module get_default_image...");
                next_tick().await;
            }
            for _ in 0..0x10 {
                if let Some(image) = module.get_default_image(&process) {
                    asr::print_message("Memory wait_attach: got module and image");
                    next_tick().await;
                    return Memory {
                        process,
                        module: Box::new(module),
                        image,
                        string_list_offsets: Box::new(StringListOffsets::new()),
                    };
                }
                next_tick().await;
            }
            if !needed_retry {
                needed_retry = true;
                asr::print_message("Memory wait_attach: retry...");
                next_tick().await;
            }
        }
    }

    pub fn deref<T: CheckedBitPattern, const CAP: usize>(
        &self,
        p: &UnityPointer<CAP>,
    ) -> Result<T, asr::Error> {
        p.deref(&self.process, &self.module, &self.image)
    }

    pub fn read_string<const CAP: usize>(&self, p: &UnityPointer<CAP>) -> Option<String> {
        let a: Address64 = self.deref(p).ok()?;
        let n: u32 = self
            .process
            .read(a + self.string_list_offsets.string_len)
            .ok()?;
        if !(n < 2048) {
            return None;
        }
        let w: Vec<u16> = self
            .process
            .read_vec(a + self.string_list_offsets.string_contents, n as usize)
            .ok()?;
        String::from_utf16(&w).ok()
    }
}

// --------------------------------------------------------

pub struct SceneStore {
    prev_scene_name: String,
    curr_scene_name: String,
    next_scene_name: String,
    new_data_curr: bool,
    new_data_next: bool,
    last_next: bool,
    pub split_this_transition: bool,
}

impl SceneStore {
    pub fn new() -> SceneStore {
        SceneStore {
            prev_scene_name: "".to_string(),
            curr_scene_name: "".to_string(),
            next_scene_name: "".to_string(),
            new_data_curr: false,
            new_data_next: false,
            last_next: true,
            split_this_transition: false,
        }
    }

    pub fn pair(&self) -> Pair<&str> {
        if self.last_next && self.next_scene_name != self.curr_scene_name {
            Pair {
                old: &self.curr_scene_name,
                current: &self.next_scene_name,
            }
        } else {
            Pair {
                old: &self.prev_scene_name,
                current: &self.curr_scene_name,
            }
        }
    }

    pub fn new_curr_scene_name(&mut self, csn: String) {
        if !csn.is_empty()
            && csn != self.curr_scene_name
            && !BAD_SCENE_NAMES.contains(&csn.as_str())
        {
            self.prev_scene_name = mem::replace(&mut self.curr_scene_name, csn);
            #[cfg(debug_assertions)]
            asr::print_message(&format!("curr_scene_name: {}", self.curr_scene_name));
            self.new_data_curr = self.curr_scene_name != self.next_scene_name;
        }
    }

    pub fn new_next_scene_name(&mut self, nsn: String) {
        if !nsn.is_empty()
            && nsn != self.next_scene_name
            && !BAD_SCENE_NAMES.contains(&nsn.as_str())
        {
            self.next_scene_name = nsn;
            #[cfg(debug_assertions)]
            asr::print_message(&format!("next_scene_name: {}", self.next_scene_name));
            self.new_data_next = !self.next_scene_name.is_empty();
        }
    }

    pub fn transition_now(&mut self, mem: &Memory, gm: &GameManagerPointers) -> bool {
        self.new_curr_scene_name(mem.read_string(&gm.scene_name).unwrap_or_default());
        self.new_next_scene_name(mem.read_string(&gm.next_scene_name).unwrap_or_default());

        if self.new_data_next {
            self.new_data_curr = false;
            self.new_data_next = false;
            self.last_next = true;
            self.split_this_transition = false;
            #[cfg(debug_assertions)]
            asr::print_message(&format!(
                "curr {} -> next {}",
                &self.curr_scene_name, &self.next_scene_name
            ));
            true
        } else if self.new_data_curr {
            self.new_data_curr = false;
            if is_menu(&self.next_scene_name)
                && !is_menu(&self.prev_scene_name)
                && !is_menu(&self.curr_scene_name)
            {
                #[cfg(debug_assertions)]
                asr::print_message(&format!(
                    "IGNORING spurious curr {} during next {}",
                    self.curr_scene_name, self.next_scene_name
                ));
                return false;
            }
            self.last_next = false;
            self.split_this_transition = false;
            #[cfg(debug_assertions)]
            asr::print_message(&format!(
                "prev {} -> curr {}",
                &self.prev_scene_name, &self.curr_scene_name
            ));
            true
        } else {
            false
        }
    }
}

// --------------------------------------------------------
