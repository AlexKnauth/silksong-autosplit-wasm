use alloc::{vec, vec::Vec};
use asr::{settings::Gui, watcher::Pair};
use ugly_widget::{
    radio_button::{options_str, RadioButtonOptions},
    store::StoreWidget,
};
use utf16_lit::utf16;

use crate::{
    silksong_memory::{
        get_at_bench, get_health, get_heart_pieces, get_max_health_base, get_respawn_scene,
        is_discontinuity_scene, is_menu, Env, SceneStore, CINEMATIC_STAG_TRAVEL,
        DEATH_RESPAWN_MARKER_INIT, GAME_STATE_PLAYING, MENU_TITLE, NON_MENU_GAME_STATES,
        OPENING_SCENES,
    },
    store::Store,
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
    /// from the opening cutscenes into Moss Grotto
    StartNewGame,
    /// Act 1 Start (Start)
    ///
    /// Splits when starting Act 1,
    /// either from the starting Bind,
    /// or from revert autosave Act1Start
    Act1Start,
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
    /// Any Bench (Bench)
    ///
    /// Splits when sitting on a bench
    BenchAny,
    /// Death (Event)
    ///
    /// Splits when player HP is 0
    PlayerDeath,
    /// Any Transition (Transition)
    ///
    /// Splits when entering a transition (only one will split per transition)
    AnyTransition,
    /// Transition excluding discontinuities (Transition)
    ///
    /// Splits when entering a transition
    /// (excludes discontinuities including save states, deaths, and bellway travel)
    TransitionExcludingDiscontinuities,
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
    /// Enter Bone Bottom (Transition)
    ///
    /// Splits when entering Bone Bottom
    EnterBoneBottom,
    /// Enter Mosshome (Transition)
    ///
    /// Splits when entering Mosshome (Mosstown_01)
    EnterMosshome,
    /// Silk Spear (Skill)
    ///
    /// Splits when obtaining Silk Spear
    SilkSpear,
    /// Silk Spear (Transition)
    ///
    /// Splits on the transition after obtaining Silk Spear
    SilkSpearTrans,
    /// Bone Bottom Simple Key (Item)
    ///
    /// Splits when buying the Bone Bottom simple key from Pebb
    BoneBottomSimpleKey,
    /// Enter Weavenest Atla (Transition)
    ///
    /// Splits on entering Weavenest Atla
    EnterWeavenestAtla,
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
    /// Enter Upper Wormways (Transition)
    ///
    /// Splits on entering the vertical transition to the upper portion of Wormways (Crawl_03)
    EnterUpperWormways,
    /// Sharpdart (Skill)
    ///
    /// Splits when obtaining Sharpdart
    Sharpdart,
    /// Sharpdart (Transition)
    ///
    /// Splits on the transition after obtaining Sharpdart
    SharpdartTrans,
    // endregion: Wormways

    // region: HuntersMarch
    /// Enter Hunter's March (Transition)
    ///
    /// Splits on entering a Hunter's March transition with area text
    /// (includes post-Skarrguard room and tall room)
    EnterHuntersMarch,
    /// Hunter's March - Post-Middle Arena (Transition)
    ///
    /// Splits on transition to the room after the middle arena in Hunter's March
    HuntersMarchPostMiddleArenaTransition,
    // endregion: HuntersMarch

    // region: FarFields
    /// Enter Far Fields (Transition)
    ///
    /// Splits when entering Far Fields
    EnterFarFields,
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
    /// Gurr the Outcast Encountered (Boss)
    ///
    /// Splits when encountering Gurr the Outcast
    GurrTheOutcastEncountered,
    /// Gurr the Outcast (Boss)
    ///
    /// Splits when Gurr the Outcast is defeated
    GurrTheOutcast,
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
    /// Thread Storm (Skill)
    ///
    /// Splits when obtaining Thread Storm
    ThreadStorm,
    /// Thread Storm (Transition)
    ///
    /// Splits on the transition after obtaining Thread Storm
    ThreadStormTrans,
    /// Halfway Basement (Transition)
    ///
    /// Splits after entering the basement in Halfway Home
    EnterHalfwayHomeBasement,
    /// Yarnaby Slap (Event)
    ///
    /// Splits when the first dialogue box appears after slapping Yarnaby post Cursed Crest-cure
    YarnabySlap,
    // endregion: Greymoor

    // region: WispThicket
    /// Enter Wisp Thicket (Transition)
    ///
    /// Splits when entering the Wisp Thicket from Greymoor
    EnterWispThicket,
    /// Enter Father of the Flame (Transition)
    ///
    /// Splits when entering the arena for the Father of the Flame boss
    EnterFatherOfTheFlame,
    /// Father of the Flame (Boss)
    ///
    /// Splits when defeating Father of the Flame
    FatherOfTheFlame,
    // endregion: WispThicket

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
    /// Widow Encountered (Boss)
    ///
    /// Splits when Widow boss encounter begins, after she disappears from the background scene
    WidowEncountered,
    /// Widow (Boss)
    ///
    /// Splits when killing Widow
    Widow,
    /// Main Menu w/ Needolin Memory (Menu)
    ///
    /// Splits on the main menu after waking up from the Needolin Memory
    MenuNeedolinMemory,
    /// Bellhart Bell (Event)
    ///
    /// Splits when ringing the Bellhart Bell Shrine
    BellhartBell,
    // endregion: Bellhart

    // region: BlastedSteps
    /// Enter Blasted Steps (Transition)
    ///
    /// Splits when entering the Blasted Steps from Shellwood, where the area text appears
    EnterBlastedSteps,
    /// Great Conchflies (Boss)
    ///
    /// Splits after defeating the Great Conchflies (two-enemy encounter) boss
    #[alias = "Conchflies1"]
    GreatConchflies,
    /// Great Conchflies (Transition)
    ///
    /// Splits on the transition after defeating the Great Conchflies boss
    GreatConchfliesTrans,
    /// Needle Strike (Skill)
    ///
    /// Splits when obtaining Needle Strike (Charge Slash)
    NeedleStrike,
    /// Needle Strike (Transition)
    ///
    /// Splits on the transition after obtaining Needle Strike (Charge Slash)
    NeedleStrikeTrans,
    /// Enter Last Judge (Transition)
    ///
    /// Splits when entering the Last Judge boss arena from the Blasted Steps
    EnterLastJudge,
    /// Last Judge Encountered (Boss)
    ///
    /// Splits after the Last Judge boss fight is first encountered, post-needolin
    LastJudgeEncountered,
    /// Last Judge (Boss)
    ///
    /// Splits when killing Last Judge
    LastJudge,
    /// Enter Citadel Front Gate (Transition)
    ///
    /// Splits when entering the Citadel past the Last Judge arena
    EnterCitadelFrontGate,
    // endregion: BlastedSteps

    // region: SinnersRoad
    /// Enter Sinner's Road (Transition)
    ///
    /// Splits when entering Sinner's Road from Greymoor
    EnterSinnersRoad,
    /// Lugoli (Boss)
    ///
    /// Splits after defeating Lugoli
    Lugoli,
    // endregion: SinnersRoad

    // region: TheMist
    /// Enter The Mist (Transition)
    ///
    /// Splits when entering The Mist
    EnterMist,
    /// Mist Crossing (Transition)
    ///
    /// Splits when entering the Mist's midpoint crossing room
    MistCrossing,
    /// Leave The Mist (Transition)
    ///
    /// Splits when leaving The Mist
    LeaveMist,
    // endregion: TheMist

    // region: Bilewater
    /// Enter Bilewater (Transition)
    ///
    /// Splits when entering a Bilewater room with area text
    /// (excludes outer Exhaust Organ room)
    EnterBilewater,
    /// Enter Exhaust Organ (Transition)
    ///
    /// Splits when entering the Exhaust Organ from Bilewater
    EnterExhaustOrgan,
    /// Phantom (Boss)
    ///
    /// Splits when killing Phantom
    Phantom,
    /// Phantom (Transition)
    ///
    /// Splits on the transition after killing Phantom
    PhantomTrans,
    /// Cross Stitch (Skill)
    ///
    /// Splits when obtaining Cross Stitch
    CrossStitch,
    /// Cross Stitch (Transition)
    ///
    /// Splits on the transition after obtaining Cross Stitch
    CrossStitchTrans,
    /// Trail's End (Transition)
    ///
    /// Splits on the transition out of the room to complete Trail's End
    TrailsEndTrans,
    // endregion: Bilewater

    // region: TheSlab
    /// Enter The Slab (Transition)
    ///
    /// Splits when entering the Slab's front entrance
    EnterTheSlab,
    /// Wardenfly Capture (Transition)
    ///
    /// Splits when entering the Slab after being captured by a Wardenfly
    WardenflyCaptureTrans,
    /// Key of Indolent (Item)
    ///
    /// Splits when you obtain the Indolent slab key
    SlabKeyIndolent,
    /// Key of Heretic (Item)
    ///
    /// Splits when you obtain the Heretic slab key
    SlabKeyHeretic,
    /// Key of Apostate (Item)
    ///
    /// Splits when you obtain the Apostate slab key
    SlabKeyApostate,
    /// Broodmother (Boss)
    ///
    /// Splits after defeating the Broodmother
    Broodmother,
    /// Enter First Sinner (Transition)
    ///
    /// Splits when entering the First Sinner's boss arena
    EnterFirstSinner,
    /// First Sinner Encountered (Boss)
    ///
    /// Splits when the First Sinner boss is activated the first time after binding
    FirstSinnerEncountered,
    /// First Sinner (Boss)
    ///
    /// Splits when defeating the First Sinner, after the memory sequence
    FirstSinner,
    /// Rune Rage (Skill)
    ///
    /// Splits when obtaining Rune Rage
    RuneRage,
    /// Rune Rage (Transition)
    ///
    /// Splits on the transition after obtaining Rune Rage
    RuneRageTrans,
    // endregion: TheSlab

    // region: MountFay
    /// Enter Mount Fay (Transition)
    ///
    /// Splits when entering Mount Fay
    EnterMountFay,
    /// Enter Brightvein (Transition)
    ///
    /// Splits when entering the Brightvein sub-area
    EnterBrightvein,
    /// Upper Mount Fay (Transition)
    ///
    /// Splits when entering the upper portion of Mount Fay on the right
    UpperMountFayTrans,
    /// Faydown Cloak (Skill)
    ///
    /// Splits when you obtain Faydown Cloak (Double Jump)
    FaydownCloak,
    // endregion: MountFay

    // region: SandsOfKarak
    /// Enter Sands of Karak (Transition)
    ///
    /// Splits when entering Sands of Karak
    EnterSandsOfKarak,
    /// Enter Voltnest (Transition)
    ///
    /// Splits when entering the Voltnest subregion
    EnterVoltnest,
    /// Voltvyrm (Boss)
    ///
    /// Splits after defeating Voltvyrm
    Voltvyrm,
    /// Raging Conchfly (Boss)
    ///
    /// Splits when the Raging Conchfly (single-enemy second encounter) is defeated
    RagingConchfly,
    /// Raging Conchfly (Transition)
    ///
    /// Splits on the transition after defeating Raging Conchfly
    RagingConchflyTrans,
    /// Watcher at the Edge (Boss)
    ///
    /// Splits when Watcher at the Edge is defeated
    WatcherAtTheEdge,
    // endregion: SandsOfKarak

    // region: Acts
    /// Act 2 Started (Event)
    ///
    /// Splits when starting Act 2
    Act2Started,
    // endregion: Acts

    // region: CogworkCore
    /// Enter Cogwork Dancers (Transition)
    ///
    /// Splits when entering the Cogwork Dancers boss arena
    EnterCogworkDancers,
    /// Cogwork Dancers Encountered (Boss)
    ///
    /// Splits when first encountering the Cogwork Dancers boss
    CogworkDancersEncountered,
    /// Cogwork Dancers (Boss)
    ///
    /// Splits when killing Cogwork Dancers
    CogworkDancers,
    /// Enter Cogwork Core (Transition)
    ///
    /// Splits when entering the main region of Cogwork Core, above or below the Cogwork Dancers arena
    EnterCogworkCore,
    /// Second Sentinel Awoken (Event)
    ///
    /// Splits when using the Cogheart to activate Second Sentinel
    SecondSentinelAwoken,
    // endregion: CogworkCore

    // region: WhisperingVaults
    /// Enter Whispering Vaults (Transition)
    ///
    /// Splits when entering the rooms where the Whispering Vaults area text appears, past the arena or from Songclave
    EnterWhisperingVaults,
    /// Whispering Vaults Arena (Mini Boss)
    ///
    /// Splits when completing the Whispering Vaults Arena
    #[alias = "WhisperingVaultsGauntlet"]
    WhisperingVaultsArena,
    // endregion: WhisperingVaults

    // region: ChoralChambers
    /// Enter Songclave (Transition)
    ///
    /// Splits when entering Songclave
    EnterSongclave,
    /// Trobbio (Boss)
    ///
    /// Splits when killing Trobbio
    Trobbio,
    /// Trobbio (Transition)
    ///
    /// Splits on the transition after killing Trobbio
    TrobbioTrans,
    /// Met Mergwin (NPC)
    ///
    /// Splits after talking to Loyal Mergwin the first time
    MetMergwin,
    /// Delivered Courier's Rasher (Event)
    ///
    /// Splits after giving the Courier's Rasher to Loyal Mergwin
    #[alias = "GivenCouriersRasher"]
    DeliveredCouriersRasher,
    /// Great Taste Reward (Item)
    ///
    /// Splits when collecting the Pale Oil reward from the Great Taste of Pharloom wish
    GreatTasteReward,
    // endregion: ChoralChambers

    // region: Underworks
    /// Clawline (Skill)
    ///
    /// Splits when obtaining Clawline (Harpoon Dash)
    Clawline,
    /// Post-Clawline Arena (Transition)
    ///
    /// Splits when exiting the Cauldron through the clawline-locked arena transition
    PostClawlineArenaTrans,
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
    /// Enter Second Sentinel (Transition)
    ///
    /// Splits when entering the arena for the Second Sentinel boss fight
    EnterSecondSentinel,
    /// Second Sentinel Encountered (Boss)
    ///
    /// Splits when the Second Sentinel boss encounter begins the first time
    SecondSentinelBossEncountered,
    /// Second Sentinel (Boss)
    ///
    /// Splits when Second Sentinel is defeated
    SecondSentinel,
    // endregion: HighHalls

    // region: Whiteward
    /// Enter Whiteward (Transition)
    ///
    /// Splits when entering the upper portion of Whiteward from the Choral Chambers
    EnterWhiteward,
    /// Post-Whiteward Elevator (Transition)
    ///
    /// Splits when taking any transition further into Whiteward past the elevator
    PostWhitewardElevatorTrans,
    /// Collected Whiteward Key (Item)
    ///
    /// Splits when you collect the Whiteward Key
    #[alias = "CollectedWhiteWardKey"]
    CollectedWhitewardKey,
    /// Collected Surgeon's Key (Item)
    ///
    /// Splits when you collect the Surgeon's Key
    CollectedSurgeonsKey,
    /// The Unravelled Encountered (Boss)
    ///
    /// Splits when The Unravelled boss first appears
    TheUnravelledEncountered,
    /// The Unravelled (Boss)
    ///
    /// Splits when The Unravelled is defeated
    TheUnravelled,
    // endregion: Whiteward

    // region: Memorium
    /// Enter Memorium (Transition)
    ///
    /// Splits when entering the Memorium
    EnterMemorium,
    // endregion: Memorium

    // region: PutrifiedDucts
    /// Enter Putrified Ducts (Transition)
    ///
    /// Splits when entering the Putrified Ducts in a room with area text
    /// (excludes Huntress room, includes tall room)
    EnterPutrifiedDucts,
    /// Flea Festival Begin (Event)
    ///
    /// Splits when starting the Flea Festival in Fleatopia
    FleaFestivalBegin,
    /// Flea Festival End (Event)
    ///
    /// Splits when the Flea Festival has ended
    FleaFestivalEnd,
    // endregion: PutrifiedDucts

    // region: TheCradle
    /// Lace 2 (Boss)
    ///
    /// Splits when defeating Lace 2 in the Cradle
    Lace2,
    /// Post-Lace 2 Arena (Transition)
    ///
    /// Splits when taking the transition after Lace 2's arena into the ventrica terminus
    PostLace2ArenaTrans,
    /// Enter The Cradle (Transition)
    ///
    /// Splits when entering the full Cradle area, past the ventrica terminus
    EnterTheCradle,
    /// Pale Nails (Skill)
    ///
    /// Splits when obtaining Pale Nails
    PaleNails,
    /// Pale Nails (Transition)
    ///
    /// Splits on the transition after obtaining Pale Nails
    PaleNailsTrans,
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

    // region: MaskShards
    /// Mask Shard (Obtain)
    ///
    /// Splits when obtaining a Mask Shard or a complete Mask upgrade
    OnObtainMaskShard,
    /// Mask Shard 1 (Fragment)
    ///
    /// Splits when getting 1st Mask Shard
    MaskShard1,
    /// Mask Shard 2 (Fragment)
    ///
    /// Splits when getting 2nd Mask Shard
    MaskShard2,
    /// Mask Shard 3 (Fragment)
    ///
    /// Splits when getting 3rd Mask Shard
    MaskShard3,
    /// Mask Upgrade 4 (Upgrade)
    ///
    /// Splits when getting 1 extra Masks (6 base HP)
    Mask1,
    /// Mask Shard 5 (Fragment)
    ///
    /// Splits when getting 5th Mask Shard
    MaskShard5,
    /// Mask Shard 6 (Fragment)
    ///
    /// Splits when getting 6th Mask Shard
    MaskShard6,
    /// Mask Shard 7 (Fragment)
    ///
    /// Splits when getting 7th Mask Shard
    MaskShard7,
    /// Mask Upgrade 8 (Upgrade)
    ///
    /// Splits when getting 2 extra Masks (7 base HP)
    Mask2,
    /// Mask Shard 9 (Fragment)
    ///
    /// Splits when getting 9th Mask Shard
    MaskShard9,
    /// Mask Shard 10 (Fragment)
    ///
    /// Splits when getting 10th Mask Shard
    MaskShard10,
    /// Mask Shard 11 (Fragment)
    ///
    /// Splits when getting 11th Mask Shard
    MaskShard11,
    /// Mask Upgrade 12 (Upgrade)
    ///
    /// Splits when getting 3 extra Masks (8 base HP)
    Mask3,
    /// Mask Shard 13 (Fragment)
    ///
    /// Splits when getting 13th Mask Shard
    MaskShard13,
    /// Mask Shard 14 (Fragment)
    ///
    /// Splits when getting 14th Mask Shard
    MaskShard14,
    /// Mask Shard 15 (Fragment)
    ///
    /// Splits when getting 15th Mask Shard
    MaskShard15,
    /// Mask Upgrade 16 (Upgrade)
    ///
    /// Splits when getting 4 extra Masks (9 base HP)
    Mask4,
    /// Mask Shard 17 (Fragment)
    ///
    /// Splits when getting 17th Mask Shard
    MaskShard17,
    /// Mask Shard 18 (Fragment)
    ///
    /// Splits when getting 18th Mask Shard
    MaskShard18,
    /// Mask Shard 19 (Fragment)
    ///
    /// Splits when getting 19th Mask Shard
    MaskShard19,
    /// Mask Upgrade 20 (Upgrade)
    ///
    /// Splits when getting 5 extra Masks (10 base HP)
    Mask5,
    // endregion: MaskShards

    // region: SpoolFragments
    /// Spool Fragment 1 (Fragment)
    ///
    /// Splits when getting 1st Spool Fragment
    SpoolFragment1,
    /// Spool Upgrade 2 (Upgrade)
    ///
    /// Splits when getting 1 extra Spool Extension (10 base silk)
    Spool1,
    /// Spool Fragment 3 (Fragment)
    ///
    /// Splits when getting 3rd Spool Fragment
    SpoolFragment3,
    /// Spool Upgrade 4 (Upgrade)
    ///
    /// Splits when getting 2 extra Spool Extension (11 base silk)
    Spool2,
    /// Spool Fragment 5 (Fragment)
    ///
    /// Splits when getting 5th Spool Fragment
    SpoolFragment5,
    /// Spool Upgrade 6 (Upgrade)
    ///
    /// Splits when getting 3 extra Spool Extension (12 base silk)
    Spool3,
    /// Spool Fragment 7 (Fragment)
    ///
    /// Splits when getting 7th Spool Fragment
    SpoolFragment7,
    /// Spool Upgrade 8 (Upgrade)
    ///
    /// Splits when getting 4 extra Spool Extension (13 base silk)
    Spool4,
    /// Spool Fragment 9 (Fragment)
    ///
    /// Splits when getting 9th Spool Fragment
    SpoolFragment9,
    /// Spool Upgrade 10 (Upgrade)
    ///
    /// Splits when getting 5 extra Spool Extension (14 base silk)
    Spool5,
    /// Spool Fragment 11 (Fragment)
    ///
    /// Splits when getting 11th Spool Fragment
    SpoolFragment11,
    /// Spool Upgrade 12 (Upgrade)
    ///
    /// Splits when getting 6 extra Spool Extension (15 base silk)
    Spool6,
    /// Spool Fragment 13 (Fragment)
    ///
    /// Splits when getting 13th Spool Fragment
    SpoolFragment13,
    /// Spool Upgrade 14 (Upgrade)
    ///
    /// Splits when getting 7 extra Spool Extension (16 base silk)
    Spool7,
    /// Spool Fragment 15 (Fragment)
    ///
    /// Splits when getting 15th Spool Fragment
    SpoolFragment15,
    /// Spool Upgrade 16 (Upgrade)
    ///
    /// Splits when getting 8 extra Spool Extension (17 base silk)
    Spool8,
    /// Spool Fragment 17 (Fragment)
    ///
    /// Splits when getting 17th Spool Fragment
    SpoolFragment17,
    /// Spool Upgrade 18 (Upgrade)
    ///
    /// Splits when getting 9 extra Spool Extension (18 base silk)
    Spool9,
    // endregion SpoolFragments

    // region: ToolPouchLevels
    /// Tool Pouch Level 1 (Upgrade)
    ///
    /// Splits when getting the 1st tool pouch capacity upgrade
    ToolPouch1,
    /// Tool Pouch Level 2 (Upgrade)
    ///
    /// Splits when getting the 2nd tool pouch capacity upgrade
    ToolPouch2,
    /// Tool Pouch Level 3 (Upgrade)
    ///
    /// Splits when getting the 3rd tool pouch capacity upgrade
    ToolPouch3,
    /// Tool Pouch Level 4 (Upgrade)
    ///
    /// Splits when getting the 4th tool pouch capacity upgrade
    ToolPouch4,
    // endregion: ToolPouchLevels

    // region: CraftingKitLevels
    /// Crafting Kit Level 1 (Upgrade)
    ///
    /// Splits when getting the 1st crafting kit damage upgrade
    CraftingKit1,
    /// Crafting Kit Level 2 (Upgrade)
    ///
    /// Splits when getting the 2nd crafting kit damage upgrade
    CraftingKit2,
    /// Crafting Kit Level 3 (Upgrade)
    ///
    /// Splits when getting the 3rd crafting kit damage upgrade
    CraftingKit3,
    /// Crafting Kit Level 4 (Upgrade)
    ///
    /// Splits when getting the 4th crafting kit damage upgrade
    CraftingKit4,
    // endregion: CraftingKitLevels

    // region: Crests
    /// Reaper Crest (Crest)
    ///
    /// Splits when the Reaper Crest is unlocked
    ReaperCrest,
    /// Reaper Crest (Transition)
    ///
    /// Splits when leaving the church with the Reaper Crest unlocked
    ReaperCrestTrans,
    /// Wanderer Crest (Crest)
    ///
    /// Splits when the Wanderer Crest is unlocked
    WandererCrest,
    /// Wanderer Crest (Transition)
    ///
    /// Splits when leaving the chapel with the Wanderer Crest unlocked
    WandererCrestTrans,
    /// Beast Crest (Crest)
    ///
    /// Splits when the Beast Crest is unlocked
    BeastCrest,
    /// Beast Crest (Transition)
    ///
    /// Splits when leaving the room with the Beast Crest unlocked
    BeastCrestTrans,
    /// Architect Crest (Crest)
    ///
    /// Splits when the Architect Crest is unlocked
    ArchitectCrest,
    /// Architect Crest (Transition)
    ///
    /// Splits when leaving the room with the Architect Crest unlocked
    ArchitectCrestTrans,
    /// Curse Crest (Crest)
    ///
    /// Splits when the Curse crest is applied
    CurseCrest,
    /// Gained Curse (Event)
    ///
    /// Splits when Hornet first gains control after breaking out of the Curse Crest tree
    GainedCurse,
    /// Witch Crest (Crest)
    ///
    /// Splits when the Cursed Crest is transformed into the Witch Crest
    WitchCrest,
    /// Witch Crest (Transition)
    ///
    /// Splits when leaving Yarnaby's room after the Witch Crest is obtained
    WitchCrestTrans,
    /// Shaman Crest (Crest)
    ///
    /// Splits when the Shaman Crest is unlocked
    ShamanCrest,
    /// Shaman Crest (Transition)
    ///
    /// Splits when leaving the room with the Shaman Crest unlocked
    ShamanCrestTrans,
    /// Hunter Crest Evolution 1 (Upgrade)
    ///
    /// Splits when the first upgrade to the Hunter Crest is obtained
    HunterCrestEvo1,
    /// Vesticrest Slot 1 (Upgrade)
    ///
    /// Splits when obtaining the Vesticrest yellow tool slot
    VesticrestYellowSlot,
    /// Vesticrest Slot 2 (Upgrade)
    ///
    /// Splits when obtaining the Vesticrest blue tool slot
    VesticrestBlueSlot,
    /// Hunter Crest Evolution 2 (Upgrade)
    ///
    /// Splits when the second upgrade to the Hunter Crest is obtained
    HunterCrestEvo2,
    /// Sylphsong (Skill)
    ///
    /// Splits when obtaining Sylphsong after binding Eva
    Sylphsong,
    /// Sylphsong (Transition)
    ///
    /// Splits when leaving the room after obtaining Sylphsong
    SylphsongTrans,
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
    /// Rescued Flea Underworks Cauldron (Flea)
    ///
    /// Splits after rescuing flea in Under_21
    #[alias = "SavedFleaUnderworksExplosions"]
    SavedFleaUnderworksCauldron,
    /// Rescued Flea Underworks Wisp Thicket (Flea)
    ///
    /// Splits after rescuing flea in Under_23
    SavedFleaUnderworksWispThicket,
    /// Rescued Giant Flea (Flea)
    ///
    /// Splits after defeating Giant Flea
    SavedFleaGiantFlea,
    /// Rescued Vog (Flea)
    ///
    /// Splits after talking to Vog
    SavedFleaVog,
    /// Rescued Kratt (Flea)
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
    /// Slab / Mount Fay (Bellway)
    ///
    /// Splits after unlocking Slab / Mount Fay Bellway
    #[alias = "MountFayStation"]
    SlabStation,
    /// Bilewater (Bellway)
    ///
    /// Splits after unlocking Bilewater Bellway
    BilewaterStation,
    /// Shellwood (Bellway)
    ///
    /// Splits after unlocking Shellwood Bellway
    ShellwoodStation,
    /// Bellway Travel (Transition)
    ///
    /// Splits when riding the Bell Beast through the Bellways
    BellwayTrans,
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
    #[alias = "CityBellwayTube"]
    GrandBellwayTube,
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
    /// Ventrica Travel (Transition)
    ///
    /// Splits when traveling to another room using the Ventrica system
    VentricaTrans,
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
    /// Shakra Map Buyout (NPC)
    ///
    /// Splits after purchasing all of Shakra's maps
    ShakraMapBuyout,
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
    /// Enter Bell Eater (Transition)
    ///
    /// Splits when entering the Bell Eater's arena
    EnterBellEater,
    /// Beastling Call (Skill)
    ///
    /// Splits when obtaining Beastling Call
    BeastlingCall,
    /// Silk Soar (Skill)
    ///
    /// Splits when you obtain Super Jump
    SilkSoar,
    /// Elegy of the Deep (Skill)
    ///
    /// Splits when obtaining Elegy of the Deep
    ElegyOfTheDeep,
    /// Enter Nyleth Memory (Transition)
    ///
    /// Splits when entering Nyleth's memory
    EnterNylethMemory,
    /// Nyleth Encountered (Boss)
    ///
    /// Splits when Nyleth is encountered for the first time
    NylethEncountered,
    /// Nyleth (Boss)
    ///
    /// Splits when Nyleth is defeated
    Nyleth,
    /// Nyleth's Heart (Item)
    ///
    /// Splits when you obtain Nyleth's Heart
    #[alias = "CollectedHeartNyleth"]
    HeartNyleth,
    /// Enter Khann Memory
    ///
    /// Splits when entering Khann's Coral Tower memory
    EnterKhannMemory,
    /// Khann Encountered (Boss)
    ///
    /// Splits when Khann is first met at the top of the Coral Tower memory
    KhannEncountered,
    /// Crust King Khann (Boss)
    ///
    /// Splits when Crust King Khann is defeated
    CrustKingKhann,
    /// Khann's Heart (Item)
    ///
    /// Splits when you obtain Khann's Heart
    #[alias = "CollectedHeartKhann"]
    HeartKhann,
    /// Enter Karmelita Memory (Transition)
    ///
    /// Splits when entering Karmelita's memory
    EnterKarmelitaMemory,
    /// Skarrsinger Karmelita (Boss)
    ///
    /// Splits when Skarrsinger Karmelita is defeated
    SkarrsingerKarmelita,
    /// Karmelita's Heart (Item)
    ///
    /// Splits when you obtain Karmelita's Heart
    #[alias = "CollectedHeartKarmelita"]
    HeartKarmelita,
    /// Enter Verdania Memory (Transition)
    ///
    /// Splits when entering the Verdania Memory
    EnterVerdaniaMemory,
    /// Enter Verdania Castle (Transition)
    ///
    /// Splits when entering the room containing the Clover Dancers boss
    EnterVerdaniaCastle,
    /// Palestag (Boss)
    ///
    /// Splits when the Palestag boss is defeated
    Palestag,
    /// Clover Dancers Encountered (Boss)
    ///
    /// Splits when the Clover Dancers boss fight begins for the first time
    CloverDancersEncountered,
    /// Clover Dancers (Boss)
    ///
    /// Splits when the Clover Dancers boss is defeated
    CloverDancers,
    /// Clover Dancer's Heart (Item)
    ///
    /// Split when you obtain Conjoined Heart
    #[alias = "CollectedHeartClover"]
    HeartClover,
    /// Enter Red Memory (Transition)
    ///
    /// Splits when beginning the Red Memory sequence
    EnterRedMemory,
    /// Red Memory (Event)
    ///
    /// Splits on completing Red Memory
    #[alias = "CompletedRedMemory"]
    RedMemory,
    /// Pavo Bellhome Key (NPC)
    ///
    /// Splits when obtaining Bellhome Key from Pavo
    BellhouseKeyConversation,
    /// Verdania Lake Fountain Orbs (Event)
    ///
    /// Splits when the orbs appear after activating the shrine in the Verdania lake
    VerdaniaLakeFountainOrbs,
    /// Verdania Orbs (Event)
    ///
    /// Splits when you reach the required number of Verdania Orbs
    VerdaniaOrbsCollected,
    /// Forebrothers (Boss)
    ///
    /// Splits after defeating the Forebrothers
    Forebrothers,
    /// Groal (Boss)
    ///
    /// Splits after defeating Groal
    Groal,
    ///  Savage Beastfly 1 (Boss)
    ///
    /// Splits after defeating the Beastfly in the Chapel
    SavageBeastfly1,
    /// Savage Beastfly 2 (Boss)
    ///
    /// Splits after completing the Beastfly in Far Fields
    SavageBeastfly2,
    /// Caravan Troupe Greymoor (Event)
    ///
    /// Splits when the Caravan Troupe moves to Greymoor
    CaravanTroupeGreymoor,
    /// Caravan Troupe Blasted Steps (Event)
    ///
    /// Splits when the Caravan Troupe moves to Blasted Steps
    CaravanTroupeBlastedSteps,
    /// Caravan Troupe Fleatopia (Event)
    ///
    /// Splits when the Caravan Troupe moves to Fleatopia
    CaravanTroupeFleatopia,
    /// Scrounge Relic Sold (Event)
    ///
    /// Splits after selling the first relic to Scrounge
    SoldRelic,
    /// Pavo Time Passed (Event)
    ///
    /// Splits after meeting the Belltown Greeter and time has passed
    PavoTimePassed,
    /// Songclave Bell (Event)
    ///
    /// Splits when ringing the Songclave Bell Shrine
    SongclaveBell,
    /// Skull Tyrant (Boss)
    ///
    /// Splits after defeating the Skull Tyrant
    SkullTyrant1,
    /// Sherma Returned (NPC)
    ///
    /// Splits when Sherma is rescued and time passes
    ShermaReturned,
    /// Jubilana Rescued Memorium (Event)
    ///
    /// Splits after saving the Jubilana in Memorium
    JubilanaRescuedMemorium,
    /// Jubilana Rescued Choral Chambers (Event)
    ///
    /// Splits after saving the Jubilana in Choral Chambers
    JubilanaRescuedChoralChambers,
    /// Silk and Soul Offered (Event)
    ///
    /// Splits when the Caretaker offers the Silk and Soul quest
    SilkAndSoulOffered,
    /// Soul Snare Ready (Event)
    ///
    /// Splits when the Soul Snare becomes ready
    SoulSnareReady,
    /// Enter Destroyed Cogworks (Transition)
    ///
    /// Splits when entering the destroyed portion of Cogworks
    EnterDestroyedCogworks,
    /// Destroyed Cogworks Void Arena (Mini Boss)
    ///
    /// Splits when completing the Destroyed Cogworks void arena
    DestroyedCogworksVoidArena,
    /// Enter Seth (Transition)
    ///
    /// Splits when entering Seth's boss arena
    EnterSeth,
    /// Seth (Boss)
    ///
    /// Splits after defeating Seth
    Seth,
    /// Ballow Moved (Event)
    ///
    /// Splits when Ballow moves to the Diving Bell
    BallowMoved,
    /// Act 3 Start (Event)
    ///
    /// Splits upon entering Act 3
    Act3Started,
    // endregion: Misc TE

    // region: Abyss
    /// Diving Bell Abyss Entry (Transition)
    ///
    /// Splits when the Diving Bell enters the Abyss
    DivingBellAbyssTrans,
    /// Enter Abyss (Transition)
    ///
    /// Splits when entering the Abyss in a room with area text
    EnterAbyss,
    /// Completed Abyss Escape (Event)
    ///
    /// Splits after completing the Abyss Escape
    AbyssEscape,
    /// Last Dive (Transition)
    ///
    /// Splits when beginning the Last Dive scene
    LastDiveTrans,
    /// Lost Lace Encountered (Boss)
    ///
    /// Splits when Lost Lace is encountered in the Abyss Cocoon
    LostLaceEncountered,
    // endregion: Abyss

    // region: Tools
    /// Ascendant's Grip (Tool)
    ///
    /// Splits when obtaining the Ascendant's Grip
    AscendantsGrip,
    /// Barbed Bracelet (Tool)
    ///
    /// Splits when obtaining the Barbed Bracelet
    BarbedBracelet,
    /// Claw Mirror (Tool)
    ///
    /// Splits when obtaining the Claw Mirror
    ClawMirror,
    /// Claw Mirrors (Tool)
    ///
    /// Splits when upgrading the Claw Mirror to Claw Mirrors
    ClawMirrors,
    /// Cogfly (Tool)
    ///
    /// Splits when obtaining the Cogfly
    Cogfly,
    /// Cogwork Wheel (Tool)
    ///
    /// Splits when obtaining the Cogwork Wheel
    CogworkWheel,
    /// Compass (Tool)
    ///
    /// Splits when obtaining the Compass
    Compass,
    /// Conchcutter (Tool)
    ///
    /// Splits when obtaining the Conchcutter
    Conchcutter,
    /// Curveclaw (Tool)
    ///
    /// Splits when obtaining the Curveclaw
    Curveclaw,
    /// Curvesickle (Tool)
    ///
    /// Splits when upgrading the Curveclaw to the Curvesickle
    Curvesickle,
    /// Dead Bug's Purse (Tool)
    ///
    /// Splits when obtaining the Dead Bug's Purse
    DeadBugsPurse,
    /// Delver's Drill (Tool)
    ///
    /// Splits when obtaining the Delver's Drill
    DelversDrill,
    /// Druid's Eye (Tool)
    ///
    /// Splits when obtaining the Druid's Eye
    DruidsEye,
    /// Druid's Eyes (Tool)
    ///
    /// Splits when upgrading the Druid's Eye to Druid's Eyes
    DruidsEyes,
    /// Egg of Flealia (Tool)
    ///
    /// Splits when obtaining the Egg of Flealia
    EggofFlealia,
    /// Flea Brew (Tool)
    ///
    /// Splits when obtaining the Flea Brew
    FleaBrew,
    /// Flintslate (Tool)
    ///
    /// Splits when obtaining the Flintslate
    Flintslate,
    /// Fractured Mask (Tool)
    ///
    /// Splits when obtaining the Fractured Mask
    FracturedMask,
    /// Injector Band (Tool)
    ///
    /// Splits when obtaining the Injector Band
    InjectorBand,
    /// Longclaw (Tool)
    ///
    /// Splits when obtaining the Longclaw
    Longclaw,
    /// Longpin (Tool)
    ///
    /// Splits when obtaining the Longpin
    Longpin,
    /// Magma Bell (Tool)
    ///
    /// Splits when obtaining the Magma Bell
    MagmaBell,
    /// Magnetite Brooch (Tool)
    ///
    /// Splits when obtaining the Magnetite Brooch
    MagnetiteBrooch,
    /// Magnetite Dice (Tool)
    ///
    /// Splits when obtaining the Magnetite Dice
    MagnetiteDice,
    /// Memory Crystal (Tool)
    ///
    /// Splits when obtaining the Memory Crystal
    MemoryCrystal,
    /// Multibinder (Tool)
    ///
    /// Splits when obtaining the Multibinder
    Multibinder,
    /// Needle Phial (Tool)
    ///
    /// Splits when obtaining the Needle Phial
    NeedlePhial,
    /// Pimpillo (Tool)
    ///
    /// Splits when obtaining the Pimpillo
    Pimpillo,
    /// Pin Badge (Tool)
    ///
    /// Splits when obtaining the Pin Badge
    PinBadge,
    /// Plasmium Phial (Tool)
    ///
    /// Splits when obtaining the Plasmium Phial
    PlasmiumPhial,
    /// Pollip Pouch (Tool)
    ///
    /// Splits when obtaining the Pollip Pouch
    PollipPouch,
    /// Quick Sling (Tool)
    ///
    /// Splits when obtaining the Quick Sling
    QuickSling,
    /// Reserve Bind (Tool)
    ///
    /// Splits when obtaining the Reserve Bind
    ReserveBind,
    /// Rosary Cannon (Tool)
    ///
    /// Splits when obtaining the Rosary Cannon
    RosaryCannon,
    /// Sawtooth Circlet (Tool)
    ///
    /// Splits when obtaining the Sawtooth Circlet
    SawtoothCirclet,
    /// Scuttlebrace (Tool)
    ///
    /// Splits when obtaining the Scuttlebrace
    Scuttlebrace,
    /// Shard Pendant (Tool)
    ///
    /// Splits when obtaining the Shard Pendant
    ShardPendant,
    /// Shell Satchel (Tool)
    ///
    /// Splits when obtaining the Shell Satchel
    ShellSatchel,
    /// Silkshot Architect (Tool)
    ///
    /// Splits when obtaining the Silkshot from the Twelfth Architect
    SilkshotArchitect,
    /// Silkshot Forge (Tool)
    ///
    /// Splits when obtaining the Silkshot from the Forge Daughter
    SilkshotForge,
    /// Silkshot Weaver (Tool)
    ///
    /// Splits when obtaining the Silkshot from Mount Fay
    SilkshotWeaver,
    /// Silkspeed Anklets (Tool)
    ///
    /// Splits when obtaining the Silkspeed Anklets
    SilkspeedAnklets,
    /// Snare Setter (Tool)
    ///
    /// Splits when obtaining the Snare Setter
    SnareSetter,
    /// Snitch Pick (Tool)
    ///
    /// Splits when obtaining the Snitch Pick
    SnitchPick,
    /// Spider Strings (Tool)
    ///
    /// Splits when obtaining the Spider Strings
    SpiderStrings,
    /// Spool Extender (Tool)
    ///
    /// Splits when obtaining the Spool Extender
    SpoolExtender,
    /// Sting Shard (Tool)
    ///
    /// Splits when obtaining the Sting Shard
    StingShard,
    /// Straight Pin (Tool)
    ///
    /// Splits when obtaining the Straight Pin
    StraightPin,
    /// Tacks (Tool)
    ///
    /// Splits when obtaining the Tacks
    Tacks,
    /// Thief's Mark (Tool)
    ///
    /// Splits when obtaining the Thief's Mark
    ThiefsMark,
    /// Threefold Pin (Tool)
    ///
    /// Splits when obtaining the Threefold Pin
    ThreefoldPin,
    /// Throwing Ring (Tool)
    ///
    /// Splits when obtaining the Throwing Ring
    ThrowingRing,
    /// Volt Filament (Tool)
    ///
    /// Splits when obtaining the Volt Filament
    VoltFilament,
    /// Voltvessels (Tool)
    ///
    /// Splits when obtaining the Voltvessels
    Voltvessels,
    /// Warding Bell (Tool)
    ///
    /// Splits when obtaining the Warding Bell
    WardingBell,
    /// Weavelight (Tool)
    ///
    /// Splits when obtaining the Weavelight
    Weavelight,
    /// Weighted Belt (Tool)
    ///
    /// Splits when obtaining the Weighted Belt
    WeightedBelt,
    /// Wispfire Lantern (Tool)
    ///
    /// Splits when obtaining the Wispfire Lantern
    WispfireLantern,
    /// Wreath of Purity (Tool)
    ///
    /// Splits when obtaining the Wreath of Purity
    WreathofPurity,
    // endregion: Tools
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
    _e: &Env,
    store: &mut Store,
) -> SplitterAction {
    match split {
        // region: Start, End, and Menu
        Split::Menu => should_split(scenes.current == MENU_TITLE),
        // endregion: Start, End, and Menu

        // region: Bellhart
        Split::MenuNeedolinMemory => should_split(
            scenes.current == MENU_TITLE
                && store.get_string("respawn_scene").unwrap_or_default() == "Belltown_Shrine",
        ),
        // endregion: Bellhart

        // else
        _ => should_split(false),
    }
}

