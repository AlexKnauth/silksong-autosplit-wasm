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

pub static DEATH_RESPAWN_MARKER_INIT: &str = "Death Respawn Marker Init";

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
pub const UI_STATE_CUTSCENE: i32 = 3;
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

        impl Default for $g {
            fn default() -> Self {
                Self::new()
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
    scene_load_activation_allowed: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &["_instance", "sceneLoad", "<IsActivationAllowed>k__BackingField"],
    ),
});

declare_pointers!(PlayerDataPointers {
    disable_pause: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &["_instance", "playerData", "disablePause"],
    ),
    health: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "health"]),
    defeated_moss_mother: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedMossMother"]),
    has_needle_throw: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasNeedleThrow"]),
    defeated_bell_beast: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedBellBeast"]),
    bell_shrine_bone_forest: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bellShrineBoneForest"]),
    has_dash: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDash"]),
    defeated_lace1: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedLace1"]),
    bell_shrine_wilds: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bellShrineWilds"]),
    has_brolly: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasBrolly"]),
    defeated_song_golem: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedSongGolem"]),
    bell_shrine_greymoor: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bellShrineGreymoor"]),
    defeated_vampire_gnat_boss: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedVampireGnatBoss"]),
    has_wall_jump: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasWalljump"]),
    spinner_defeated: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "spinnerDefeated"]),
    bell_shrine_bellhart: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bellShrineBellhart"]),
    bell_shrine_shellwood: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bellShrineShellwood"]),
    defeated_last_judge: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedLastJudge"]),
    defeated_phantom: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedPhantom"]),
    act2_started: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "act2Started"]),
    defeated_cogwork_dancers: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedCogworkDancers"]),
    completed_library_entry_battle: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "completedLibraryEntryBattle"]),
    defeated_trobbio: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedTrobbio"]),
    has_harpoon_dash: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasHarpoonDash"]),
    hang04_battle: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hang04Battle"]),
    defeated_lace_tower: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedLaceTower"]),
    has_melody_librarian: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "HasMelodyLibrarian"]),
    has_melody_conductor: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "HasMelodyConductor"]),
    has_melody_architect: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "HasMelodyArchitect"]),
    unlocked_melody_lift: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedMelodyLift"]),
    nail_upgrades: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "nailUpgrades"]),
    completed_memory_reaper: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "completedMemory_reaper"]),

    savedflea_ant_03: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Ant_03"]),
    savedflea_belltown_04: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Belltown_04"]),
    savedflea_bone_06: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Bone_06"]),
    savedflea_bone_east_05: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Bone_East_05"]),
    savedflea_bone_east_10_church: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Bone_East_10_Church"]),
    savedflea_bone_east_17b: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Bone_East_17b"]),
    savedflea_coral_24: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Coral_24"]),
    savedflea_coral_35: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Coral_35"]),
    savedflea_crawl_06: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Crawl_06"]),
    savedflea_dock_03d: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Dock_03d"]),
    savedflea_dock_16: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Dock_16"]),
    savedflea_dust_09: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Dust_09"]),
    savedflea_dust_12: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Dust_12"]),
    savedflea_greymoor_06: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Greymoor_06"]),
    savedflea_greymoor_15b: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Greymoor_15b"]),
    savedflea_library_01: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Library_01"]),
    savedflea_library_09: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Library_09"]),
    savedflea_peak_05c: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Peak_05c"]),
    savedflea_shadow_10: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Shadow_10"]),
    savedflea_shadow_28: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Shadow_28"]),
    savedflea_shellwood_03: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Shellwood_03"]),
    savedflea_slab_06: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Slab_06"]),
    savedflea_slab_cell: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Slab_Cell"]),
    savedflea_song_11: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Song_11"]),
    savedflea_song_14: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Song_14"]),
    savedflea_under_21: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Under_21"]),
    savedflea_under_23: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SavedFlea_Under_23"]),
    tamed_giant_flea: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "tamedGiantFlea"]),
    met_troupe_hunter_wild: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "MetTroupeHunterWild"]),
    caravan_lech_saved: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CaravanLechSaved"]),

    unlocked_aqueduct_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedAqueductStation"]),
    unlocked_belltown_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedBelltownStation"]),
    unlocked_boneforest_east_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedBoneforestEastStation"]),
    unlocked_city_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedCityStation"]),
    unlocked_coral_tower_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedCoralTowerStation"]),
    unlocked_docks_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedDocksStation"]),
    unlocked_greymoor_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedGreymoorStation"]),
    unlocked_peak_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedPeakStation"]),
    unlocked_shadow_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedShadowStation"]),
    unlocked_shellwood_station: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedShellwoodStation"]),

    unlocked_song_tube: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedSongTube"]),
    unlocked_under_tube: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedUnderTube"]),
    unlocked_city_bellway_tube: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedCityBellwayTube"]),
    unlocked_hang_tube: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedHangTube"]),
    unlocked_enclave_tube: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedEnclaveTube"]),
    unlocked_arborium_tube: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedArboriumTube"]),

    seen_mapper_bonetown: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperBonetown"]),
    seen_mapper_bone_forest: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperBoneForest"]),
    seen_mapper_docks: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperDocks"]),
    seen_mapper_wilds: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperWilds"]),
    seen_mapper_crawl: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperCrawl"]),
    seen_mapper_greymoor: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperGreymoor"]),
    seen_mapper_bellhart: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperBellhart"]),
    seen_mapper_shellwood: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperShellwood"]),
    seen_mapper_hunters_nest: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperHuntersNest"]),
    seen_mapper_judge_steps: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperJudgeSteps"]),
    seen_mapper_dustpens: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperDustpens"]),
    seen_mapper_peak: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperPeak"]),
    seen_mapper_shadow: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperShadow"]),
    seen_mapper_coral_caverns: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenMapperCoralCaverns"]),

    met_city_merchant_enclave: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "MetCityMerchantEnclave"]),
    met_sherma_enclave: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "MetShermaEnclave"]),
    unlocked_dust_cage: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "UnlockedDustCage"]),
    green_prince_location: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "GreenPrinceLocation"]),
    seen_fleatopia_empty: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "SeenFleatopiaEmpty"]),
    has_double_jump: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasDoubleJump"]),
    has_super_jump: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "hasSuperJump"]),
    collected_heart_flower: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CollectedHeartFlower"]),
    collected_heart_coral: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CollectedHeartCoral"]),
    collected_heart_hunter: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CollectedHeartHunter"]),
    collected_heart_clover: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CollectedHeartClover"]),
    completed_red_memory: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CompletedRedMemory"]),
    belltown_greeter_house_full_dlg: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "BelltownGreeterHouseFullDlg"]),
    clover_memory_orbs_collected_target: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CloverMemoryOrbsCollectedTarget"]),
    defeated_dock_foremen: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedDockForemen"]),
    defeated_swamp_shaman: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "DefeatedSwampShaman"]),
    defeated_bone_flyer_giant: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedBoneFlyerGiant"]),
    defeated_bone_flyer_giant_golem_scene: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedBoneFlyerGiantGolemScene"]),
    caravan_troupe_location: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CaravanTroupeLocation"]),
    belltown_relic_dealer_gave_relic: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "BelltownRelicDealerGaveRelic"]),
    collected_ward_key: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CollectedWardKey"]),
    belltown_greeter_met_time_passed: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "BelltownGreeterMetTimepassed"]),
    bell_shrine_enclave: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "bellShrineEnclave"]),
    defeated_zap_core_enemy: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedZapCoreEnemy"]),
    skull_king_defeated: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "skullKingDefeated"]),
    sherma_healer_active: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "ShermaHealerActive"]),
    city_merchant_saved: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "cityMerchantSaved"]),
    enclave_merchant_saved: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "enclaveMerchantSaved"]),
    caretaker_offered_snare_quest: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CaretakerOfferedSnareQuest"]),
    soul_snare_ready: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "soulSnareReady"]),
    defeated_seth: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedSeth"]),
    completed_abyss_ascent: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "CompletedAbyssAscent"]),
    ballow_moved_to_diving_bell: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "BallowMovedToDivingBell"]),
    black_thread_world: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "blackThreadWorld"]),
    defeated_coral_drillers: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "defeatedCoralDrillers"]),

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
            let module = mono::Module::wait_attach(process, mono::Version::V3).await;
            if !found_module {
                found_module = true;
                asr::print_message("Memory wait_attach: module get_default_image...");
                next_tick().await;
            }
            for _ in 0..0x10 {
                if let Some(image) = module.get_default_image(process) {
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
        p.deref(self.process, &self.module, &self.image)
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
        if mem
            .deref(&gm.scene_load_activation_allowed)
            .unwrap_or_default()
        {
            self.new_next_scene_name(mem.read_string(&gm.next_scene_name).unwrap_or_default());
        }

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

impl Default for SceneStore {
    fn default() -> Self {
        Self::new()
    }
}

// --------------------------------------------------------
