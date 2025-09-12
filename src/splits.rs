use alloc::{vec, vec::Vec};
use asr::{settings::Gui, watcher::Pair};
use ugly_widget::{
    radio_button::{options_str, RadioButtonOptions},
    store::StoreWidget,
};

use crate::{
    silksong_memory::{
        is_menu, GameManagerPointers, Memory, PlayerDataPointers, SceneStore, MENU_TITLE,
        NON_MENU_GAME_STATES, OPENING_SCENES,
    },
    timer::{should_split, SplitterAction},
};

#[derive(Clone, Debug, Default, Eq, Gui, Ord, PartialEq, PartialOrd, RadioButtonOptions)]
pub enum Split {
    // region: Start, End, and Menu
    /// Manual Split (Misc)
    ///
    /// Never splits. Use this when you need to manually split
    #[default]
    ManualSplit,
    /// Start New Game (Start)
    ///
    /// Splits when starting a new save file
    StartNewGame,
    /// Credits Roll (Ending)
    ///
    /// Splits on any credits rolling, any ending
    EndingSplit,
    /// Main Menu (Menu)
    ///
    /// Splits on the main menu
    Menu,
    /// Death (Event)
    ///
    /// Splits when player HP is 0
    PlayerDeath,
    /// Any Transition (Transition)
    ///
    /// Splits when entering a transition (only one will split per transition)
    AnyTransition,
    // endregion: Start, End, and Menu

    // region: MossLands
    /// Moss Mother (Boss)
    ///
    /// Splits when killing Moss Mother
    MossMother,
    /// Silk Spear (Skill)
    ///
    /// Splits when obtaining Silk Spear
    SilkSpear,
    // endregion: MossLands

    // region: Marrow
    /// Bell Beast (Boss)
    ///
    /// Splits when defeating the Bell Beast
    BellBeast,
    // endregion: Marrow

    // region: DeepDocks
    /// Swift Step (Skill)
    ///
    /// Splits when obtaining Swift Step (Dash/Sprint)
    SwiftStep,
    /// Lace 1 (Boss)
    ///
    /// Splits when defeating Lace 1 in DeepDocks
    Lace1,
    // endregion: DeepDocks

    // region: FarFields
    /// Drifter's Cloak (Skill)
    ///
    /// Splits when obtaining Drifter's Cloak (Umbrella/Float)
    DriftersCloak,
    /// Fourth Chorus (Boss)
    ///
    /// Splits when killing Fourth Chorus
    FourthChorus,
    // endregion: FarFields

    // region: Greymoor
    /// Float to Greymoor (Transition)
    ///
    /// Splits when entering Greymoor from Far Fields
    FloatToGreymoor,
    /// Greymoor 04 Gauntlet (Gauntlet)
    ///
    /// Splits when completing Greymoor_04 Gauntlet
    Greymoor04Gauntlet,
    /// Moorwing (Boss)
    ///
    /// Splits when killing Moorwing
    Moorwing,
    // endregion: Greymoor

    // region: Shellwood
    /// Cling Grip (Skill)
    ///
    /// Splits when obtaining Cling Grip (Wall Jump)
    ClingGrip,
    // endregion: Shellwood

    // region: Bellhart
    /// Widow (Boss)
    ///
    /// Splits when killing Widow
    Widow,
    // endregion: Bellhart

    // region: SinnersRoad
    /// Enter Mist From Sinners Road (Transition)
    ///
    /// Splits when entering The Mist from Sinners Road
    EnterMistFromSinnersRoad,
    /// Leave Mist From Sinners Road (Transition)
    ///
    /// Splits when leaving The Mist from Sinners Road
    LeaveMistFromSinnersRoad,
    // endregion: SinnersRoad

    // region: Bilewater
    /// Phantom (Boss)
    ///
    /// Splits when killing Phantom
    Phantom,
    // endregion: Bilewater
}

impl StoreWidget for Split {
    fn insert_into(&self, settings_map: &asr::settings::Map, key: &str) -> bool {
        let new_s = options_str(self);
        if settings_map
            .get(key)
            .is_some_and(|old_v| old_v.get_string().is_some_and(|old_s| old_s == new_s))
        {
            return false;
        }
        settings_map.insert(key, new_s);
        true
    }
}