pub fn transition_splits(split: &Split, scenes: &Pair<&str>, e: &Env) -> SplitterAction {
    let Env { mem, pd, gm } = e;
    match split {
        // region: Start, End, and Menu
        Split::StartNewGame => {
            should_split(OPENING_SCENES.contains(&scenes.old) && scenes.current == "Tut_01")
        }
        Split::EndingSplit => should_split(scenes.current.starts_with("Cinematic_Ending")),
        Split::EndingA => should_split(scenes.current == "Cinematic_Ending_A"),
        Split::AnyTransition => should_split(true),
        // TODO: if there's anything like DreamGate in Silksong,
        // should TransitionExcludingDiscontinuities exclude that too?
        Split::TransitionExcludingDiscontinuities => should_split(
            !(is_discontinuity_scene(scenes.old)
                || is_discontinuity_scene(scenes.current)
                || mem.deref(&pd.health).is_ok_and(|h: i32| h == 0)),
        ),
        // endregion: Start, End, and Menu

        // region: MossLands
        Split::MossMotherTrans => {
            should_split(mem.deref(&pd.defeated_moss_mother).unwrap_or_default())
        }
        Split::SilkSpearTrans => should_split(mem.deref(&pd.has_needle_throw).unwrap_or_default()),
        Split::EnterBoneBottom => should_split(scenes.changed_to(&"Bonetown")),
        Split::EnterMosshome => {
            should_split(scenes.old == "Bone_05" && scenes.current == "Mosstown_01")
        }
        Split::EnterWeavenestAtla => {
            should_split(scenes.old == "Tut_01b" && scenes.current == "Weave_04")
        }
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
        Split::EnterUpperWormways => {
            should_split(scenes.old == "Crawl_03b" && scenes.current == "Crawl_03")
        }
        Split::SharpdartTrans => should_split(mem.deref(&pd.has_silk_charge).unwrap_or_default()),
        // endregion: Wormways

        // region: HuntersMarch
        Split::EnterHuntersMarch => should_split(
            (scenes.old == "Ant_02" && scenes.current == "Ant_03")
                || (scenes.old == "Ant_05b" && scenes.current == "Ant_14"),
        ),
        Split::HuntersMarchPostMiddleArenaTransition => {
            should_split(scenes.old == "Ant_04_mid" && scenes.current == "Ant_04")
        }
        // endregion: HuntersMarch

        // region: FarFields
        Split::EnterFarFields => should_split(
            !scenes.old.starts_with("Bone_East") && scenes.current.starts_with("Bone_East"),
        ),
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
        Split::ThreadStormTrans => {
            should_split(mem.deref(&pd.has_thread_sphere).unwrap_or_default())
        }
        Split::EnterHalfwayHomeBasement => {
            should_split(scenes.old == "Halfway_01" && scenes.current == "Ant_08")
        }
        // endregion: Greymoor

        // region: WispThicket
        Split::EnterWispThicket => {
            should_split(scenes.old == "Greymoor_06" && scenes.current == "Wisp_04")
        }
        Split::EnterFatherOfTheFlame => {
            should_split(scenes.old == "Wisp_02" && scenes.current == "Belltown_08")
        }
        // endregion: WispThicket

        // region: Bellhart
        Split::EnterBellhart => should_split(
            (scenes.old == "Belltown_06"
                || scenes.old == "Belltown_07"
                || scenes.old == "Belltown_basement")
                && scenes.current == "Belltown",
        ),
        // endregion: Bellhart

        // region: Shellwood
        Split::ClingGripTrans => should_split(mem.deref(&pd.has_wall_jump).unwrap_or_default()),
        Split::EnterShellwood => should_split(
            !scenes.old.starts_with("Shellwood") && scenes.current.starts_with("Shellwood"),
        ),
        // endregion: Shellwood

        // region: BlastedSteps
        Split::EnterBlastedSteps => {
            should_split(scenes.old == "Coral_19" && scenes.current == "Coral_02")
        }
        Split::GreatConchfliesTrans => {
            should_split(mem.deref(&pd.defeated_coral_drillers).unwrap_or_default())
        }
        Split::NeedleStrikeTrans => {
            should_split(mem.deref(&pd.has_charge_slash).unwrap_or_default())
        }
        Split::EnterLastJudge => {
            should_split(scenes.old == "Coral_32" && scenes.current == "Coral_Judge_Arena")
        }
        Split::EnterCitadelFrontGate => {
            should_split(scenes.old == "Coral_Judge_Arena" && scenes.current == "Coral_10")
        }
        // endregion: BlastedSteps

        // region: SinnersRoad
        Split::EnterSinnersRoad => should_split(
            (scenes.old == "Greymoor_03" && scenes.current == "Dust_01")
                || (scenes.old == "Dust_Maze_08_completed" && scenes.current == "Dust_05")
                || (scenes.old == "Shadow_05" && scenes.current == "Dust_06"),
        ),
        // endregion: SinnersRoad

        // region: TheMist
        Split::EnterMist => should_split(
            (scenes.old == "Dust_05" || scenes.old == "Shadow_04")
                && scenes.current == "Dust_Maze_09_entrance",
        ),
        Split::MistCrossing => should_split(
            scenes.old.starts_with("Dust_Maze_0") && scenes.current == "Dust_Maze_crossing",
        ),
        Split::LeaveMist => {
            should_split(scenes.old == "Dust_Maze_Last_Hall" && scenes.current == "Dust_09")
        }
        // endregion: TheMist

        // region: Bilewater
        Split::EnterBilewater => should_split(
            (scenes.old == "Dust_06" && scenes.current == "Shadow_05")
                || (scenes.old == "Library_07" && scenes.current == "Shadow_22")
                || (scenes.old == "Dust_09" && scenes.current == "Shadow_04")
                || (scenes.old == "Aqueduct_04" && scenes.current == "Shadow_01"),
        ),
        Split::EnterExhaustOrgan => {
            should_split(scenes.old == "Dust_09" && scenes.current == "Organ_01")
        }
        Split::PhantomTrans => should_split(mem.deref(&pd.defeated_phantom).unwrap_or_default()),
        Split::CrossStitchTrans => should_split(mem.deref(&pd.has_parry).unwrap_or_default()),
        Split::TrailsEndTrans => {
            should_split(scenes.old == "Shadow_24" && scenes.current == "Shadow_19")
        }
        // endregion: Bilewater

        // region: TheSlab
        Split::EnterTheSlab => should_split(scenes.old == "Slab_01" && scenes.current == "Slab_02"),
        Split::WardenflyCaptureTrans => {
            let gate = mem.read_string(&gm.entry_gate_name).unwrap_or_default();
            should_split(scenes.changed_to(&"Slab_03") && gate == "door_slabCaged")
        }
        Split::EnterFirstSinner => {
            should_split(scenes.old == "Slab_10c" && scenes.current == "Slab_10b")
        }
        Split::RuneRageTrans => should_split(mem.deref(&pd.has_silk_bomb).unwrap_or_default()),
        // endregion: TheSlab

        // region: MountFay
        Split::EnterMountFay => {
            should_split(scenes.old == "Slab_06" && scenes.current == "Peak_01")
        }
        Split::EnterBrightvein => {
            should_split(scenes.old == "Peak_06b" && scenes.current == "Peak_06")
        }
        Split::UpperMountFayTrans => {
            should_split(scenes.old == "Peak_01" && scenes.current == "Peak_07")
        }
        // endregion: MountFay

        // region: SandsOfKarak
        Split::EnterSandsOfKarak => should_split(
            (scenes.old == "Coral_25" && scenes.current == "Coral_23")
                || (scenes.old == "Coral_35" && scenes.current == "Coral_35b")
                || (scenes.old == "Coral_38" && scenes.current == "Coral_26"),
        ),
        Split::EnterVoltnest => {
            should_split(scenes.old == "Coral_35b" && scenes.current == "Coral_29")
        }
        Split::RagingConchflyTrans => should_split(
            mem.deref(&pd.defeated_coral_driller_solo)
                .unwrap_or_default(),
        ),
        // endregion: SandsOfKarak

        // region: ChoralChambers
        Split::EnterSongclave => should_split(
            (scenes.old == "Song_Enclave_Tube"
                || scenes.old == "Song_25"
                || scenes.old == "Library_02"
                || scenes.old == "Library_04")
                && scenes.current == "Song_Enclave",
        ),
        Split::TrobbioTrans => should_split(mem.deref(&pd.defeated_trobbio).unwrap_or_default()),
        // endregion: ChoralChambers

        // region: Underworks
        Split::PostClawlineArenaTrans => {
            let gate = mem.read_string(&gm.entry_gate_name).unwrap_or_default();
            should_split(
                gate == "bot2" && (scenes.old == "Under_18" && scenes.current == "Under_17"),
            )
        }
        // endregion: Underworks

        // region: CogworkCore
        Split::EnterCogworkDancers => should_split(
            (scenes.old == "Hang_07" || scenes.old == "Song_25") && scenes.current == "Cog_Dancers",
        ),
        Split::EnterCogworkCore => should_split(
            // main transition from dancers arena to either above or below
            (scenes.old == "Cog_Dancers"
                && (scenes.current == "Cog_04" || scenes.current == "Cog_08"))
                // other transitions into lower core
                || ((scenes.old == "Cog_05" || scenes.old == "Cog_06" || scenes.old == "Cog_07")
                && scenes.current == "Cog_04"),
        ),
        // endregion: CogworkCore

        // region: WhisperingVaults
        Split::EnterWhisperingVaults => should_split(
            (scenes.old == "Library_02" && scenes.current == "Library_01")
                || (scenes.old == "Song_Enclave" && scenes.current == "Library_04"),
        ),
        // endregion: WhisperingVaults

        // region: HighHalls
        Split::EnterHighHalls => should_split(
            (scenes.old == "Hang_01" && scenes.current == "Hang_02")
                || (scenes.old == "Hang_07" && scenes.current == "Hang_06"),
        ),
        Split::EnterHighHallsArena => {
            should_split(scenes.old == "Hang_06" && scenes.current == "Hang_04")
        }
        Split::EnterSecondSentinel => {
            should_split(scenes.old == "Hang_08" && scenes.current == "Hang_17b")
        }
        // endregion: HighHalls

        // region: Whiteward
        Split::EnterWhiteward => {
            should_split(scenes.old == "Song_05" && scenes.current == "Ward_01")
        }
        Split::PostWhitewardElevatorTrans => should_split(
            (scenes.old == "Ward_01")
                && (scenes.current == "Ward_02"
                    || scenes.current == "Ward_02b"
                    || scenes.current == "Ward_05"
                    || scenes.current == "Ward_04"
                    || scenes.current == "Ward_03"),
        ),
        // endregion: Whiteward

        // region: Memorium
        Split::EnterMemorium => {
            should_split(scenes.old == "Song_25" && scenes.current == "Arborium_01")
        }
        // endregion: Memorium

        // region: PutrifiedDucts
        Split::EnterPutrifiedDucts => should_split(
            (scenes.old == "Arborium_11" && scenes.current == "Aqueduct_01")
                || (scenes.old == "Aqueduct_04" && scenes.current == "Aqueduct_02"),
        ),
        // endregion: PutrifiedDucts

        // region: TheCradle
        Split::PostLace2ArenaTrans => {
            should_split(scenes.old == "Song_Tower_01" && scenes.current == "Tube_Hub")
        }
        Split::EnterTheCradle => {
            should_split(scenes.old == "Tube_Hub" && scenes.current == "Cradle_01")
        }
        Split::PaleNailsTrans => {
            should_split(mem.deref(&pd.has_silk_boss_needle).unwrap_or_default())
        }
        // endregion: TheCradle

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

        // region: Crests
        Split::ReaperCrestTrans => {
            should_split(mem.deref(&pd.completed_memory_reaper).unwrap_or_default())
        }
        Split::WandererCrestTrans => {
            should_split(mem.deref(&pd.completed_memory_wanderer).unwrap_or_default())
        }
        Split::BeastCrestTrans => {
            should_split(mem.deref(&pd.completed_memory_beast).unwrap_or_default())
        }
        Split::ArchitectCrestTrans => should_split(
            mem.deref(&pd.completed_memory_toolmaster)
                .unwrap_or_default(),
        ),
        Split::WitchCrestTrans => should_split(
            mem.deref(&pd.belltown_doctor_cured_curse)
                .unwrap_or_default(),
        ),
        Split::ShamanCrestTrans => {
            should_split(mem.deref(&pd.completed_memory_shaman).unwrap_or_default())
        }
        Split::SylphsongTrans => {
            should_split(mem.deref(&pd.has_bound_crest_upgrader).unwrap_or_default())
        }
        // endregion: Crests

        // region: Bellways
        Split::BellwayTrans => should_split(scenes.changed_from(&CINEMATIC_STAG_TRAVEL)),
        // endregion: Bellway

        // region: Ventrica
        Split::VentricaTrans => {
            let gate = mem.read_string(&gm.entry_gate_name).unwrap_or_default();
            should_split(gate == "door_tubeEnter" && scenes.changed())
        }
        // endregion: Ventrica

        // region: MiscTE
        Split::EnterBellEater => should_split(scenes.changed_to(&"Bellway_Centipede_Arena")),
        Split::EnterDestroyedCogworks => should_split(
            (scenes.old == "Song_Tower_Destroyed" && scenes.current == "Cog_09_Destroyed")
                || (scenes.old == "Song_25" && scenes.current == "Cog_10_Destroyed"),
        ),
        Split::EnterSeth => {
            should_split(scenes.old == "Under_27" && scenes.current == "Shellwood_22")
        }
        Split::EnterNylethMemory => {
            should_split(scenes.old == "Shellwood_11b" && scenes.current == "Shellwood_11b_Memory")
        }
        Split::EnterKarmelitaMemory => {
            should_split(scenes.old == "Ant_Queen" && scenes.current == "Memory_Ant_Queen")
        }
        Split::EnterVerdaniaMemory => {
            should_split(scenes.old == "Clover_01" && scenes.current == "Clover_01b")
        }
        Split::EnterVerdaniaCastle => {
            should_split(scenes.old == "Clover_04b" && scenes.current == "Clover_10")
        }
        Split::EnterKhannMemory => {
            should_split(scenes.old == "Coral_Tower_01" && scenes.current == "Memory_Coral_Tower")
        }
        Split::EnterRedMemory => {
            should_split(scenes.old == "Tut_04" && scenes.current == "Memory_Red")
        }
        // endregion: MiscTE

        // region: Abyss
        Split::DivingBellAbyssTrans => should_split(
            scenes.old == "Room_Diving_Bell"
                && scenes.current.starts_with("Room_Diving_Bell_Abyss"),
        ),
        Split::EnterAbyss => should_split(
            (scenes.old == "Abyss_03" && scenes.current == "Abyss_02")
                || (scenes.old == "Abyss_11" && scenes.current == "Abyss_02b"),
        ),
        Split::LastDiveTrans => {
            should_split(scenes.old == "Abyss_05" && scenes.current == "Last_Dive")
        }
        // endregion: Abyss

        // else
        _ => should_split(false),
    }
}

