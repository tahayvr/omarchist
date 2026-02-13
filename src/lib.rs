pub mod assets;
pub mod shell;
pub mod system;
pub mod types;
pub mod ui;

pub use assets::{CombinedAssets, OmarchistAssets};
pub use types::themes;
pub use ui::about_page;
pub use ui::app_view::{ActivePage, MainWindowView};
pub use ui::dialogs;
pub use ui::menu;
pub use ui::menu::app_menu;
pub use ui::menu::title_bar::MainTitleBar;
pub use ui::settings_page;
pub use ui::theme_edit_page;
pub use ui::theme_edit_page::theme_edit::{NavigateToThemes, ThemeEditPage};
pub use ui::themes_page;
pub use ui::themes_page::themes::ThemesPage;
