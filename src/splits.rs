use alloc::{vec, vec::Vec};
use asr::{settings::Gui, watcher::Pair};
use ugly_widget::{
    radio_button::{options_str, RadioButtonOptions},
    store::StoreWidget,
};

use crate::{
    silksong_memory::{
        is_menu, GameManagerPointers, Memory, PlayerDataPointers, SceneStore,
        DEATH_RESPAWN_MARKER_INIT, GAME_STATE_PLAYING, MENU_TITLE, NON_MENU_GAME_STATES,
        OPENING_SCENES,
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

    // region: Wormways
    /// Enter Wormways (Transition)
    ///
    /// Splits on entering Wormways
    EnterWormways,
    // endregion: Wormways

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
    /// Enter Shellwood (Transition)
    ///
    /// Splits when entering Shellwood
    EnterShellwood,
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
    /// Enter Bellhart (Transition)
    ///
    /// Splits when entering Bellhart
    EnterBellhart,
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
    /// Whispering Vaults Arena (Mini Boss)
    ///
    /// Splits when completing the Whispering Vaults Arena
    #[alias = "WhisperingVaultsGauntlet"]
    WhisperingVaultsArena,
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
    /// Enter High Halls Arena (Transition)
    ///
    /// Splits when entering the High Halls Arena room
    #[alias = "EnterHighHallsGauntlet"]
    EnterHighHallsArena,
    /// High Halls Arena (Mini Boss)
    ///
    /// Splits when completing the High Halls Arena
    #[alias = "HighHallsGauntlet"]
    HighHallsArena,
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

    // region: Crests
    /// Reaper Crest (Transition)
    ///
    /// Splits when leaving the church with the Reaper Crest unlocked
    ReaperCrestTrans,
    // endregion: Crests

    // region: FleaSpecific
    /// Rescued Flea Hunter's March (Flea)
    ///
    /// Splits after rescuing flea in Ant_03
    SavedFleaHuntersMarch,
    /// Rescued Flea Bellhart (Flea)
    ///
    /// Splits after rescuing flea in Belltown_04
    SavedFleaBellhart,
    /// Rescued Flea Marrow (Flea)
    ///
    /// Splits after rescuing flea in Bone_06
    SavedFleaMarrow,
    /// Rescued Flea Deep Docks Sprint (Flea)
    ///
    /// Splits after rescuing flea in Bone_East_05
    SavedFleaDeepDocksSprint,
    /// Rescued Flea Far Fields Pilgrim's Rest (Flea)
    ///
    /// Splits after rescuing flea in Bone_East_10_Church
    SavedFleaFarFieldsPilgrimsRest,
    /// Rescued Flea Far Fields Trap (Flea)
    ///
    /// Splits after rescuing flea in Bone_East_17b
    SavedFleaFarFieldsTrap,
    /// Rescued Flea Sands of Karak (Flea)
    ///
    /// Splits after rescuing flea in Coral_24
    SavedFleaSandsOfKarak,
    /// Rescued Flea Blasted Steps (Flea)
    ///
    /// Splits after rescuing flea in Coral_35
    SavedFleaBlastedSteps,
    /// Rescued Flea Wormways (Flea)
    ///
    /// Splits after rescuing flea in Crawl_06
    SavedFleaWormways,
    /// Rescued Flea Deep Docks Arena (Flea)
    ///
    /// Splits after rescuing flea in Dock_03d
    SavedFleaDeepDocksArena,
    /// Rescued Flea Deep Docks Bellway (Flea)
    ///
    /// Splits after rescuing flea in Dock_16
    SavedFleaDeepDocksBellway,
    /// Rescued Flea Bilewater Organ (Flea)
    ///
    /// Splits after rescuing flea in Dust_09
    SavedFleaBilewaterOrgan,
    /// Rescued Flea Sinner's Road (Flea)
    ///
    /// Splits after rescuing flea in Dust_12
    SavedFleaSinnersRoad,
    /// Rescued Flea Greymoor Roof (Flea)
    ///
    /// Splits after rescuing flea in Greymoor_06
    SavedFleaGreymoorRoof,
    /// Rescued Flea Greymoor Lake (Flea)
    ///
    /// Splits after rescuing flea in Greymoor_15b
    SavedFleaGreymoorLake,
    /// Rescued Flea Whispering Vaults (Flea)
    ///
    /// Splits after rescuing flea in Library_01
    SavedFleaWhisperingVaults,
    /// Rescued Flea Songclave (Flea)
    ///
    /// Splits after rescuing flea in Library_09
    SavedFleaSongclave,
    /// Rescued Flea Mount Fay (Flea)
    ///
    /// Splits after rescuing flea in Peak_05c
    SavedFleaMountFay,
    /// Rescued Flea Bilewater Trap (Flea)
    ///
    /// Splits after rescuing flea in Shadow_10
    SavedFleaBilewaterTrap,
    /// Rescued Flea Bilewater Thieves (Flea)
    ///
    /// Splits after rescuing flea in Shadow_28
    SavedFleaBilewaterThieves,
    /// Rescued Flea Shellwood (Flea)
    ///
    /// Splits after rescuing flea in Shellwood_03
    SavedFleaShellwood,
    /// Rescued Flea Slab Bellway (Flea)
    ///
    /// Splits after rescuing flea in Slab_06
    SavedFleaSlabBellway,
    /// Rescued Flea Slab Cage (Flea)
    ///
    /// Splits after rescuing flea in Slab_Cell
    SavedFleaSlabCage,
    /// Rescued Flea Choral Chambers Wind (Flea)
    ///
    /// Splits after rescuing flea in Song_11
    SavedFleaChoralChambersWind,
    /// Rescued Flea Choral Chambers Cage (Flea)
    ///
    /// Splits after rescuing flea in Song_14
    SavedFleaChoralChambersCage,
    /// Rescued Flea Underworks Explosions (Flea)
    ///
    /// Splits after rescuing flea in Under_21
    SavedFleaUnderworksExplosions,
    /// Rescued Flea Underworks Wisp Thicket (Flea)
    ///
    /// Splits after rescuing flea in Under_23
    SavedFleaUnderworksWispThicket,
    /// Defeated Giant Flea (Flea)
    ///
    /// Splits after defeating Giant Flea
    SavedFleaGiantFlea,
    /// Met Vog (Flea)
    ///
    /// Splits after talking to Vog
    SavedFleaVog,
    /// Freed Kratt (Flea)
    ///
    /// Splits after freeing Kratt
    SavedFleaKratt,
    // endregion: FleaSpecific

    // region: Bellways
    /// Putrified Ducts (Bellway)
    ///
    /// Splits after unlocking Putrified Ducts Bellway
    PutrifiedDuctsStation,
    /// Bellhart (Bellway)
    ///
    /// Splits after unlocking Bellhart Bellway
    BellhartStation,
    /// Far Fields (Bellway)
    ///
    /// Splits after unlocking Far Fields Bellway
    FarFieldsStation,
    /// Grand Bellway (Bellway)
    ///
    /// Splits after unlocking Grand Bellway
    GrandBellwayStation,
    /// Blasted Steps (Bellway)
    ///
    /// Splits after unlocking Blasted Steps Bellway
    BlastedStepsStation,
    /// Deep Docks (Bellway)
    ///
    /// Splits after unlocking Deep Docks Bellway
    DeepDocksStation,
    /// Greymoor (Bellway)
    ///
    /// Splits after unlocking Greymoor Bellway
    GreymoorStation,
    /// Mount Fay (Bellway)
    ///
    /// Splits after unlocking Mount Fay Bellway
    MountFayStation,
    /// Bilewater (Bellway)
    ///
    /// Splits after unlocking Bilewater Bellway
    BilewaterStation,
    /// Shellwood (Bellway)
    ///
    /// Splits after unlocking Shellwood Bellway
    ShellwoodStation,
    // endregion: Bellways

    // region: Ventricas
    /// Choral Chambers (Ventrica)
    ///
    /// Splits after unlocking Choral Chambers Ventrica
    ChoralChambersTube,
    /// Underworks (Ventrica)
    ///
    /// Splits after unlocking Underworks Ventrica
    UnderworksTube,
    /// Grand Bellway (Ventrica)
    ///
    /// Splits after unlocking Grand Bellway Ventrica
    CityBellwayTube,
    /// High Halls (Ventrica)
    ///
    /// Splits after unlocking High Halls Ventrica
    HighHallsTube,
    /// Songclave (Ventrica)
    ///
    /// Splits after unlocking Songclave Ventrica
    SongclaveTube,
    /// Memorium (Ventrica)
    ///
    /// Splits after unlocking Memorium Ventrica
    MemoriumTube,
    // endregion: Ventricas

    // region: ShakraEncounters
    /// Seen Shakra Bonebottom (NPC)
    ///
    /// Splits after seeing Shakra in Bonebottom
    SeenShakraBonebottom,
    /// Seen Shakra Marrow (NPC)
    ///
    /// Splits after seeing Shakra in Marrow
    SeenShakraMarrow,
    /// Seen Shakra Deep Docks (NPC)
    ///
    /// Splits after seeing Shakra in Deep Docks
    SeenShakraDeepDocks,
    /// Seen Shakra Far Fields (NPC)
    ///
    /// Splits after seeing Shakra in Far Fields
    SeenShakraFarFields,
    /// Seen Shakra Wormways (NPC)
    ///
    /// Splits after seeing Shakra in Wormways
    SeenShakraWormways,
    /// Seen Shakra Greymoor (NPC)
    ///
    /// Splits after seeing Shakra in Greymoor
    SeenShakraGreymoor,
    /// Seen Shakra Bellhart (NPC)
    ///
    /// Splits after seeing Shakra in Bellhart
    SeenShakraBellhart,
    /// Seen Shakra Shellwood (NPC)
    ///
    /// Splits after seeing Shakra in Shellwood
    SeenShakraShellwood,
    /// Seen Shakra Hunter's March (NPC)
    ///
    /// Splits after seeing Shakra in Hunter's March
    SeenShakraHuntersMarch,
    /// Seen Shakra Blasted Steps (NPC)
    ///
    /// Splits after seeing Shakra in Blasted Steps
    SeenShakraBlastedSteps,
    /// Seen Shakra Sinner's Road (NPC)
    ///
    /// Splits after seeing Shakra in Sinner's Road
    SeenShakraSinnersRoad,
    /// Seen Shakra Mount Fay (NPC)
    ///
    /// Splits after seeing Shakra in Mount Fay
    SeenShakraMountFay,
    /// Seen Shakra Bilewater (NPC)
    ///
    /// Splits after seeing Shakra in Bilewater
    SeenShakraBilewater,
    /// Seen Shakra Sands of Karak (NPC)
    ///
    /// Splits after seeing Shakra in Sands of Karak
    SeenShakraSandsOfKarak,
    // endregion: ShakraEncounters

    // region: MiscTE
    /// Met Merchant Enclave (NPC)
    ///
    /// Splits after talking to Jubilana in Songclave
    MetJubilanaEnclave,
    /// Met Sherma Enclave (NPC)
    ///
    /// Splits after talking to Sherma in Songclave
    MetShermaEnclave,
    /// Unlock Prince Cage (Event)
    ///
    /// Splits when you unlock Green Prince's Cage in Sinner's Road
    UnlockedPrinceCage,
    /// Met Green Prince Cogwork (NPC)
    ///
    /// Splits when you talk to Green Prince in Cogwork Dancer's arena
    GreenPrinceInVerdania,
    /// Seen Fleatopia Empty (Event)
    ///
    /// Splits when you find Fleatopias location
    SeenFleatopiaEmpty,
    /// Faydown Cloak (Skill)
    ///
    /// Splits when you obtain Double Jump
    FaydownCloak,
    /// Silk Soar (Skill)
    ///
    /// Splits when you obtain Super Jump
    SilkSoar,
    /// Nyleth's Heart (Item)
    ///
    /// Splits when you obtain Nyleth's Heart
    CollectedHeartNyleth,
    /// Khann's Heart (Item)
    ///
    /// Splits when you obtain Khann's Heart
    CollectedHeartKhann,
    /// Karmelita's Heart (Item)
    ///
    /// Splits when you obtain Karmelita's Heart
    CollectedHeartKarmelita,
    /// Clover Dancer's Heart (Item)
    ///
    /// Split when you obtain Conjoined Heart
    CollectedHeartClover,
    /// Red Memory (Event)
    ///
    /// Splits on completing Red Memory
    CompletedRedMemory,
    /// Pavo Bellhome Key (NPC)
    ///
    /// Splits when obtaining Bellhome Key from Pavo
    BellhouseKeyConversation,
    // endregion: Misc TE
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

pub fn menu_splits(
    split: &Split,
    scenes: &Pair<&str>,
    _mem: &Memory,
    _gm: &GameManagerPointers,
    _pd: &PlayerDataPointers,
) -> SplitterAction {
    match split {
        // region: Start, End, and Menu
        Split::Menu => should_split(scenes.current == MENU_TITLE),
        // endregion: Start, End, and Menu

        // else
        _ => should_split(false),
    }
}

pub fn transition_splits(
    split: &Split,
    scenes: &Pair<&str>,
    mem: &Memory,
    _gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
) -> SplitterAction {
    match split {
        // region: Start, End, and Menu
        Split::EndingSplit => should_split(scenes.current.starts_with("Cinematic_Ending")),
        Split::EndingA => should_split(scenes.current == "Cinematic_Ending_A"),
        Split::AnyTransition => should_split(true),
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

        // region: Wormways
        Split::EnterWormways => should_split(
            (scenes.old == "Crawl_02" && scenes.current == "Crawl_03b")
                || (scenes.old == "Aspid_01" && scenes.current == "Crawl_01"),
        ),
        // endregion: Wormways

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

        // region: Bellhart
        Split::EnterBellhart => should_split(
            (scenes.old == "Belltown_06" || scenes.old == "Belltown_07")
                && scenes.current == "Belltown",
        ),
        // endregion: Bellhart

        // region: Shellwood
        Split::ClingGripTrans => should_split(mem.deref(&pd.has_wall_jump).unwrap_or_default()),
        Split::EnterShellwood => should_split(
            !scenes.old.starts_with("Shellwood") && scenes.current.starts_with("Shellwood"),
        ),
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
        Split::EnterHighHallsArena => {
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

pub fn transition_once_splits(
    split: &Split,
    scenes: &Pair<&str>,
    mem: &Memory,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
) -> SplitterAction {
    match split {
        // region: Start, End, and Menu
        Split::StartNewGame => should_split(
            scenes.current == "Tut_01"
                && (OPENING_SCENES.contains(&scenes.old)
                    || (scenes.old == MENU_TITLE
                        && mem.read_string(&gm.entry_gate_name).unwrap_or_default()
                            == DEATH_RESPAWN_MARKER_INIT))
                && mem.deref(&pd.disable_pause).is_ok_and(|d: bool| !d)
                && mem
                    .deref(&gm.game_state)
                    .is_ok_and(|s: i32| s == GAME_STATE_PLAYING),
        ),

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
        Split::WhisperingVaultsArena => should_split(
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
        Split::HighHallsArena => should_split(mem.deref(&pd.hang04_battle).unwrap_or_default()),
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

        // region: Crests
        Split::ReaperCrestTrans => {
            should_split(mem.deref(&pd.completed_memory_reaper).unwrap_or_default())
        }
        // endregion: Crests

        // region: FleaSpecific
        Split::SavedFleaHuntersMarch => {
            should_split(mem.deref(&pd.savedflea_ant_03).unwrap_or_default())
        }
        Split::SavedFleaBellhart => {
            should_split(mem.deref(&pd.savedflea_belltown_04).unwrap_or_default())
        }
        Split::SavedFleaMarrow => {
            should_split(mem.deref(&pd.savedflea_bone_06).unwrap_or_default())
        }
        Split::SavedFleaDeepDocksSprint => {
            should_split(mem.deref(&pd.savedflea_bone_east_05).unwrap_or_default())
        }
        Split::SavedFleaFarFieldsPilgrimsRest => should_split(
            mem.deref(&pd.savedflea_bone_east_10_church)
                .unwrap_or_default(),
        ),
        Split::SavedFleaFarFieldsTrap => {
            should_split(mem.deref(&pd.savedflea_bone_east_17b).unwrap_or_default())
        }
        Split::SavedFleaSandsOfKarak => {
            should_split(mem.deref(&pd.savedflea_coral_24).unwrap_or_default())
        }
        Split::SavedFleaBlastedSteps => {
            should_split(mem.deref(&pd.savedflea_coral_35).unwrap_or_default())
        }
        Split::SavedFleaWormways => {
            should_split(mem.deref(&pd.savedflea_crawl_06).unwrap_or_default())
        }
        Split::SavedFleaDeepDocksArena => {
            should_split(mem.deref(&pd.savedflea_dock_03d).unwrap_or_default())
        }
        Split::SavedFleaDeepDocksBellway => {
            should_split(mem.deref(&pd.savedflea_dock_16).unwrap_or_default())
        }
        Split::SavedFleaBilewaterOrgan => {
            should_split(mem.deref(&pd.savedflea_dust_09).unwrap_or_default())
        }
        Split::SavedFleaSinnersRoad => {
            should_split(mem.deref(&pd.savedflea_dust_12).unwrap_or_default())
        }
        Split::SavedFleaGreymoorRoof => {
            should_split(mem.deref(&pd.savedflea_greymoor_06).unwrap_or_default())
        }
        Split::SavedFleaGreymoorLake => {
            should_split(mem.deref(&pd.savedflea_greymoor_15b).unwrap_or_default())
        }
        Split::SavedFleaWhisperingVaults => {
            should_split(mem.deref(&pd.savedflea_library_01).unwrap_or_default())
        }
        Split::SavedFleaSongclave => {
            should_split(mem.deref(&pd.savedflea_library_09).unwrap_or_default())
        }
        Split::SavedFleaMountFay => {
            should_split(mem.deref(&pd.savedflea_peak_05c).unwrap_or_default())
        }
        Split::SavedFleaBilewaterTrap => {
            should_split(mem.deref(&pd.savedflea_shadow_10).unwrap_or_default())
        }
        Split::SavedFleaBilewaterThieves => {
            should_split(mem.deref(&pd.savedflea_shadow_28).unwrap_or_default())
        }
        Split::SavedFleaShellwood => {
            should_split(mem.deref(&pd.savedflea_shellwood_03).unwrap_or_default())
        }
        Split::SavedFleaSlabBellway => {
            should_split(mem.deref(&pd.savedflea_slab_06).unwrap_or_default())
        }
        Split::SavedFleaSlabCage => {
            should_split(mem.deref(&pd.savedflea_slab_cell).unwrap_or_default())
        }
        Split::SavedFleaChoralChambersWind => {
            should_split(mem.deref(&pd.savedflea_song_11).unwrap_or_default())
        }
        Split::SavedFleaChoralChambersCage => {
            should_split(mem.deref(&pd.savedflea_song_14).unwrap_or_default())
        }
        Split::SavedFleaUnderworksExplosions => {
            should_split(mem.deref(&pd.savedflea_under_21).unwrap_or_default())
        }
        Split::SavedFleaUnderworksWispThicket => {
            should_split(mem.deref(&pd.savedflea_under_23).unwrap_or_default())
        }
        Split::SavedFleaGiantFlea => {
            should_split(mem.deref(&pd.tamed_giant_flea).unwrap_or_default())
        }
        Split::SavedFleaVog => {
            should_split(mem.deref(&pd.met_troupe_hunter_wild).unwrap_or_default())
        }
        Split::SavedFleaKratt => {
            should_split(mem.deref(&pd.caravan_lech_saved).unwrap_or_default())
        }
        // endregion: FleaSpecific

        // region: Stations (Bellway)
        Split::PutrifiedDuctsStation => {
            should_split(mem.deref(&pd.unlocked_aqueduct_station).unwrap_or_default())
        }
        Split::BellhartStation => {
            should_split(mem.deref(&pd.unlocked_belltown_station).unwrap_or_default())
        }
        Split::FarFieldsStation => should_split(
            mem.deref(&pd.unlocked_boneforest_east_station)
                .unwrap_or_default(),
        ),
        Split::GrandBellwayStation => {
            should_split(mem.deref(&pd.unlocked_city_station).unwrap_or_default())
        }
        Split::BlastedStepsStation => should_split(
            mem.deref(&pd.unlocked_coral_tower_station)
                .unwrap_or_default(),
        ),
        Split::DeepDocksStation => {
            should_split(mem.deref(&pd.unlocked_docks_station).unwrap_or_default())
        }
        Split::GreymoorStation => {
            should_split(mem.deref(&pd.unlocked_greymoor_station).unwrap_or_default())
        }
        Split::MountFayStation => {
            should_split(mem.deref(&pd.unlocked_peak_station).unwrap_or_default())
        }
        Split::BilewaterStation => {
            should_split(mem.deref(&pd.unlocked_shadow_station).unwrap_or_default())
        }
        Split::ShellwoodStation => should_split(
            mem.deref(&pd.unlocked_shellwood_station)
                .unwrap_or_default(),
        ),
        // endregion: Stations (Bellway)

        // region: Ventricas
        Split::ChoralChambersTube => {
            should_split(mem.deref(&pd.unlocked_song_tube).unwrap_or_default())
        }
        Split::UnderworksTube => {
            should_split(mem.deref(&pd.unlocked_under_tube).unwrap_or_default())
        }
        Split::CityBellwayTube => should_split(
            mem.deref(&pd.unlocked_city_bellway_tube)
                .unwrap_or_default(),
        ),
        Split::HighHallsTube => should_split(mem.deref(&pd.unlocked_hang_tube).unwrap_or_default()),
        Split::SongclaveTube => {
            should_split(mem.deref(&pd.unlocked_enclave_tube).unwrap_or_default())
        }
        Split::MemoriumTube => {
            should_split(mem.deref(&pd.unlocked_arborium_tube).unwrap_or_default())
        }
        // endregion: Ventricas

        // region: ShakraEncounters
        Split::SeenShakraBonebottom => {
            should_split(mem.deref(&pd.seen_mapper_bonetown).unwrap_or_default())
        }
        Split::SeenShakraMarrow => {
            should_split(mem.deref(&pd.seen_mapper_bone_forest).unwrap_or_default())
        }
        Split::SeenShakraDeepDocks => {
            should_split(mem.deref(&pd.seen_mapper_docks).unwrap_or_default())
        }
        Split::SeenShakraFarFields => {
            should_split(mem.deref(&pd.seen_mapper_wilds).unwrap_or_default())
        }
        Split::SeenShakraWormways => {
            should_split(mem.deref(&pd.seen_mapper_crawl).unwrap_or_default())
        }
        Split::SeenShakraGreymoor => {
            should_split(mem.deref(&pd.seen_mapper_greymoor).unwrap_or_default())
        }
        Split::SeenShakraBellhart => {
            should_split(mem.deref(&pd.seen_mapper_bellhart).unwrap_or_default())
        }
        Split::SeenShakraShellwood => {
            should_split(mem.deref(&pd.seen_mapper_shellwood).unwrap_or_default())
        }
        Split::SeenShakraHuntersMarch => {
            should_split(mem.deref(&pd.seen_mapper_hunters_nest).unwrap_or_default())
        }
        Split::SeenShakraBlastedSteps => {
            should_split(mem.deref(&pd.seen_mapper_judge_steps).unwrap_or_default())
        }
        Split::SeenShakraSinnersRoad => {
            should_split(mem.deref(&pd.seen_mapper_dustpens).unwrap_or_default())
        }
        Split::SeenShakraMountFay => {
            should_split(mem.deref(&pd.seen_mapper_peak).unwrap_or_default())
        }
        Split::SeenShakraBilewater => {
            should_split(mem.deref(&pd.seen_mapper_shadow).unwrap_or_default())
        }
        Split::SeenShakraSandsOfKarak => {
            should_split(mem.deref(&pd.seen_mapper_coral_caverns).unwrap_or_default())
        }
        // endregion: ShakraEncounters

        // region: MiscTE
        Split::MetJubilanaEnclave => {
            should_split(mem.deref(&pd.met_city_merchant_enclave).unwrap_or_default())
        }
        Split::MetShermaEnclave => {
            should_split(mem.deref(&pd.met_sherma_enclave).unwrap_or_default())
        }
        Split::UnlockedPrinceCage => {
            should_split(mem.deref(&pd.unlocked_dust_cage).unwrap_or_default())
        }
        Split::GreenPrinceInVerdania => should_split(
            mem.deref(&pd.green_prince_location)
                .is_ok_and(|n: i32| n == 3),
        ),
        Split::SeenFleatopiaEmpty => {
            should_split(mem.deref(&pd.seen_fleatopia_empty).unwrap_or_default())
        }
        Split::FaydownCloak => should_split(mem.deref(&pd.has_double_jump).unwrap_or_default()),
        Split::SilkSoar => should_split(mem.deref(&pd.has_super_jump).unwrap_or_default()),
        Split::CollectedHeartNyleth => {
            should_split(mem.deref(&pd.collected_heart_flower).unwrap_or_default())
        }
        Split::CollectedHeartKhann => {
            should_split(mem.deref(&pd.collected_heart_coral).unwrap_or_default())
        }
        Split::CollectedHeartKarmelita => {
            should_split(mem.deref(&pd.collected_heart_hunter).unwrap_or_default())
        }
        Split::CollectedHeartClover => {
            should_split(mem.deref(&pd.collected_heart_clover).unwrap_or_default())
        }
        Split::CompletedRedMemory => {
            should_split(mem.deref(&pd.completed_red_memory).unwrap_or_default())
        }
        Split::BellhouseKeyConversation => should_split(
            mem.deref(&pd.belltown_greeter_house_full_dlg)
                .unwrap_or_default(),
        ),
        // endregion: MiscTE

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
        let a2 = if !ss.split_this_transition {
            transition_once_splits(split, &scenes, mem, gm, pd)
        } else {
            SplitterAction::Pass
        };
        a2.or_else(|| {
            if trans_now {
                if is_menu(scenes.old) || is_menu(scenes.current) {
                    menu_splits(split, &scenes, mem, gm, pd)
                } else {
                    transition_splits(split, &scenes, mem, gm, pd)
                }
            } else {
                SplitterAction::Pass
            }
        })
    });
    if a1 != SplitterAction::Pass {
        ss.split_this_transition = true;
    }
    a1
}
