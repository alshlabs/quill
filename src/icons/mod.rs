#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod mod_linux; 
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use self::mod_linux::*;

pub const ICON_ASTERISK         : &[u8] = include_bytes!("./fontawesome-icons-7.3.1/solid/asterisk.svg");
pub const ICON_SEARCH           : &[u8] = include_bytes!("./vscode/search-large.svg");
pub const ICON_CLOSE            : &[u8] = include_bytes!("./vscode/close.svg");
pub const ICON_COMMENT          : &[u8] = include_bytes!("./vscode/comment.svg");
pub const ICON_NEW_SESSION       : &[u8] = include_bytes!("./vscode/new-session.svg");
pub const ICON_FEATHER          : &[u8] = include_bytes!("./fontawesome-icons-7.3.1/solid/feather.svg");
pub const ICON_EXTENSIONS       : &[u8] = include_bytes!("./vscode/extensions.svg");
pub const ICON_GEAR             : &[u8] = include_bytes!("./vscode/gear.svg");
// pub const ICON_ACCOUNT          : &[u8] = include_bytes!("./vscode/account.svg");
pub const ICON_BRUSH            : &[u8] = include_bytes!("./fontawesome-icons-7.3.1/solid/paintbrush.svg");
pub const ICON_FOLDER_OPENED    : &[u8] = include_bytes!("./vscode/folder-opened.svg");
pub const ICON_COMMAND          : &[u8] = include_bytes!("./bootstrap-icons-1.13.1/command.svg");
// pub const ICON_SHIFT            : &[u8] = include_bytes!("./bootstrap-icons-1.13.1/shift.svg");
// pub const ICON_THREE_BARS       : &[u8] = include_bytes!("./vscode/three-bars.svg");
pub const ICON_CODE             : &[u8] = include_bytes!("./vscode/code.svg");
pub const ICON_SPARKLE          : &[u8] = include_bytes!("./vscode/sparkle.svg");
pub const ICON_SETTINGS         : &[u8] = include_bytes!("./vscode/settings.svg");
pub const ICON_MIC              : &[u8] = include_bytes!("./vscode/mic.svg");
pub const ICON_ARROW_UP         : &[u8] = include_bytes!("./vscode/arrow-up.svg");
pub const ICON_ADD              : &[u8] = include_bytes!("./vscode/add.svg");
