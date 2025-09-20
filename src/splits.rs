use alloc::{vec, vec::Vec};
use asr::{settings::Gui, watcher::Pair};
use ugly_widget::{
    radio_button::{options_str, RadioButtonOptions},
    store::StoreWidget,
};

use crate::{
    silksong_memory::{
        is_menu, GameManagerPointers, Memory, PlayerDataPointers, GameplayPointers, SceneStore, MENU_TITLE,
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
    /// Weaver Queen (Ending)
    ///
    /// Splits on Weaver Queen ending
    EndingA,
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
    /// Moss Mother (Transition)
    ///
    /// Splits on the transition after killing Moss Mother
    MossMotherTrans,
    /// Silk Spear (Skill)
    ///
    /// Splits when obtaining Silk Spear
    SilkSpear,
    /// Silk Spear (Transition)
    ///
    /// Splits on the transition after obtaining Silk Spear
    SilkSpearTrans,
    // endregion: MossLands

    // region: Marrow
    /// Bell Beast (Boss)
    ///
    /// Splits when defeating the Bell Beast
    BellBeast,
    /// Bell Beast (Transition)
    ///
    /// Splits on the transition after defeating the Bell Beast
    BellBeastTrans,
    /// Marrow Bell (Event)
    ///
    /// Splits when ringing the Marrow Bell Shrine
    MarrowBell,
    // endregion: Marrow

    // region: DeepDocks
    /// Swift Step (Skill)
    ///
    /// Splits when obtaining Swift Step (Dash/Sprint)
    SwiftStep,
    /// Swift Step (Transition)
    ///
    /// Splits on the transition after obtaining Swift Step (Dash/Sprint)
    SwiftStepTrans,
    /// Lace 1 (Boss)
    ///
    /// Splits when defeating Lace 1 in DeepDocks
    Lace1,
    /// Lace 1 (Transition)
    ///
    /// Splits on the transition after defeating Lace 1 in DeepDocks
    Lace1Trans,
    /// Deep Docks Bell (Event)
    ///
    /// Splits when ringing the Deep Docks Bell Shrine
    DeepDocksBell,
    // endregion: DeepDocks

    // region: FarFields
    /// Drifter's Cloak (Skill)
    ///
    /// Splits when obtaining Drifter's Cloak (Umbrella/Float)
    DriftersCloak,
    /// Drifter's Cloak (Transition)
    ///
    /// Splits on the transition after obtaining Drifter's Cloak (Umbrella/Float)
    DriftersCloakTrans,
    /// Fourth Chorus (Boss)
    ///
    /// Splits when killing Fourth Chorus
    FourthChorus,
    // endregion: FarFields

    // region: Greymoor
    /// Enter Greymoor (Transition)
    ///
    /// Splits when entering Greymoor
    EnterGreymoor,
    /// Greymoor Bell (Event)
    ///
    /// Splits when ringing the Greymoor Bell Shrine
    GreymoorBell,
    /// Moorwing (Boss)
    ///
    /// Splits when killing Moorwing
    Moorwing,
    /// Moorwing (Transition)
    ///
    /// Splits on the transition after killing Moorwing
    MoorwingTrans,
    // endregion: Greymoor

    // region: Shellwood
    /// Cling Grip (Skill)
    ///
    /// Splits when obtaining Cling Grip (Wall Jump)
    ClingGrip,
    /// Cling Grip (Transition)
    ///
    /// Splits on the transition after obtaining Cling Grip (Wall Jump)
    ClingGripTrans,
    /// Shellwood Bell (Event)
    ///
    /// Splits when ringing the Shellwood Bell Shrine
    ShellwoodBell,
    // endregion: Shellwood

    // region: Bellhart
    /// Widow (Boss)
    ///
    /// Splits when killing Widow
    Widow,
    /// Bellhart Bell (Event)
    ///
    /// Splits when ringing the Bellhart Bell Shrine
    BellhartBell,
    // endregion: Bellhart

    // region: BlastedSteps
    /// Last Judge (Boss)
    ///
    /// Splits when killing Last Judge
    LastJudge,
    // endregion: BlastedSteps

    // region: TheMist
    /// Enter The Mist (Transition)
    ///
    /// Splits when entering The Mist
    EnterMist,
    /// Leave The Mist (Transition)
    ///
    /// Splits when leaving The Mist
    LeaveMist,
    // endregion: TheMist

    // region: Bilewater
    /// Phantom (Boss)
    ///
    /// Splits when killing Phantom
    Phantom,
    // endregion: Bilewater

    // region: Acts
    /// Act 2 Started (Event)
    ///
    /// Splits when starting Act 2
    Act2Started,
    // endregion: Acts

    // region: CogworkCore
    /// Cogwork Dancers (Boss)
    ///
    /// Splits when killing Cogwork Dancers
    CogworkDancers,
    // endregion: CogworkCore

    // region: WhisperingVaults
    /// Whispering Vaults Gauntlet (Mini Boss)
    ///
    /// Splits when completing the Whispering Vaults Gauntlet
    WhisperingVaultsGauntlet,
    // endregion: WhisperingVaults

    // region: ChoralChambers
    /// Trobbio (Boss)
    ///
    /// Splits when killing Trobbio
    Trobbio,
    /// Trobbio (Transition)
    ///
    /// Splits on the transition after killing Trobbio
    TrobbioTrans,
    // endregion: ChoralChambers

    // region: Underworks
    /// Clawline (Skill)
    ///
    /// Splits when obtaining Clawline (Harpoon Dash)
    Clawline,
    // endregion: Underworks

    // region: HighHalls
    /// Enter High Halls (Transition)
    ///
    /// Splits when entering High Halls
    EnterHighHalls,
    /// Enter High Halls Gauntlet (Transition)
    ///
    /// Splits when entering the High Halls Gauntlet room
    EnterHighHallsGauntlet,
    /// High Halls Gauntlet (Mini Boss)
    ///
    /// Splits when completing the High Halls Gauntlet
    HighHallsGauntlet,
    // endregion: HighHalls

    // region: TheCradle
    /// Lace 2 (Boss)
    ///
    /// Splits when defeating Lace 2 in TheCradle
    Lace2,
    // endregion: TheCradle

    // region: ThreefoldMelody
    /// Vaultkeepers Melody (Melody)
    ///
    /// Splits when learning Vaultkeepers Melody
    VaultkeepersMelody,
    /// Vaultkeepers Melody (Transition)
    ///
    /// Splits on the transition after learning Vaultkeepers Melody
    VaultkeepersMelodyTrans,
    /// Architects Melody (Melody)
    ///
    /// Splits when learning Architects Melody
    ArchitectsMelody,
    /// Architects Melody (Transition)
    ///
    /// Splits on the transition after learning Architects Melody
    ArchitectsMelodyTrans,
    /// Conductors Melody (Melody)
    ///
    /// Splits when learning Conductors Melody
    ConductorsMelody,
    /// Conductors Melody (Transition)
    ///
    /// Splits on the transition after learning Conductors Melody
    ConductorsMelodyTrans,
    /// Unlock Threefold Melody Lift (Event)
    ///
    /// Splits when unlocking the Threefold Melody Lift
    UnlockedMelodyLift,
    // endregion: ThreefoldMelody

    // region: NeedleUpgrade
    /// Needle 1 (Upgrade)
    ///
    /// Splits when upgrading to Sharpened Needle
    NeedleUpgrade1,
    /// Needle 2 (Upgrade)
    ///
    /// Splits when upgrading to Shining Needle
    NeedleUpgrade2,
    /// Needle 3 (Upgrade)
    ///
    /// Splits when upgrading to Hivesteel Needle
    NeedleUpgrade3,
    /// Needle 4 (Upgrade)
    ///
    /// Splits when upgrading to Pale Steel Needle
    NeedleUpgrade4,
    // endregion: NeedleUpgrade

    //region: FleaSpecific
    /// Flea Hunter's March
    /// 
    /// Splits when rescuing flea in Ant_03
    SavedFleaAnt03,
    /// Flea Bellhart
    /// 
    /// Splits when rescuing flea in Belltown_04
    SavedFleaBelltown04,
    /// Flea Marrow
    /// 
    /// Splits when rescuing flea in Bone_06
    SavedFleaBone06,
    /// Flea Deep Docks Sprint
    /// 
    /// Splits when rescuing flea in Bone_East_05
    SavedFleaBoneEast05,
    /// Flea Far Fields Pilgrim's Rest
    /// 
    /// Splits when rescuing flea in Bone_East_10_Church
    SavedFleaBoneEast10Church,
    /// Flea Far Fields Trap
    /// 
    /// Splits when rescuing flea in Bone_East_17b
    SavedFleaBoneEast17b,
    /// Flea Sands of Karak
    /// 
    /// Splits when rescuing flea in Coral_24
    SavedFleaCoral24,
    /// Flea Blasted Steps
    /// 
    /// Splits when rescuing flea in Coral_35
    SavedFleaCoral35,
    /// Flea Wormways
    /// 
    /// Splits when rescuing flea in Crawl_06
    SavedFleaCrawl06,
    /// Flea Deep Docks Arena
    /// 
    /// Splits when rescuing flea in Dock_03d
    SavedFleaDock03d,
    /// Flea Deep Docks Bellway
    /// 
    /// Splits when rescuing flea in Dock_16
    SavedFleaDock16,
    /// Flea Bilewater Organ
    /// 
    /// Splits when rescuing flea in Dust_09
    SavedFleaDust09,
    /// Flea Sinner's Road
    /// 
    /// Splits when rescuing flea in Dust_12
    SavedFleaDust12,
    /// Flea Greymoor Roof
    /// 
    /// Splits when rescuing flea in Greymoor_06
    SavedFleaGreymoor06,
    /// Flea Greymoor Lake
    /// 
    /// Splits when rescuing flea in Greymoor_15b
    SavedFleaGreymoor15b,
    /// Flea Whispering Vaults
    /// 
    /// Splits when rescuing flea in Library_01
    SavedFleaLibrary01,
    /// Flea Songclave
    /// 
    /// Splits when rescuing flea in Library_09
    SavedFleaLibrary09,
    /// Flea Mount Fay
    /// 
    /// Splits when rescuing flea in Peak_05c
    SavedFleaPeak05c,
    /// Flea Bilewater Trap
    /// 
    /// Splits when rescuing flea in Shadow_10
    SavedFleaShadow10,
    /// Flea Bilewater Thieves
    /// 
    /// Splits when rescuing flea in Shadow_28
    SavedFleaShadow28,
    /// Flea Shellwood
    /// 
    /// Splits when rescuing flea in Shellwood_03
    SavedFleaShellwood03,
    /// Flea Slab Bellway
    /// 
    /// Splits when rescuing flea in Slab_06
    SavedFleaSlab06,
    /// Flea Slab Cage
    /// 
    /// Splits when rescuing flea in Slab_Cell
    SavedFleaSlabCell,
    /// Flea Choral Chambers Wind
    /// 
    /// Splits when rescuing flea in Song_11
    SavedFleaSong11,
    /// Flea Choral Chambers Cage
    /// 
    /// Splits when rescuing flea in Song_14
    SavedFleaSong14,
    /// Flea Underworks Explosions
    /// 
    /// Splits when rescuing flea in Under_21
    SavedFleaUnder21,
    /// Flea Underworks Wisp Thicket
    /// 
    /// Splits when rescuing flea in Under_23
    SavedFleaUnder23,
    /// Flea Giant Tamed
    /// 
    /// Splits when defeating Giant Flea
    TamedGiantFlea,
    /// Flea Vog
    /// 
    /// Splits after talking to Vog
    MetTroupeHunterWild,
    /// Flea Kratt
    /// 
    /// Splits after freeing Kratt
    CaravanLechSaved,
//endregion
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
    mem: &Memory,
    _gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
    _gp: &GameplayPointers,
) -> SplitterAction {
    match split {
        // region: Start, End, and Menu
        Split::StartNewGame => {
            should_split(OPENING_SCENES.contains(&scenes.old) && scenes.current == "Tut_01")
        }
        Split::EndingSplit => should_split(scenes.current.starts_with("Cinematic_Ending")),
        Split::EndingA => should_split(scenes.current == "Cinematic_Ending_A"),
        Split::Menu => should_split(scenes.current == MENU_TITLE),
        Split::AnyTransition => should_split(
            scenes.current != scenes.old && !(is_menu(scenes.old) || is_menu(scenes.current)),
        ),
        // endregion: Start, End, and Menu

        // region: MossLands
        Split::MossMotherTrans => {
            should_split(mem.deref(&pd.defeated_moss_mother).unwrap_or_default())
        }
        Split::SilkSpearTrans => should_split(mem.deref(&pd.has_needle_throw).unwrap_or_default()),
        // endregion: MossLands

        // region: Marrow
        Split::BellBeastTrans => {
            should_split(mem.deref(&pd.defeated_bell_beast).unwrap_or_default())
        }
        // endregion: Marrow

        // region: DeepDocks
        Split::SwiftStepTrans => should_split(mem.deref(&pd.has_dash).unwrap_or_default()),
        Split::Lace1Trans => should_split(mem.deref(&pd.defeated_lace1).unwrap_or_default()),
        // endregion: DeepDocks

        // region: FarFields
        Split::DriftersCloakTrans => should_split(mem.deref(&pd.has_brolly).unwrap_or_default()),
        // endregion: FarFields

        // region: Greymoor
        Split::EnterGreymoor => should_split(
            !scenes.old.starts_with("Greymoor") && scenes.current.starts_with("Greymoor"),
        ),
        Split::MoorwingTrans => should_split(
            mem.deref(&pd.defeated_vampire_gnat_boss)
                .unwrap_or_default(),
        ),
        // endregion: Greymoor

        // region: Shellwood
        Split::ClingGripTrans => should_split(mem.deref(&pd.has_wall_jump).unwrap_or_default()),
        // endregion: Shellwood

        // region: TheMist
        Split::EnterMist => should_split(
            (scenes.old == "Dust_05" || scenes.old == "Shadow_04")
                && scenes.current == "Dust_Maze_09_entrance",
        ),
        Split::LeaveMist => {
            should_split(scenes.old == "Dust_Maze_Last_Hall" && scenes.current == "Dust_09")
        }
        // endregion: TheMist

        // region: ChoralChambers
        Split::TrobbioTrans => should_split(mem.deref(&pd.defeated_trobbio).unwrap_or_default()),
        //endregion: ChoralChambers

        // region: HighHalls
        Split::EnterHighHalls => {
            should_split(scenes.old == "Hang_01" && scenes.current == "Hang_02")
        }
        Split::EnterHighHallsGauntlet => {
            should_split(scenes.old == "Hang_06" && scenes.current == "Hang_04")
        }
        // endregion: HighHalls

        // region: ThreefoldMelody
        Split::VaultkeepersMelodyTrans => {
            should_split(mem.deref(&pd.has_melody_librarian).unwrap_or_default())
        }
        Split::ArchitectsMelodyTrans => {
            should_split(mem.deref(&pd.has_melody_architect).unwrap_or_default())
        }
        Split::ConductorsMelodyTrans => {
            should_split(mem.deref(&pd.has_melody_conductor).unwrap_or_default())
        }
        // endregion: ThreefoldMelody

        // else
        _ => should_split(false),
    }
}

pub fn continuous_splits(
    split: &Split,
    mem: &Memory,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
    _gp: &GameplayPointers,
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
        Split::MarrowBell => {
            should_split(mem.deref(&pd.bell_shrine_bone_forest).unwrap_or_default())
        }
        // endregion: Marrow

        // region: DeepDocks
        Split::SwiftStep => should_split(mem.deref(&pd.has_dash).unwrap_or_default()),
        Split::Lace1 => should_split(mem.deref(&pd.defeated_lace1).unwrap_or_default()),
        Split::DeepDocksBell => should_split(mem.deref(&pd.bell_shrine_wilds).unwrap_or_default()),
        // endregion: DeepDocks

        // region: FarFields
        Split::DriftersCloak => should_split(mem.deref(&pd.has_brolly).unwrap_or_default()),
        Split::FourthChorus => should_split(mem.deref(&pd.defeated_song_golem).unwrap_or_default()),
        // endregion: FarFields

        // region: Greymoor
        Split::GreymoorBell => {
            should_split(mem.deref(&pd.bell_shrine_greymoor).unwrap_or_default())
        }
        Split::Moorwing => should_split(
            mem.deref(&pd.defeated_vampire_gnat_boss)
                .unwrap_or_default(),
        ),
        // endregion: Greymoor

        // region: Shellwood
        Split::ClingGrip => should_split(mem.deref(&pd.has_wall_jump).unwrap_or_default()),
        Split::ShellwoodBell => {
            should_split(mem.deref(&pd.bell_shrine_shellwood).unwrap_or_default())
        }
        // endregion: Shellwood

        // region: Bellhart
        Split::Widow => should_split(mem.deref(&pd.spinner_defeated).unwrap_or_default()),
        Split::BellhartBell => {
            should_split(mem.deref(&pd.bell_shrine_bellhart).unwrap_or_default())
        }
        // endregion: Bellhart

        // region: BlastedSteps
        Split::LastJudge => should_split(mem.deref(&pd.defeated_last_judge).unwrap_or_default()),
        // endregion: BlastedSteps

        // region: Bilewater
        Split::Phantom => should_split(mem.deref(&pd.defeated_phantom).unwrap_or_default()),
        // endregion: Bilewater

        // region: Acts
        Split::Act2Started => should_split(mem.deref(&pd.act2_started).unwrap_or_default()),
        // endregion: Acts

        // region: CogworkCore
        Split::CogworkDancers => {
            should_split(mem.deref(&pd.defeated_cogwork_dancers).unwrap_or_default())
        }
        // endregion: CogworkCore

        // region: WhisperingVaults
        Split::WhisperingVaultsGauntlet => should_split(
            mem.deref(&pd.completed_library_entry_battle)
                .unwrap_or_default(),
        ),
        // endregion: WhisperingVaults

        // region: ChoralChambers
        Split::Trobbio => should_split(mem.deref(&pd.defeated_trobbio).unwrap_or_default()),
        //endregion: ChoralChambers

        // region: Underworks
        Split::Clawline => should_split(mem.deref(&pd.has_harpoon_dash).unwrap_or_default()),
        //endregion: Underworks

        // region: HighHalls
        Split::HighHallsGauntlet => should_split(mem.deref(&pd.hang04_battle).unwrap_or_default()),
        //endregion: HighHalls

        // region: TheCradle
        Split::Lace2 => should_split(mem.deref(&pd.defeated_lace_tower).unwrap_or_default()),
        // endregion: TheCradle

        // region: ThreefoldMelody
        Split::VaultkeepersMelody => {
            should_split(mem.deref(&pd.has_melody_librarian).unwrap_or_default())
        }
        Split::ArchitectsMelody => {
            should_split(mem.deref(&pd.has_melody_architect).unwrap_or_default())
        }
        Split::ConductorsMelody => {
            should_split(mem.deref(&pd.has_melody_conductor).unwrap_or_default())
        }
        Split::UnlockedMelodyLift => {
            should_split(mem.deref(&pd.unlocked_melody_lift).unwrap_or_default())
        }
        // endregion: ThreefoldMelody

        // region: NeedleUpgrade
        Split::NeedleUpgrade1 => {
            should_split(mem.deref(&pd.nail_upgrades).is_ok_and(|n: i32| n >= 1))
        }
        Split::NeedleUpgrade2 => {
            should_split(mem.deref(&pd.nail_upgrades).is_ok_and(|n: i32| n >= 2))
        }
        Split::NeedleUpgrade3 => {
            should_split(mem.deref(&pd.nail_upgrades).is_ok_and(|n: i32| n >= 3))
        }
        Split::NeedleUpgrade4 => {
            should_split(mem.deref(&pd.nail_upgrades).is_ok_and(|n: i32| n >= 4))
        }
        // endregion: NeedleUpgrade

        //region: FleaSpecific
        Split::SavedFleaAnt03 => should_split(mem.deref(&pd.savedflea_ant_03).unwrap_or_default()),
        Split::SavedFleaBelltown04 => should_split(mem.deref(&pd.savedflea_belltown_04).unwrap_or_default()),
        Split::SavedFleaBone06 => should_split(mem.deref(&pd.savedflea_bone_06).unwrap_or_default()),
        Split::SavedFleaBoneEast05 => should_split(mem.deref(&pd.savedflea_bone_east_05).unwrap_or_default()),
        Split::SavedFleaBoneEast10Church => should_split(mem.deref(&pd.savedflea_bone_east_10_church).unwrap_or_default()),
        Split::SavedFleaBoneEast17b => should_split(mem.deref(&pd.savedflea_bone_east_17b).unwrap_or_default()),
        Split::SavedFleaCoral24 => should_split(mem.deref(&pd.savedflea_coral_24).unwrap_or_default()),
        Split::SavedFleaCoral35 => should_split(mem.deref(&pd.savedflea_coral_35).unwrap_or_default()),
        Split::SavedFleaCrawl06 => should_split(mem.deref(&pd.savedflea_crawl_06).unwrap_or_default()),
        Split::SavedFleaDock03d => should_split(mem.deref(&pd.savedflea_dock_03d).unwrap_or_default()),
        Split::SavedFleaDock16 => should_split(mem.deref(&pd.savedflea_dock_16).unwrap_or_default()),
        Split::SavedFleaDust09 => should_split(mem.deref(&pd.savedflea_dust_09).unwrap_or_default()),
        Split::SavedFleaDust12 => should_split(mem.deref(&pd.savedflea_dust_12).unwrap_or_default()),
        Split::SavedFleaGreymoor06 => should_split(mem.deref(&pd.savedflea_greymoor_06).unwrap_or_default()),
        Split::SavedFleaGreymoor15b => should_split(mem.deref(&pd.savedflea_greymoor_15b).unwrap_or_default()),
        Split::SavedFleaLibrary01 => should_split(mem.deref(&pd.savedflea_library_01).unwrap_or_default()),
        Split::SavedFleaLibrary09 => should_split(mem.deref(&pd.savedflea_library_09).unwrap_or_default()),
        Split::SavedFleaPeak05c => should_split(mem.deref(&pd.savedflea_peak_05c).unwrap_or_default()),
        Split::SavedFleaShadow10 => should_split(mem.deref(&pd.savedflea_shadow_10).unwrap_or_default()),
        Split::SavedFleaShadow28 => should_split(mem.deref(&pd.savedflea_shadow_28).unwrap_or_default()),
        Split::SavedFleaShellwood03 => should_split(mem.deref(&pd.savedflea_shellwood_03).unwrap_or_default()),
        Split::SavedFleaSlab06 => should_split(mem.deref(&pd.savedflea_slab_06).unwrap_or_default()),
        Split::SavedFleaSlabCell => should_split(mem.deref(&pd.savedflea_slab_cell).unwrap_or_default()),
        Split::SavedFleaSong11 => should_split(mem.deref(&pd.savedflea_song_11).unwrap_or_default()),
        Split::SavedFleaSong14 => should_split(mem.deref(&pd.savedflea_song_14).unwrap_or_default()),
        Split::SavedFleaUnder21 => should_split(mem.deref(&pd.savedflea_under_21).unwrap_or_default()),
        Split::SavedFleaUnder23 => should_split(mem.deref(&pd.savedflea_under_23).unwrap_or_default()),
        Split::TamedGiantFlea => should_split(mem.deref(&pd.tamed_giant_flea).unwrap_or_default()),
        Split::MetTroupeHunterWild => should_split(mem.deref(&pd.met_troupe_hunter_wild).unwrap_or_default()),
        Split::CaravanLechSaved => should_split(mem.deref(&pd.caravan_lech_saved).unwrap_or_default()),
        // endregion: FleaSpecific

        // else
        _ => should_split(false),
    }
}

pub fn splits(
    split: &Split,
    mem: &Memory,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
    gp: &GameplayPointers,
    trans_now: bool,
    ss: &mut SceneStore,
) -> SplitterAction {
    let a1 = continuous_splits(split, mem, gm, pd, gp).or_else(|| {
        let scenes = ss.pair();
        if trans_now {
            transition_splits(split, &scenes, mem, gm, pd, gp)
        } else {
            SplitterAction::Pass
        }
    });
    if a1 != SplitterAction::Pass {
        ss.split_this_transition = true;
    }
    a1
}