pub fn transition_once_splits(split: &Split, scenes: &Pair<&str>, e: &Env) -> SplitterAction {
    let Env { mem, gm, pd } = e;
    match split {
        // region: Start, End, and Menu
        Split::Act1Start => should_split(
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

fn mask_shard_split(e: &Env, shard: i32) -> bool {
    let Env { mem, pd, .. } = e;
    const START_MASKS: i32 = 5;
    let current_shards = shard % 4;
    let additional_masks = shard / 4;

    (mem.deref(&pd.max_health_base)
        .is_ok_and(|n: i32| START_MASKS + additional_masks == n))
        && (mem
            .deref(&pd.heart_pieces)
            .is_ok_and(|n: i32| n == current_shards))
}

fn spool_shard_split(e: &Env, shard: i32) -> bool {
    let Env { mem, pd, .. } = e;
    const START_SPOOLS: i32 = 9;
    let current_shards = shard % 2;
    let additional_spools = shard / 2;

    (mem.deref(&pd.silk_max)
        .is_ok_and(|n: i32| START_SPOOLS + additional_spools == n))
        && (mem
            .deref(&pd.silk_spool_parts)
            .is_ok_and(|n: i32| n == current_shards))
}

pub fn continuous_splits(split: &Split, e: &Env, store: &mut Store) -> SplitterAction {
    let Env { mem, gm, pd } = e;
    let game_state: i32 = mem.deref(&gm.game_state).unwrap_or_default();
    if !NON_MENU_GAME_STATES.contains(&game_state) {
        return should_split(false);
    }
    match split {
        // region: Start, End, and Menu
        Split::ManualSplit => SplitterAction::ManualSplit,
        Split::BenchAny => should_split(
            store
                .get_bool_pair_bang("at_bench", &get_at_bench, Some(e))
                .is_some_and(|p| p.changed_to(&true)),
        ),
        Split::PlayerDeath => should_split(
            store
                .get_i32_pair_bang("health", &get_health, Some(e))
                .is_some_and(|p| p.changed_to(&0)),
        ),
        // endregion: Start, End, and Menu

        // region: MossLands
        Split::MossMother => should_split(mem.deref(&pd.defeated_moss_mother).unwrap_or_default()),
        Split::SilkSpear => should_split(mem.deref(&pd.has_needle_throw).unwrap_or_default()),
        Split::BoneBottomSimpleKey => {
            should_split(mem.deref(&pd.has_bonebottom_simple_key).unwrap_or_default())
        }
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

        // region: Wormways
        Split::Sharpdart => should_split(mem.deref(&pd.has_silk_charge).unwrap_or_default()),
        // endregion: Wormways

        // region: FarFields
        Split::DriftersCloak => should_split(mem.deref(&pd.has_brolly).unwrap_or_default()),
        Split::FourthChorus => should_split(mem.deref(&pd.defeated_song_golem).unwrap_or_default()),
        Split::GurrTheOutcastEncountered => {
            should_split(mem.deref(&pd.encountered_ant_trapper).unwrap_or_default())
        }
        Split::GurrTheOutcast => {
            should_split(mem.deref(&pd.defeated_ant_trapper).unwrap_or_default())
        }
        // endregion: FarFields

        // region: Greymoor
        Split::GreymoorBell => {
            should_split(mem.deref(&pd.bell_shrine_greymoor).unwrap_or_default())
        }
        Split::Moorwing => should_split(
            mem.deref(&pd.defeated_vampire_gnat_boss)
                .unwrap_or_default(),
        ),
        Split::ThreadStorm => should_split(mem.deref(&pd.has_thread_sphere).unwrap_or_default()),
        Split::YarnabySlap => {
            let convo_level: i32 = mem.deref(&pd.belltown_doctor_convo).unwrap_or_default();
            should_split(convo_level == 3)
        }
        // endregion: Greymoor

        // region: WispThicket
        Split::FatherOfTheFlame => {
            should_split(mem.deref(&pd.defeated_wisp_pyre_effigy).unwrap_or_default())
        }
        // endregion: WispThicket

        // region: Shellwood
        Split::ClingGrip => should_split(mem.deref(&pd.has_wall_jump).unwrap_or_default()),
        Split::ShellwoodBell => {
            should_split(mem.deref(&pd.bell_shrine_shellwood).unwrap_or_default())
        }
        // endregion: Shellwood

        // region: Bellhart
        Split::WidowEncountered => {
            should_split(mem.deref(&pd.encountered_spinner).unwrap_or_default())
        }
        Split::Widow => should_split(mem.deref(&pd.spinner_defeated).unwrap_or_default()),
        Split::MenuNeedolinMemory => {
            store.get_string_bang("respawn_scene", &get_respawn_scene, Some(e));
            should_split(false)
        }
        Split::BellhartBell => {
            should_split(mem.deref(&pd.bell_shrine_bellhart).unwrap_or_default())
        }
        // endregion: Bellhart

        // region: BlastedSteps
        Split::NeedleStrike => should_split(mem.deref(&pd.has_charge_slash).unwrap_or_default()),
        Split::GreatConchflies => {
            should_split(mem.deref(&pd.defeated_coral_drillers).unwrap_or_default())
        }
        Split::LastJudgeEncountered => {
            should_split(mem.deref(&pd.encountered_last_judge).unwrap_or_default())
        }
        Split::LastJudge => should_split(mem.deref(&pd.defeated_last_judge).unwrap_or_default()),
        // endregion: BlastedSteps

        // region: SinnersRoad
        Split::Lugoli => should_split(
            mem.deref(&pd.defeated_roach_keeper_chef)
                .unwrap_or_default(),
        ),
        // endregion: SinnersRoad

        // region: Bilewater
        Split::Phantom => should_split(mem.deref(&pd.defeated_phantom).unwrap_or_default()),
        Split::CrossStitch => should_split(mem.deref(&pd.has_parry).unwrap_or_default()),
        // endregion: Bilewater

        // region: TheSlab
        Split::SlabKeyIndolent => should_split(mem.deref(&pd.has_slab_key_a).unwrap_or_default()),
        Split::SlabKeyHeretic => should_split(mem.deref(&pd.has_slab_key_b).unwrap_or_default()),
        Split::SlabKeyApostate => should_split(mem.deref(&pd.has_slab_key_c).unwrap_or_default()),
        Split::Broodmother => {
            should_split(mem.deref(&pd.defeated_brood_mother).unwrap_or_default())
        }
        Split::FirstSinnerEncountered => {
            should_split(mem.deref(&pd.encountered_first_weaver).unwrap_or_default())
        }
        Split::FirstSinner => {
            should_split(mem.deref(&pd.defeated_first_weaver).unwrap_or_default())
        }
        Split::RuneRage => should_split(mem.deref(&pd.has_silk_bomb).unwrap_or_default()),
        // endregion: TheSlab

        // region: MountFay
        Split::FaydownCloak => should_split(mem.deref(&pd.has_double_jump).unwrap_or_default()),
        // endregion: MountFay

        // region: SandsOfKarak
        Split::RagingConchfly => should_split(
            mem.deref(&pd.defeated_coral_driller_solo)
                .unwrap_or_default(),
        ),
        Split::Voltvyrm => should_split(mem.deref(&pd.defeated_zap_core_enemy).unwrap_or_default()),
        Split::WatcherAtTheEdge => {
            should_split(mem.deref(&pd.defeated_grey_warrior).unwrap_or_default())
        }
        // endregion: SandsOfKarak

        // region: Acts
        Split::Act2Started => should_split(mem.deref(&pd.act2_started).unwrap_or_default()),
        // endregion: Acts

        // region: CogworkCore
        Split::CogworkDancersEncountered => should_split(
            mem.deref(&pd.encountered_cogwork_dancers)
                .unwrap_or_default(),
        ),
        Split::CogworkDancers => {
            should_split(mem.deref(&pd.defeated_cogwork_dancers).unwrap_or_default())
        }
        Split::SecondSentinelAwoken => {
            should_split(mem.deref(&pd.woke_song_chevalier).unwrap_or_default())
        }
        // endregion: CogworkCore

        // region: WhisperingVaults
        Split::WhisperingVaultsArena => should_split(
            mem.deref(&pd.completed_library_entry_battle)
                .unwrap_or_default(),
        ),
        // endregion: WhisperingVaults

        // region: ChoralChambers
        Split::MetMergwin => should_split(mem.deref(&pd.met_gourmand_servant).unwrap_or_default()),
        Split::DeliveredCouriersRasher => {
            should_split(mem.deref(&pd.gourmand_given_meat).unwrap_or_default())
        }
        Split::GreatTasteReward => {
            should_split(mem.deref(&pd.got_gourmand_reward).unwrap_or_default())
        }
        Split::Trobbio => should_split(mem.deref(&pd.defeated_trobbio).unwrap_or_default()),
        //endregion: ChoralChambers

        // region: Underworks
        Split::Clawline => should_split(mem.deref(&pd.has_harpoon_dash).unwrap_or_default()),
        //endregion: Underworks

        // region: HighHalls
        Split::HighHallsArena => should_split(mem.deref(&pd.hang04_battle).unwrap_or_default()),
        Split::SecondSentinelBossEncountered => should_split(
            mem.deref(&pd.encountered_song_chevalier_boss)
                .unwrap_or_default(),
        ),
        Split::SecondSentinel => should_split(
            mem.deref(&pd.defeated_song_chevalier_boss)
                .unwrap_or_default(),
        ),
        //endregion: HighHalls

        // region: Whiteward
        Split::CollectedWhitewardKey => {
            should_split(mem.deref(&pd.collected_ward_key).unwrap_or_default())
        }
        Split::CollectedSurgeonsKey => {
            should_split(mem.deref(&pd.collected_ward_boss_key).unwrap_or_default())
        }
        Split::TheUnravelledEncountered => {
            should_split(mem.deref(&pd.ward_boss_encountered).unwrap_or_default())
        }
        Split::TheUnravelled => should_split(mem.deref(&pd.ward_boss_defeated).unwrap_or_default()),
        // endregion: Whiteward

        // region: PutrifiedDucts
        Split::FleaFestivalBegin => {
            should_split(mem.deref(&pd.flea_games_started).unwrap_or_default())
        }
        Split::FleaFestivalEnd => should_split(mem.deref(&pd.flea_games_ended).unwrap_or_default()),
        // endregion: PutrifiedDucts

        // region: TheCradle
        Split::Lace2 => should_split(mem.deref(&pd.defeated_lace_tower).unwrap_or_default()),
        Split::PaleNails => should_split(mem.deref(&pd.has_silk_boss_needle).unwrap_or_default()),
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

        // region: MaskShards
        Split::OnObtainMaskShard => {
            let max_hp_increased = store
                .get_i32_pair_bang("max_health_base", &get_max_health_base, Some(e))
                .is_some_and(|p| p.increased());
            let shards_increased = store
                .get_i32_pair_bang("heart_pieces", &get_heart_pieces, Some(e))
                // sanity check: mask shards should go from 3 to 0 on mask completion
                .is_some_and(|p| p.increased() && p.current < 4);
            should_split(max_hp_increased || shards_increased)
        }
        Split::MaskShard1 => should_split(mask_shard_split(e, 1)),
        Split::MaskShard2 => should_split(mask_shard_split(e, 2)),
        Split::MaskShard3 => should_split(mask_shard_split(e, 3)),
        Split::Mask1 => should_split(mask_shard_split(e, 4)),
        Split::MaskShard5 => should_split(mask_shard_split(e, 5)),
        Split::MaskShard6 => should_split(mask_shard_split(e, 6)),
        Split::MaskShard7 => should_split(mask_shard_split(e, 7)),
        Split::Mask2 => should_split(mask_shard_split(e, 8)),
        Split::MaskShard9 => should_split(mask_shard_split(e, 9)),
        Split::MaskShard10 => should_split(mask_shard_split(e, 10)),
        Split::MaskShard11 => should_split(mask_shard_split(e, 11)),
        Split::Mask3 => should_split(mask_shard_split(e, 12)),
        Split::MaskShard13 => should_split(mask_shard_split(e, 13)),
        Split::MaskShard14 => should_split(mask_shard_split(e, 14)),
        Split::MaskShard15 => should_split(mask_shard_split(e, 15)),
        Split::Mask4 => should_split(mask_shard_split(e, 16)),
        Split::MaskShard17 => should_split(mask_shard_split(e, 17)),
        Split::MaskShard18 => should_split(mask_shard_split(e, 18)),
        Split::MaskShard19 => should_split(mask_shard_split(e, 19)),
        Split::Mask5 => should_split(mask_shard_split(e, 20)),
        // endregion: MaskShards

        // region: SpoolFragments
        Split::SpoolFragment1 => should_split(spool_shard_split(e, 1)),
        Split::Spool1 => should_split(spool_shard_split(e, 2)),
        Split::SpoolFragment3 => should_split(spool_shard_split(e, 3)),
        Split::Spool2 => should_split(spool_shard_split(e, 4)),
        Split::SpoolFragment5 => should_split(spool_shard_split(e, 5)),
        Split::Spool3 => should_split(spool_shard_split(e, 6)),
        Split::SpoolFragment7 => should_split(spool_shard_split(e, 7)),
        Split::Spool4 => should_split(spool_shard_split(e, 8)),
        Split::SpoolFragment9 => should_split(spool_shard_split(e, 9)),
        Split::Spool5 => should_split(spool_shard_split(e, 10)),
        Split::SpoolFragment11 => should_split(spool_shard_split(e, 11)),
        Split::Spool6 => should_split(spool_shard_split(e, 12)),
        Split::SpoolFragment13 => should_split(spool_shard_split(e, 13)),
        Split::Spool7 => should_split(spool_shard_split(e, 14)),
        Split::SpoolFragment15 => should_split(spool_shard_split(e, 15)),
        Split::Spool8 => should_split(spool_shard_split(e, 16)),
        Split::SpoolFragment17 => should_split(spool_shard_split(e, 17)),
        Split::Spool9 => should_split(spool_shard_split(e, 18)),
        // endregion SpoolFragments

        // region: ToolPouchLevels
        Split::ToolPouch1 => should_split(
            mem.deref(&pd.tool_pouch_upgrades)
                .is_ok_and(|n: i32| n == 1),
        ),
        Split::ToolPouch2 => should_split(
            mem.deref(&pd.tool_pouch_upgrades)
                .is_ok_and(|n: i32| n == 2),
        ),
        Split::ToolPouch3 => should_split(
            mem.deref(&pd.tool_pouch_upgrades)
                .is_ok_and(|n: i32| n == 3),
        ),
        Split::ToolPouch4 => should_split(
            mem.deref(&pd.tool_pouch_upgrades)
                .is_ok_and(|n: i32| n == 4),
        ),
        // endregion: ToolPouchLevels

        // region: CraftingKitLevels
        Split::CraftingKit1 => {
            should_split(mem.deref(&pd.tool_kit_upgrades).is_ok_and(|n: i32| n == 1))
        }
        Split::CraftingKit2 => {
            should_split(mem.deref(&pd.tool_kit_upgrades).is_ok_and(|n: i32| n == 2))
        }
        Split::CraftingKit3 => {
            should_split(mem.deref(&pd.tool_kit_upgrades).is_ok_and(|n: i32| n == 3))
        }
        Split::CraftingKit4 => {
            should_split(mem.deref(&pd.tool_kit_upgrades).is_ok_and(|n: i32| n == 4))
        }
        // endregion: CraftingKitLevels

        // region: Crests
        Split::ReaperCrest => {
            should_split(mem.deref(&pd.completed_memory_reaper).unwrap_or_default())
        }
        Split::WandererCrest => {
            should_split(mem.deref(&pd.completed_memory_wanderer).unwrap_or_default())
        }
        Split::BeastCrest => {
            should_split(mem.deref(&pd.completed_memory_beast).unwrap_or_default())
        }
        Split::ArchitectCrest => should_split(
            mem.deref(&pd.completed_memory_toolmaster)
                .unwrap_or_default(),
        ),
        Split::CurseCrest => {
            should_split(mem.deref(&pd.completed_memory_witch).unwrap_or_default())
        }
        Split::GainedCurse => should_split(mem.deref(&pd.gained_curse).unwrap_or_default()),
        Split::WitchCrest => should_split(
            mem.deref(&pd.belltown_doctor_cured_curse)
                .unwrap_or_default(),
        ),
        Split::ShamanCrest => {
            should_split(mem.deref(&pd.completed_memory_shaman).unwrap_or_default())
        }
        Split::HunterCrestEvo1 => {
            let crest = mem.read_string(&pd.current_crest_id).unwrap_or_default();
            should_split(crest == "Hunter_v2")
        }
        Split::HunterCrestEvo2 => {
            let crest = mem.read_string(&pd.current_crest_id).unwrap_or_default();
            should_split(crest == "Hunter_v3")
        }
        Split::VesticrestYellowSlot => should_split(
            mem.deref(&pd.unlocked_extra_yellow_slot)
                .unwrap_or_default(),
        ),
        Split::VesticrestBlueSlot => {
            should_split(mem.deref(&pd.unlocked_extra_blue_slot).unwrap_or_default())
        }
        Split::Sylphsong => {
            should_split(mem.deref(&pd.has_bound_crest_upgrader).unwrap_or_default())
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
        Split::SavedFleaUnderworksCauldron => {
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
        Split::SlabStation => {
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
        Split::GrandBellwayTube => should_split(
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
        Split::ShakraMapBuyout => {
            // iterate over maps in reverse shop order to fail out as quickly as possible
            let maps = [
                &pd.has_swamp_map,
                &pd.has_coral_map,
                &pd.has_peak_map,
                &pd.has_dustpens_map,
                &pd.has_judge_steps_map,
                &pd.has_shellwood_map,
                &pd.has_bellhart_map,
                &pd.has_greymoor_map,
                &pd.has_hunters_nest_map,
                &pd.has_crawl_map,
                &pd.has_wilds_map,
                &pd.has_docks_map,
                &pd.has_boneforest_map,
                &pd.has_moss_grotto_map,
            ];
            should_split(maps.iter().all(|map| mem.deref(map).unwrap_or_default()))
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
        Split::BeastlingCall => {
            should_split(mem.deref(&pd.has_fast_travel_teleport).unwrap_or_default())
        }
        Split::SilkSoar => should_split(mem.deref(&pd.has_super_jump).unwrap_or_default()),
        Split::ElegyOfTheDeep => should_split(
            mem.deref(&pd.has_needolin_memory_powerup)
                .unwrap_or_default(),
        ),
        Split::NylethEncountered => {
            should_split(mem.deref(&pd.encountered_flower_queen).unwrap_or_default())
        }
        Split::Nyleth => should_split(mem.deref(&pd.defeated_flower_queen).unwrap_or_default()),
        Split::HeartNyleth => {
            should_split(mem.deref(&pd.collected_heart_flower).unwrap_or_default())
        }
        Split::KhannEncountered => {
            should_split(mem.deref(&pd.encountered_coral_king).unwrap_or_default())
        }
        Split::CrustKingKhann => {
            should_split(mem.deref(&pd.defeated_coral_king).unwrap_or_default())
        }
        Split::HeartKhann => should_split(mem.deref(&pd.collected_heart_coral).unwrap_or_default()),
        Split::SkarrsingerKarmelita => {
            should_split(mem.deref(&pd.defeated_ant_queen).unwrap_or_default())
        }
        Split::HeartKarmelita => {
            should_split(mem.deref(&pd.collected_heart_hunter).unwrap_or_default())
        }
        Split::Palestag => {
            should_split(mem.deref(&pd.defeated_white_cloverstag).unwrap_or_default())
        }
        Split::CloverDancersEncountered => should_split(
            mem.deref(&pd.encountered_clover_dancers)
                .unwrap_or_default(),
        ),
        Split::CloverDancers => {
            should_split(mem.deref(&pd.defeated_clover_dancers).unwrap_or_default())
        }
        Split::HeartClover => {
            should_split(mem.deref(&pd.collected_heart_clover).unwrap_or_default())
        }
        Split::RedMemory => should_split(mem.deref(&pd.completed_red_memory).unwrap_or_default()),
        Split::BellhouseKeyConversation => should_split(
            mem.deref(&pd.belltown_greeter_house_full_dlg)
                .unwrap_or_default(),
        ),
        Split::VerdaniaLakeFountainOrbs => {
            should_split(mem.deref(&pd.summoned_lake_orbs).unwrap_or_default())
        }
        Split::VerdaniaOrbsCollected => {
            let orb_02c: bool = mem.deref(&pd.orbs_02c).unwrap_or_default();
            let orb_03: bool = mem.deref(&pd.orbs_03).unwrap_or_default();
            let orb_06: bool = mem.deref(&pd.orbs_06).unwrap_or_default();
            let orb_11: bool = mem.deref(&pd.orbs_11).unwrap_or_default();
            let orb_16_b: bool = mem.deref(&pd.orbs_16_b).unwrap_or_default();
            let orb_16_c: bool = mem.deref(&pd.orbs_16_c).unwrap_or_default();
            let orb_21: bool = mem.deref(&pd.orbs_21).unwrap_or_default();
            // C# ulong is an unsigned 64-bit int
            let orb_18_a: u64 = mem.deref(&pd.orbs_18_a).unwrap_or_default();
            let orb_18_b: u64 = mem.deref(&pd.orbs_18_b).unwrap_or_default();
            let orb_18_c: u64 = mem.deref(&pd.orbs_18_c).unwrap_or_default();
            let orb_18_d: u64 = mem.deref(&pd.orbs_18_d).unwrap_or_default();
            let orb_18_e: u64 = mem.deref(&pd.orbs_18_e).unwrap_or_default();
            let orb_19: u64 = mem.deref(&pd.orbs_19).unwrap_or_default();

            let singles: usize = [orb_02c, orb_03, orb_06, orb_11, orb_16_b, orb_16_c, orb_21]
                .into_iter()
                .filter(|b| *b)
                .count();
            let multis: usize = [orb_18_a, orb_18_b, orb_18_c, orb_18_d, orb_18_e, orb_19]
                .into_iter()
                .fold(0, |acc, x| acc + (x.count_ones() as usize));
            let total = singles + multis;
            should_split(total >= 12)
        }
        Split::Forebrothers => {
            should_split(mem.deref(&pd.defeated_dock_foremen).unwrap_or_default())
        }
        Split::Groal => should_split(mem.deref(&pd.defeated_swamp_shaman).unwrap_or_default()),
        Split::SavageBeastfly1 => {
            should_split(mem.deref(&pd.defeated_bone_flyer_giant).unwrap_or_default())
        }
        Split::SavageBeastfly2 => should_split(
            mem.deref(&pd.defeated_bone_flyer_giant_golem_scene)
                .unwrap_or_default(),
        ),
        Split::CaravanTroupeGreymoor => should_split(
            mem.deref(&pd.caravan_troupe_location)
                .is_ok_and(|n: i32| n >= 1),
        ),
        Split::CaravanTroupeBlastedSteps => should_split(
            mem.deref(&pd.caravan_troupe_location)
                .is_ok_and(|n: i32| n >= 2),
        ),
        Split::CaravanTroupeFleatopia => should_split(
            mem.deref(&pd.caravan_troupe_location)
                .is_ok_and(|n: i32| n >= 3),
        ),
        Split::SoldRelic => should_split(
            mem.deref(&pd.belltown_relic_dealer_gave_relic)
                .unwrap_or_default(),
        ),
        Split::PavoTimePassed => should_split(
            mem.deref(&pd.belltown_greeter_met_time_passed)
                .unwrap_or_default(),
        ),
        Split::SongclaveBell => {
            should_split(mem.deref(&pd.bell_shrine_enclave).unwrap_or_default())
        }
        Split::SkullTyrant1 => should_split(mem.deref(&pd.skull_king_defeated).unwrap_or_default()),
        Split::ShermaReturned => {
            should_split(mem.deref(&pd.sherma_healer_active).unwrap_or_default())
        }
        Split::JubilanaRescuedMemorium => {
            should_split(mem.deref(&pd.enclave_merchant_saved).unwrap_or_default())
        }
        Split::JubilanaRescuedChoralChambers => {
            should_split(mem.deref(&pd.city_merchant_saved).unwrap_or_default())
        }
        Split::SilkAndSoulOffered => should_split(
            mem.deref(&pd.caretaker_offered_snare_quest)
                .unwrap_or_default(),
        ),
        Split::SoulSnareReady => should_split(mem.deref(&pd.soul_snare_ready).unwrap_or_default()),
        Split::DestroyedCogworksVoidArena => should_split(
            mem.deref(&pd.completed_cog_10_abyss_battle)
                .unwrap_or_default(),
        ),
        Split::Seth => should_split(mem.deref(&pd.defeated_seth).unwrap_or_default()),
        Split::BallowMoved => should_split(
            mem.deref(&pd.ballow_moved_to_diving_bell)
                .unwrap_or_default(),
        ),
        Split::Act3Started => should_split(mem.deref(&pd.black_thread_world).unwrap_or_default()),
        // endregion: MiscTE

        // region: Abyss
        Split::AbyssEscape => {
            should_split(mem.deref(&pd.completed_abyss_ascent).unwrap_or_default())
        }
        Split::LostLaceEncountered => {
            should_split(mem.deref(&pd.encountered_lost_lace).unwrap_or_default())
        }
        // endregion: Abyss

        // region Tools
        Split::StraightPin => should_split(store.has_tool(&utf16!("Straight Pin"), e)),
        Split::ThreefoldPin => should_split(store.has_tool(&utf16!("Tri Pin"), e)),
        Split::StingShard => should_split(store.has_tool(&utf16!("Sting Shard"), e)),
        Split::Tacks => should_split(store.has_tool(&utf16!("Tack"), e)),
        Split::Longpin => should_split(store.has_tool(&utf16!("Harpoon"), e)),
        Split::Curveclaw => should_split(store.has_tool(&utf16!("Curve Claws"), e)),
        Split::Curvesickle => should_split(store.has_tool(&utf16!("Curve Claws Upgraded"), e)),
        Split::ThrowingRing => should_split(store.has_tool(&utf16!("Shakra Ring"), e)),
        Split::Pimpillo => should_split(store.has_tool(&utf16!("Pimpilo"), e)),
        Split::Conchcutter => should_split(store.has_tool(&utf16!("Conch Drill"), e)),
        Split::SilkshotForge => should_split(store.has_tool(&utf16!("WebShot Forge"), e)),
        Split::SilkshotArchitect => should_split(store.has_tool(&utf16!("WebShot Architect"), e)),
        Split::SilkshotWeaver => should_split(store.has_tool(&utf16!("WebShot Weaver"), e)),
        Split::DelversDrill => should_split(store.has_tool(&utf16!("Screw Attack"), e)),
        Split::CogworkWheel => should_split(store.has_tool(&utf16!("Cogwork Saw"), e)),
        Split::Cogfly => should_split(store.has_tool(&utf16!("Cogwork Flier"), e)),
        Split::RosaryCannon => should_split(store.has_tool(&utf16!("Rosary Cannon"), e)),
        Split::Voltvessels => should_split(store.has_tool(&utf16!("Lightning Rod"), e)),
        Split::Flintslate => should_split(store.has_tool(&utf16!("Flintstone"), e)),
        Split::SnareSetter => should_split(store.has_tool(&utf16!("Silk Snare"), e)),
        Split::FleaBrew => should_split(store.has_tool(&utf16!("Flea Brew"), e)),
        Split::PlasmiumPhial => should_split(store.has_tool(&utf16!("Lifeblood Syringe"), e)),
        Split::NeedlePhial => should_split(store.has_tool(&utf16!("Extractor"), e)),
        Split::DruidsEye => should_split(store.has_tool(&utf16!("Mosscreep Tool 1"), e)),
        Split::DruidsEyes => should_split(store.has_tool(&utf16!("Mosscreep Tool 2"), e)),
        Split::MagmaBell => should_split(store.has_tool(&utf16!("Lava Charm"), e)),
        Split::WardingBell => should_split(store.has_tool(&utf16!("Bell Bind"), e)),
        Split::PollipPouch => should_split(store.has_tool(&utf16!("Poison Pouch"), e)),
        Split::FracturedMask => should_split(store.has_tool(&utf16!("Fractured Mask"), e)),
        Split::Multibinder => should_split(store.has_tool(&utf16!("Multibind"), e)),
        Split::Weavelight => should_split(store.has_tool(&utf16!("White Ring"), e)),
        Split::SawtoothCirclet => should_split(store.has_tool(&utf16!("Brolly Spike"), e)),
        Split::InjectorBand => should_split(store.has_tool(&utf16!("Quickbind"), e)),
        Split::SpoolExtender => should_split(store.has_tool(&utf16!("Spool Extender"), e)),
        Split::ReserveBind => should_split(store.has_tool(&utf16!("Reserve Bind"), e)),
        Split::ClawMirror => should_split(store.has_tool(&utf16!("Dazzle Bind"), e)),
        Split::ClawMirrors => should_split(store.has_tool(&utf16!("Dazzle Bind Upgraded"), e)),
        Split::MemoryCrystal => should_split(store.has_tool(&utf16!("Revenge Crystal"), e)),
        Split::SnitchPick => should_split(store.has_tool(&utf16!("Thief Claw"), e)),
        Split::VoltFilament => should_split(store.has_tool(&utf16!("Zap Imbuement"), e)),
        Split::QuickSling => should_split(store.has_tool(&utf16!("Quick Sling"), e)),
        Split::WreathofPurity => should_split(store.has_tool(&utf16!("Maggot Charm"), e)),
        Split::Longclaw => should_split(store.has_tool(&utf16!("Longneedle"), e)),
        Split::WispfireLantern => should_split(store.has_tool(&utf16!("Wisp Lantern"), e)),
        Split::EggofFlealia => should_split(store.has_tool(&utf16!("Flea Charm"), e)),
        Split::PinBadge => should_split(store.has_tool(&utf16!("Pinstress Tool"), e)),
        Split::Compass => should_split(store.has_tool(&utf16!("Compass"), e)),
        Split::ShardPendant => should_split(store.has_tool(&utf16!("Bone Necklace"), e)),
        Split::MagnetiteBrooch => should_split(store.has_tool(&utf16!("Rosary Magnet"), e)),
        Split::WeightedBelt => should_split(store.has_tool(&utf16!("Weighted Anklet"), e)),
        Split::BarbedBracelet => should_split(store.has_tool(&utf16!("Barbed Wire"), e)),
        Split::DeadBugsPurse => should_split(store.has_tool(&utf16!("Dead Mans Purse"), e)),
        Split::ShellSatchel => should_split(store.has_tool(&utf16!("Shell Satchel"), e)),
        Split::MagnetiteDice => should_split(store.has_tool(&utf16!("Magnetite Dice"), e)),
        Split::Scuttlebrace => should_split(store.has_tool(&utf16!("Scuttlebrace"), e)),
        Split::AscendantsGrip => should_split(store.has_tool(&utf16!("Wallcling"), e)),
        Split::SpiderStrings => should_split(store.has_tool(&utf16!("Musician Charm"), e)),
        Split::SilkspeedAnklets => should_split(store.has_tool(&utf16!("Sprintmaster"), e)),
        Split::ThiefsMark => should_split(store.has_tool(&utf16!("Thief Charm"), e)),
        // endregion Tools

        // else
        _ => should_split(false),
    }
}

pub fn splits(
    split: &Split,
    env: &Env,
    trans_now: bool,
    ss: &mut SceneStore,
    store: &mut Store,
) -> SplitterAction {
    let a1 = continuous_splits(split, env, store).or_else(|| {
        let scenes = ss.pair();
        let a2 = if !ss.split_this_transition {
            transition_once_splits(split, &scenes, env)
        } else {
            SplitterAction::Pass
        };
        a2.or_else(|| {
            if trans_now {
                if is_menu(scenes.old) || is_menu(scenes.current) {
                    menu_splits(split, &scenes, env, store)
                } else {
                    transition_splits(split, &scenes, env)
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
