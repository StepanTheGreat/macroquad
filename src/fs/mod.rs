use std::{path::Path, sync::mpsc::channel};

static mut PC_ASSETS_FOLDER: Option<String> = None;

/// To avoid unneccessary management, this is one of the rare cases where global state
/// can somewhat simplify our life. If you keep this static at [`None`] (the default)
/// - nothing happens, but if you need - you can change it to [`Some(...)`]
/// 
pub unsafe fn set_pc_assets_folder(to: Option<String>) {
    PC_ASSETS_FOLDER = to;
}

/// Get the inner value of pc_assets_folder static variable.
pub unsafe fn get_pc_assets_folder() -> Option<&'static String> {
    PC_ASSETS_FOLDER.as_ref()
}

/// Load file from the path and block until its loaded
/// Will use filesystem on PC and do http request on web
pub fn load_file(path: &str) -> Result<Vec<u8>, Error> {
    #[cfg(target_os = "ios")]
    let _ = std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap());

    #[cfg(not(target_os = "android"))]
    let path = if let Some(ref pc_assets) = unsafe { get_pc_assets_folder() } {
        format!("{pc_assets}/{path}")
    } else {
        path.to_string()
    };

    let (tx, rx) = channel();

    miniquad::fs::load_file(&path, |res| {
        tx.send(res);
    });

    rx.recv()
}

/// Load string from the path and block until its loaded.
/// Right now this will use load_file and `from_utf8_lossy` internally, but
/// implementation details may change in the future
pub fn load_string(path: &str) -> Result<String, Error> {
    let data = load_file(path);
    Ok(String::from_utf8_lossy(&data).to_string())
}

// / There are super common project layout like this:
// / ```skip
// /    .
// /    ├── assets
// /    ├── └── nice_texture.png
// /    ├── src
// /    ├── └── main.rs
// /    └── Cargo.toml
// / ```
// / when such a project being run on desktop assets should be referenced as
// / "assets/nice_texture.png".
// / While on web or android it usually is just "nice_texture.png".
// / The reason: on PC assets are being referenced relative to current active directory/executable path. In most IDEs its the root of the project.
// / While on, say, android it is:
// / ```skip
// / [package.metadata.android]
// / assets = "assets"
// / ```
// / And therefore on android assets are referenced from the root of "assets" folder.
// /
// / In the future there going to be some sort of meta-data file for PC as well.
// / But right now to resolve this situation and keep pathes consistent across platforms
// / `set_pc_assets_folder("assets");`call before first `load_file`/`load_texture` will allow using same pathes on PC and Android.