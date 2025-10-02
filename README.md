# silksong-autosplit-wasm

An auto splitter for Hollow Knight: Silksong.

## Installation

### LiveSplit (Windows)

Get [LiveSplit](https://livesplit.org/downloads/), 1.8.34 or later.

Right-click for the context menu:
- Open Splits, From File... : Select your `.lss` splits file. Go to [HKSplitMaker](https://hksplitmaker.com/?game=silksong) to generate and download `.lss` splits files.
- Edit Splits... : Activate the autosplitter.
- Compare Against: Game Time.

See also:
- [Silksong-Resources LiveSplit Setup Guide](https://github.com/hk-speedrunning/Silksong-Resources/blob/main/setup.md#livesplit)

### LiveSplit One Druid (Windows, Linux, Mac)

Go to the [LiveSplit One Druid Latest Release](https://github.com/AlexKnauth/livesplit-one-druid/releases/latest) page,
and under the `Assets` section, download the one for your architecture and operating system.

When you run LiveSplitOne, it needs to have permission to read memory of other processes.
- Windows: no additional steps required.
- Linux: set the capabilities to include `CAP_SYS_PTRACE`, with a command like `sudo setcap CAP_SYS_PTRACE=+eip LiveSplitOne` to run once after downloading LiveSplitOne.
- Mac: you have to run it under `sudo`, with a command like `sudo ./LiveSplitOne` to run every time you want to open it.

Right-click or Control-click for the context menu:
- Splits, Open... : Select your `.lss` splits file. Go to [HKSplitMaker](https://hksplitmaker.com/?game=silksong) to generate and download `.lss` splits files.
- Open Auto-splitter... : Select the `silksong_autosplit_wasm_stable.wasm` file. Go to the [silksong-autosplit-wasm Latest Release](https://github.com/AlexKnauth/silksong-autosplit-wasm/releases/latest) to download that.
- Compare Against: Game Time.
- Hotkeys: Configure the hotkeys you want. The default hotkeys use numpad, so if your computer doesn't have a numpad, configure them differently.

#### Mac requirement: Rosetta

The autosplitter currently requires the game to be running as an Intel / x86_64 process, not an Apple / arm64 process.
So on Apple Silicon (M1, M2, etc.) Macs, you have to run the game under Rosetta:
- Right click on `Hollow Knight Silksong.app` in Game Files, `Get Info`,  check the box for `Open using Rosetta`.
- Next to `Hollow Knight Silksong.app`, put a [`steam_appid.txt`](https://github.com/hk-speedrunning/Silksong-Resources/releases/download/files/steam_appid.txt) file containing the number `1030300`.
- When you open `Hollow Knight Silksong.app`, check in Activity Moniter, on the CPU tab, the Kind column should say `Intel` for Silksong, not `Apple`.

### OBS LiveSplit One (Windows, Linux)

Go to the [OBS LiveSplit One Latest Release](https://github.com/AlexKnauth/obs-livesplit-one/releases/latest) page,
and under the `Assets` section, download the one for your architecture and operating system.
Follow the instructions in [How to install](https://github.com/AlexKnauth/obs-livesplit-one?tab=readme-ov-file#how-to-install):
- Windows: Extract the `obs-livesplit-one.dll` to `C:\Program Files\obs-studio\obs-plugins\64bit` or equivalent install directory.
- Linux: Ensure the plugins folder exists with `mkdir -p $HOME/.config/obs-studio/plugins`, then extract with a command like `tar -zxvf obs-livesplit-one-*-x86_64-unknown-linux-gnu.tar.gz -C $HOME/.config/obs-studio/plugins/`.

When you run OBS, it needs to have permission to read memory of other processes.
- Windows: no additional steps required.
- Linux: set the capabilities to include `CAP_SYS_PTRACE`, with a command like `sudo setcap CAP_SYS_PTRACE=+eip /usr/bin/obs` to run once after downloading OBS.

Add OBS Source: LiveSplit One.

Properties:
- Splits: Select your splits file. Go to [HKSplitMaker](https://hksplitmaker.com/?game=silksong) to generate and download `.lss` splits files.
- Activate

Open the OBS Settings from File, Settings:
- Go to the Hotkeys section and scroll down until you find LiveSplit One.
- Set a hotkey for `Toggle Timing Method`, and hit Ok.
- Hit that hotkey once to switch from the default, Real Time, to Game Time.

## Compilation

This auto splitter is written in Rust. In order to compile it, you need to
install the Rust compiler: [Install Rust](https://www.rust-lang.org/tools/install).

Afterwards install the WebAssembly target:
```sh
rustup target add wasm32-unknown-unknown --toolchain stable
```

The auto splitter can now be compiled:
```sh
cargo b --release
```

The auto splitter is then available at:
```
target/wasm32-unknown-unknown/release/silksong_autosplit_wasm.wasm
```

Make sure to look into the [API documentation](https://livesplit.org/asr/asr/) for the `asr` crate.

## Development

You can use the [debugger](https://github.com/LiveSplit/asr-debugger) while
developing the auto splitter to more easily see the log messages, statistics,
dump memory, step through the code and more.

The repository comes with preconfigured Visual Studio Code tasks. During
development it is recommended to use the `Debug Auto Splitter` launch action to
run the `asr-debugger`. You need to install the `CodeLLDB` extension to run it.

You can then use the `Build Auto Splitter (Debug)` task to manually build the
auto splitter. This will automatically hot reload the auto splitter in the
`asr-debugger`.

Alternatively you can install the [`cargo
watch`](https://github.com/watchexec/cargo-watch?tab=readme-ov-file#install)
subcommand and run the `Watch Auto Splitter` task for it to automatically build
when you save your changes.

The debugger is able to step through the code. You can set breakpoints in VSCode
and it should stop there when the breakpoint is hit. Inspecting variables may
not work all the time.

## Contributing

My approach to adding a new autosplit would look like this:
1. Search through the list of fields ([Silksong-Mono-dissector.TXT](Silksong-Mono-dissector.TXT)) to find one or more candidate fields that might correspond to what the autosplit should look for. For example on `Silk Spear (Skill)`, my candidate fields were `hasSilkSpecial` and `hasNeedleThrow`, and I wasn't sure which was the right one.
2. Test all candidate fields using a testing tool (https://github.com/AlexKnauth/asr-unity-mono-mac-testing/tree/silksong in combination with https://github.com/LiveSplit/asr-debugger can test it on all 3 OS's, not just Mac), ideally playing the game from the point right before getting to the point you want, seeing that good candidates should be `false` before, and then once you get the skill or boss or whatever, good candidates should be `true` after. Even better to test using a 2nd moniter so you can see exactly when a field goes from `false` to `true`. After I did this for `hasSilkSpecial` and `hasNeedleThrow`, I saw both go from `false` to `true` at basically the same time, so this didn't actually narrow it down, but at least confirmed they were related.
3. If multiple candidates pass step (2), ask for help. In the example of `hasSilkSpecial` and `hasNeedleThrow`, I got help from Atomic and Kazekai on the speedrun discord `#ss-tech-support` channel.
4. Make a new branch on your clone of the Github repository for the new feature you want to add. I'd recommend that you *don't* just use your master branch.
5. Add the field to the relevant `declare_pointers!` statement in `silksong_memory.rs`, add the split to the `Splits` datatype in `splits.rs`, and add the code for the split in the relevant function (either `menu_splits`, `transition_splits`, or `continuous_splits` in `splits.rs`).
6. Make a Pull Request on the Github repository (https://github.com/AlexKnauth/silksong-autosplit-wasm/pulls).
