# BN-classifier

The tool consists of two parts - the CLI classification engine and the GUI visualizer.

## Classifier

To compile the code, go to `classifier` directory and run

`cargo build --release`

To then invoke the tool, run the binary as
```
.\target\release\classifier <INPUT_PATH> [-o <OUTPUT_PATH>]
```
- `INPUT_PATH` is a path to a file in aeon format containing a PSBN model, and HCTL properties
- `OUTPUT_PATH` is path where a resulting zip archive will be created


## Visualizer

Desktop app for visually exploring HCTL properties of Boolean networks.

### Development guide

Install `tauri` CLI using `cargo install tauri-cli`.

Then you can run `cargo tauri dev`, which will serve you the HTML/JS app stored in `ui` using the Rust code in `src-tauri`. You can use `tauri::command` macros to declare functions which interop between Rust and JS (see `main.rs` for an example).

As the app takes two CLI arguments (zip with classification results and original model in aeon format), if you want to use `cargo tauri dev`, run it as:

``
cargo tauri dev -- -- classification_results model
``

You can use the prepared example:

``
cargo tauri dev -- -- '../benchmarks/mapk/results.zip' '../benchmarks/mapk/model-with-properties.aeon'
``