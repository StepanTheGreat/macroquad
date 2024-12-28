use std::sync::mpsc::channel;

static mut PC_ASSETS_FOLDER: Option<String> = None;


/// Set the pc assets path
///
/// To avoid unneccessary management, this is one of the rare cases where global state
/// can somewhat simplify our life. If you keep this static at [None] (the default) - nothing happens, 
/// but if you need it, you can change it to [Some(...)] 
pub unsafe fn set_pc_assets_folder(to: Option<String>) {
    PC_ASSETS_FOLDER = to;
}

/// Get the inner value of pc_assets_folder static variable.
pub unsafe fn get_pc_assets_folder() -> Option<&'static String> {
    PC_ASSETS_FOLDER.as_ref()
}

/// Load file from the path and block until its loaded.
/// 
/// Will use filesystem on native targets, and http requests on the web. Under the hood it also uses the
/// global variable pc_assets_folder, which will be used on android to load files
/// 
/// For an "async" version of it (i.e. that uses calls) use [miniquad::fs::load_file] directly.
/// PS: This implementation simply uses a channel with default
pub fn load_file(path: &str) -> Result<Vec<u8>, miniquad::fs::Error> {
    #[cfg(target_os = "ios")]
    let _ = std::env::set_current_dir(std::env::current_exe().unwrap().parent().unwrap());

    #[cfg(not(target_os = "android"))]
    let path = if let Some(ref pc_assets) = unsafe { get_pc_assets_folder() } {
        format!("{pc_assets}/{path}")
    } else {
        path.to_string()
    };

    let (tx, rx) = channel();

    miniquad::fs::load_file(&path, move |res| {
        let _ = tx.send(res);
    });

    rx.recv().expect("Should be impossible to return an error")
}

/// Load string from the path and block until its loaded.
/// Right now this will use load_file and `from_utf8_lossy` internally, but
/// implementation details may change in the future
pub fn load_string(path: &str) -> Result<String, miniquad::fs::Error> {
    let data = load_file(path)?;
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