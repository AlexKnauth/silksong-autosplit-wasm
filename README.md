# silksong-autosplit-wasm

An auto splitter for Hollow Knight: Silksong.

## Installation

Download the `silksong_autosplit_wasm_stable.wasm` file from the [Latest Release](https://github.com/AlexKnauth/silksong-autosplit-wasm/releases/latest).

Or follow the steps in [Compilation](#compilation) and use `target/wasm32-unknown-unknown/release/silksong_autosplit_wasm.wasm`.

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
4. Make a new branch on your clone of the Github repository, add the field to the relevant `declare_pointers!` statement in `silksong_memory.rs`, add the split to the `Splits` datatype in `splits.rs`, and add the code for the split in the relevant function (either `transition_splits` or `continuous_splits` in `splits.rs`).
5. Make a Pull Request on the Github repository (https://github.com/AlexKnauth/silksong-autosplit-wasm/pulls).
