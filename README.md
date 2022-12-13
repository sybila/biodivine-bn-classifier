# biodivine-hctl-explorer
Desktop app for visually exploring HCTL properties of Boolean networks.


# Development guide

Install `tauri` CLI using `cargo install tauri-cli`.

Then you can run `cargo tauri dev`, which will serve you the HTML/JS app stored in `ui` using the Rust code in `src-tauri`. You can use `tauri::command` macros to declare functions which interop between Rust and JS (see `main.rs` for an example).