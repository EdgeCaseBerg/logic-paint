use std::path::PathBuf;

pub enum UiActions {
    Nothing,
    SaveLevel,
    RecomputePalette,
    OpenLevel,
    LevelGridUpdated,
}

pub enum IOWorkerRequest {
    OpenFileDialog,
    Shutdown,
}

pub enum IOWorkerResponse {
    IoOpenChoice(PathBuf),
}