pub fn transition_splits(
    split: &Split,
    scenes: &Pair<&str>,
    _mem: &Memory,
    _gm: &GameManagerPointers,
    _pd: &PlayerDataPointers,
) -> SplitterAction {
    match split {
        // region: Start, End, and Menu
        Split::StartNewGame => {
            should_split(OPENING_SCENES.contains(&scenes.old) && scenes.current == "Tut_01")
        }
        Split::EndingSplit => should_split(scenes.current.starts_with("Cinematic_Ending")),
        Split::Menu => should_split(scenes.current == MENU_TITLE),
        Split::AnyTransition => should_split(
            scenes.current != scenes.old && !(is_menu(scenes.old) || is_menu(scenes.current)),
        ),
        // endregion: Start, End, and Menu

        // region: Greymoor
        Split::FloatToGreymoor => should_split(scenes.old == "Bone_East_11" && scenes.current == "Greymoor_01"),
        // region: Greymoor

        // region: SinnersRoad
        Split::EnterMistFromSinnersRoad => should_split(scenes.old == "Dust_05" && scenes.current == "Dust_Maze_09_entrance"),
        Split::LeaveMistFromSinnersRoad => should_split(scenes.old == "Dust_Maze_Last_Hall" && scenes.current == "Dust_09"),
        // region: SinnersRoad

        // else
        _ => should_split(false),
    }
}

pub fn continuous_splits(
    split: &Split,
    mem: &Memory,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
) -> SplitterAction {
    let game_state: i32 = mem.deref(&gm.game_state).unwrap_or_default();
    if !NON_MENU_GAME_STATES.contains(&game_state) {
        return should_split(false);
    }
    match split {
        // region: Start, End, and Menu
        Split::ManualSplit => SplitterAction::ManualSplit,
        Split::PlayerDeath => should_split(mem.deref(&pd.health).is_ok_and(|h: i32| h == 0)),
        // endregion: Start, End, and Menu

        // region: MossLands
        Split::MossMother => should_split(mem.deref(&pd.defeated_moss_mother).unwrap_or_default()),
        Split::SilkSpear => should_split(mem.deref(&pd.has_needle_throw).unwrap_or_default()),
        // endregion: MossLands

        // region: Marrow
        Split::BellBeast => should_split(mem.deref(&pd.defeated_bell_beast).unwrap_or_default()),
        // endregion: Marrow

        // region: DeepDocks
        Split::SwiftStep => should_split(mem.deref(&pd.has_dash).unwrap_or_default()),
        Split::Lace1 => should_split(mem.deref(&pd.defeated_lace1).unwrap_or_default()),
        // endregion: DeepDocks

        // region: FarFields
        Split::DriftersCloak => should_split(mem.deref(&pd.has_brolly).unwrap_or_default()),
        Split::FourthChorus => should_split(mem.deref(&pd.defeated_song_golem).unwrap_or_default()),
        // endregion: FarFields

        // region: Greymoor
        Split::Greymoor04Gauntlet => should_split(mem.deref(&pd.greymoor_04_battle_completed).unwrap_or_default()),
        Split::Moorwing => should_split(mem.deref(&pd.defeated_vampire_gnat_boss).unwrap_or_default()),
        // endregion: Greymoor

        // region: Shellwood
        Split::ClingGrip => should_split(mem.deref(&pd.has_wall_jump).unwrap_or_default()),
        // endregion: Shellwood

        // region: Bellhart
        Split::Widow => should_split(mem.deref(&pd.spinner_defeated).unwrap_or_default()),
        // endregion: Bellhart

        // region: Bilewater
        Split::Phantom => should_split(mem.deref(&pd.defeated_phantom).unwrap_or_default()),
        // endregion: Bilewater

        // else
        _ => should_split(false),
    }
}

pub fn splits(
    split: &Split,
    mem: &Memory,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
    trans_now: bool,
    ss: &mut SceneStore,
) -> SplitterAction {
    let a1 = continuous_splits(split, mem, gm, pd).or_else(|| {
        let scenes = ss.pair();
        if trans_now {
            transition_splits(split, &scenes, mem, gm, pd)
        } else {
            SplitterAction::Pass
        }
    });
    if a1 != SplitterAction::Pass {
        ss.split_this_transition = true;
    }
    a1
}
