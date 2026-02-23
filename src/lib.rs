pub mod editor_grids;
pub mod editor_settings;
pub mod levels;
pub mod netbpm;
pub mod netppm;
pub mod playstate;
pub mod pop_up;
pub mod screens;
pub mod ui;
pub mod ui_actions;

pub fn base_dir() -> std::path::PathBuf {
    let dir = std::env::current_exe()
        .expect("failed to get current_exe")
        .parent()
        .unwrap()
        .to_path_buf();

    // To avoid a bit of friction with cargo run versus cargo build --release + run the binary
    // we can find out if we're in "dev mode" by looking for the tell tale signs of development
    // aka: there's a toml file hanging out somewhere above wherever we are.
    let mut probe = dir.clone();
    while probe.pop() {
        if probe.join("Cargo.toml").exists() {
            return probe;
        }
    }

    dir
}
