// SPDX-License-Identifier: GPL-3.0-only

use std::{any::TypeId, num::NonZeroU16, path::PathBuf};

use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    iced::Subscription,
    theme, Application,
};
use hex_color::HexColor;
use serde::{Deserialize, Serialize};

use crate::localize::LANGUAGE_SORTER;
use crate::{app::App, tab1::View as View1};

pub const CONFIG_VERSION: u64 = 1;
pub const COSMIC_THEME_DARK: &str = "COSMIC Dark";
pub const COSMIC_THEME_LIGHT: &str = "COSMIC Light";

// Default icon sizes
pub const ICON_SIZE_LIST: u16 = 32;
pub const ICON_SIZE_LIST_CONDENSED: u16 = 48;
pub const ICON_SIZE_GRID: u16 = 64;
// TODO: 5 is an arbitrary number. Maybe there's a better icon size max
pub const ICON_SCALE_MAX: u16 = 5;

macro_rules! percent {
    ($perc:expr, $pixel:ident) => {
        (($perc.get() as f32 * $pixel as f32) / 100.).clamp(1., ($pixel * ICON_SCALE_MAX) as _)
    };
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Favorite {
    Home,
    Documents,
    Downloads,
    Music,
    Pictures,
    Videos,
    Path(PathBuf),
}

impl Favorite {
    pub fn from_path(path: PathBuf) -> Self {
        // Ensure that special folders are handled properly
        for favorite in &[
            Self::Home,
            Self::Documents,
            Self::Downloads,
            Self::Music,
            Self::Pictures,
            Self::Videos,
        ] {
            if let Some(favorite_path) = favorite.path_opt() {
                if favorite_path == path {
                    return favorite.clone();
                }
            }
        }
        Self::Path(path)
    }

    pub fn path_opt(&self) -> Option<PathBuf> {
        match self {
            Self::Home => dirs::home_dir(),
            Self::Documents => dirs::document_dir(),
            Self::Downloads => dirs::download_dir(),
            Self::Music => dirs::audio_dir(),
            Self::Pictures => dirs::picture_dir(),
            Self::Videos => dirs::video_dir(),
            Self::Path(path) => Some(path.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ColorSchemeKind {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ColorSchemeId(pub u64);

//TODO: there is a lot of extra code to keep the exported color scheme clean,
//consider how to reduce this
fn de_color_opt<'de, D>(deserializer: D) -> Result<Option<HexColor>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let hex_color: HexColor = Deserialize::deserialize(deserializer)?;
    Ok(Some(hex_color))
}

fn ser_color_opt<S>(hex_color_opt: &Option<HexColor>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::Error as _;
    match hex_color_opt {
        Some(hex_color) => Serialize::serialize(hex_color, serializer),
        None => Err(S::Error::custom("ser_color_opt called with None")),
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorSchemeAnsi {
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub black: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub red: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub green: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub yellow: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub blue: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub magenta: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub cyan: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub white: Option<HexColor>,
}

impl ColorSchemeAnsi {
    pub fn is_empty(&self) -> bool {
        self.black.is_none()
            && self.red.is_none()
            && self.green.is_none()
            && self.yellow.is_none()
            && self.blue.is_none()
            && self.magenta.is_none()
            && self.cyan.is_none()
            && self.white.is_none()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorScheme {
    pub name: String,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub foreground: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub background: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub cursor: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub bright_foreground: Option<HexColor>,
    #[serde(
        deserialize_with = "de_color_opt",
        serialize_with = "ser_color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub dim_foreground: Option<HexColor>,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub normal: ColorSchemeAnsi,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub bright: ColorSchemeAnsi,
    #[serde(skip_serializing_if = "ColorSchemeAnsi::is_empty")]
    pub dim: ColorSchemeAnsi,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ProfileId(pub u64);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub syntax_theme_dark: String,
    #[serde(default)]
    pub syntax_theme_light: String,
    #[serde(default)]
    pub tab_title: String,
    #[serde(default)]
    pub working_directory: String,
    #[serde(default)]
    pub hold: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "new-profile".to_string(),
            command: String::new(),
            syntax_theme_dark: COSMIC_THEME_DARK.to_string(),
            syntax_theme_light: COSMIC_THEME_LIGHT.to_string(),
            tab_title: String::new(),
            working_directory: String::new(),
            hold: false,
        }
    }
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
    pub app_theme: AppTheme,
    pub color_schemes_dark: std::collections::BTreeMap<ColorSchemeId, ColorScheme>,
    pub color_schemes_light: std::collections::BTreeMap<ColorSchemeId, ColorScheme>,
    pub desktop: DesktopConfig,
    pub favorites: Vec<Favorite>,
    pub show_details: bool,
    pub show_button_row: bool,
    pub show_embedded_terminal: bool,
    pub show_second_panel: bool,
    pub queue_file_operations: bool,
    pub tab_left: TabConfig,
    pub tab_right: TabConfig,
    pub paths_left: Vec<String>,
    pub paths_right: Vec<String>,
}

impl Config {
    pub fn load() -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
            Ok(config_handler) => {
                let config = match Config::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        log::info!("errors loading config: {:?}", errs);
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                log::error!("failed to create config handler: {}", err);
                (None, Config::default())
            }
        }
    }

    pub fn subscription() -> Subscription<cosmic_config::Update<Self>> {
        struct ConfigSubscription;
        cosmic_config::config_subscription(
            TypeId::of::<ConfigSubscription>(),
            App::APP_ID.into(),
            CONFIG_VERSION,
        )
    }

    pub fn color_schemes(
        &self,
        color_scheme_kind: ColorSchemeKind,
    ) -> &std::collections::BTreeMap<ColorSchemeId, ColorScheme> {
        match color_scheme_kind {
            ColorSchemeKind::Dark => &self.color_schemes_dark,
            ColorSchemeKind::Light => &self.color_schemes_light,
        }
    }

    pub fn color_scheme_kind(&self) -> ColorSchemeKind {
        if self.app_theme == AppTheme::Light {
            ColorSchemeKind::Light
        } else {
            ColorSchemeKind::Dark
        }
    }

    // Get a sorted and adjusted for duplicates list of color scheme names and ids
    pub fn color_scheme_names(
        &self,
        color_scheme_kind: ColorSchemeKind,
    ) -> Vec<(String, ColorSchemeId)> {
        let color_schemes = self.color_schemes(color_scheme_kind);
        let mut color_scheme_names =
            Vec::<(String, ColorSchemeId)>::with_capacity(color_schemes.len());
        for (color_scheme_id, color_scheme) in color_schemes {
            let mut name = color_scheme.name.clone();

            let mut copies = 1;
            while color_scheme_names.iter().any(|x| x.0 == name) {
                copies += 1;
                name = format!("{} ({})", color_scheme.name, copies);
            }

            color_scheme_names.push((name, *color_scheme_id));
        }
        color_scheme_names.sort_by(|a, b| LANGUAGE_SORTER.compare(&a.0, &b.0));
        color_scheme_names
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            color_schemes_dark: std::collections::BTreeMap::new(),
            color_schemes_light: std::collections::BTreeMap::new(),
            desktop: DesktopConfig::default(),
            favorites: vec![
                Favorite::Home,
                Favorite::Documents,
                Favorite::Downloads,
                Favorite::Music,
                Favorite::Pictures,
                Favorite::Videos,
            ],
            show_details: false,
            show_button_row: true,
            show_embedded_terminal: true,
            show_second_panel: true,
            queue_file_operations: true,
            tab_left: TabConfig::default(),
            tab_right: TabConfig::default(),
            paths_left: Vec::new(),
            paths_right: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct DesktopConfig {
    pub grid_spacing: NonZeroU16,
    pub icon_size: NonZeroU16,
    pub show_content: bool,
    pub show_mounted_drives: bool,
    pub show_trash: bool,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            grid_spacing: 100.try_into().unwrap(),
            icon_size: 100.try_into().unwrap(),
            show_content: true,
            show_mounted_drives: false,
            show_trash: false,
        }
    }
}

impl DesktopConfig {
    pub fn grid_spacing_for(&self, space: u16) -> u16 {
        percent!(self.grid_spacing, space) as _
    }
}

/// Global and local [`crate::tab::Tab`] config.
///
/// [`TabConfig`] contains options that are passed to each instance of [`crate::tab::Tab`].
/// These options are set globally through the main config, but each tab may change options
/// locally. Local changes aren't saved to the main config.
///
/// Both panels share this single type; `TabConfig1`/`TabConfig2` are kept as aliases for
/// compatibility with existing code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct TabConfig {
    pub view: View1,
    /// Show folders before files
    pub folders_first: bool,
    /// Show hidden files and folders
    pub show_hidden: bool,
    /// Icon zoom
    pub icon_sizes: IconSizes,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            view: View1::List,
            folders_first: true,
            show_hidden: false,
            icon_sizes: IconSizes::default(),
        }
    }
}

/// Compatibility aliases: both panes share the same configuration type.
pub type TabConfig1 = TabConfig;
pub type TabConfig2 = TabConfig;

macro_rules! percent {
    ($perc:expr, $pixel:ident) => {
        (($perc.get() as f32 * $pixel as f32) / 100.).clamp(1., ($pixel * ICON_SCALE_MAX) as _)
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct IconSizes {
    pub list: NonZeroU16,
    pub grid: NonZeroU16,
}

impl Default for IconSizes {
    fn default() -> Self {
        Self {
            list: 60.try_into().unwrap(),
            grid: 240.try_into().unwrap(),
        }
    }
}

impl IconSizes {
    pub fn list(&self) -> u16 {
        percent!(self.list, ICON_SIZE_LIST) as _
    }

    pub fn list_condensed(&self) -> u16 {
        percent!(self.list, ICON_SIZE_LIST_CONDENSED) as _
    }

    pub fn grid(&self) -> u16 {
        percent!(self.grid, ICON_SIZE_GRID) as _
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ─── IconSizes ────────────────────────────────────────────────────────────────

    #[test]
    fn icon_sizes_defaults() {
        let sizes = IconSizes::default();
        assert_eq!(sizes.list.get(), 60);
        assert_eq!(sizes.grid.get(), 240);
    }

    #[test]
    fn icon_sizes_list_clamped_to_range() {
        let sizes = IconSizes {
            list: 200.try_into().unwrap(),
            grid: 100.try_into().unwrap(),
        };
        // percent! clamps to 1..=ICON_SIZE_LIST * ICON_SCALE_MAX (32*5=160)
        assert!(sizes.list() >= 1);
        assert!(sizes.list() <= (ICON_SIZE_LIST * ICON_SCALE_MAX) as u16);
    }

    #[test]
    fn icon_sizes_grid_clamped_to_range() {
        let sizes = IconSizes {
            list: 100.try_into().unwrap(),
            grid: 500.try_into().unwrap(),
        };
        assert!(sizes.grid() >= 1);
        assert!(sizes.grid() <= (ICON_SIZE_GRID * ICON_SCALE_MAX) as u16);
    }

    #[test]
    fn icon_sizes_list_condensed_uses_condensed_base() {
        let sizes = IconSizes {
            list: 100.try_into().unwrap(),
            grid: 100.try_into().unwrap(),
        };
        // 100% of ICON_SIZE_LIST_CONDENSED (48) = 48
        assert_eq!(sizes.list_condensed(), 48);
        // 100% of ICON_SIZE_LIST (32) = 32
        assert_eq!(sizes.list(), 32);
    }

    // ─── AppTheme ─────────────────────────────────────────────────────────────────

    #[test]
    fn app_theme_serde_roundtrip() {
        let variants = [AppTheme::Dark, AppTheme::Light, AppTheme::System];
        for theme in variants {
            let json = serde_json::to_string(&theme).unwrap();
            let deserialized: AppTheme = serde_json::from_str(&json).unwrap();
            assert_eq!(theme, deserialized);
        }
    }

    #[test]
    fn app_theme_produces_valid_theme() {
        for theme in [AppTheme::Dark, AppTheme::Light, AppTheme::System] {
            let t = theme.theme();
            // Just ensure it doesn't panic and returns a usable theme
            let _ = t;
        }
    }

    // ─── Favorite ─────────────────────────────────────────────────────────────────

    #[test]
    fn favorite_home_roundtrip() {
        let fav = Favorite::Home;
        assert!(fav.path_opt().is_some());
    }

    #[test]
    fn favorite_path_preserves_path() {
        let path = PathBuf::from("/some/arbitrary/path");
        let fav = Favorite::Path(path.clone());
        assert_eq!(fav.path_opt(), Some(path));
    }

    #[test]
    fn favorite_from_path_recognizes_home() {
        let home = dirs::home_dir().unwrap();
        let fav = Favorite::from_path(home.clone());
        assert_eq!(fav, Favorite::Home);
    }

    #[test]
    fn favorite_from_path_recognizes_documents() {
        let docs = dirs::document_dir().unwrap();
        let fav = Favorite::from_path(docs.clone());
        assert_eq!(fav, Favorite::Documents);
    }

    #[test]
    fn favorite_from_path_fallback_to_path_variant() {
        let path = PathBuf::from("/nonexistent/unique/path");
        let fav = Favorite::from_path(path.clone());
        assert!(matches!(fav, Favorite::Path(ref p) if *p == path));
    }

    // ─── ColorSchemeAnsi ──────────────────────────────────────────────────────────

    #[test]
    fn color_scheme_ansi_empty_by_default() {
        let ansi = ColorSchemeAnsi::default();
        assert!(ansi.is_empty());
    }

    #[test]
    fn color_scheme_ansi_not_empty_when_set() {
        let mut ansi = ColorSchemeAnsi::default();
        ansi.black = Some(HexColor::from((255, 0, 0, 255)));
        assert!(!ansi.is_empty());
    }

    #[test]
    fn color_scheme_ansi_serde_roundtrip() {
        let ansi = ColorSchemeAnsi::default();
        let json = serde_json::to_string(&ansi).unwrap();
        let deserialized: ColorSchemeAnsi = serde_json::from_str(&json).unwrap();
        assert_eq!(ansi, deserialized);
    }

    // ─── Config defaults ──────────────────────────────────────────────────────────

    #[test]
    fn config_defaults_sanity() {
        let config = Config::default();
        assert_eq!(config.app_theme, AppTheme::System);
        assert!(config.show_button_row);
        assert!(config.show_embedded_terminal);
        assert!(config.show_second_panel);
        assert!(config.queue_file_operations);
        assert!(config.favorites.len() == 6);
        assert!(config.paths_left.is_empty());
        assert!(config.paths_right.is_empty());
    }

    #[test]
    fn config_tab_configs_default() {
        let config = Config::default();
        assert_eq!(config.tab_left.view, View1::List);
        assert!(config.tab_left.folders_first);
        assert!(!config.tab_left.show_hidden);

        assert_eq!(config.tab_right.view, View1::List);
        assert!(config.tab_right.folders_first);
        assert!(!config.tab_right.show_hidden);
    }

    // ─── DesktopConfig ────────────────────────────────────────────────────────────

    #[test]
    fn desktop_config_defaults() {
        let dc = DesktopConfig::default();
        assert_eq!(dc.grid_spacing.get(), 100);
        assert_eq!(dc.icon_size.get(), 100);
        assert!(dc.show_content);
        assert!(!dc.show_mounted_drives);
        assert!(!dc.show_trash);
    }

    #[test]
    fn desktop_config_grid_spacing_for() {
        let dc = DesktopConfig::default();
        // 100% of 100px = 100, clamped to max 100*5=500
        let result = dc.grid_spacing_for(100);
        assert!(result >= 1);
        assert!(result <= 500);
    }

    // ─── ColorSchemeId / ProfileId ───────────────────────────────────────────────

    #[test]
    fn color_scheme_id_transparent_serde() {
        let id = ColorSchemeId(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
        let deserialized: ColorSchemeId = serde_json::from_str("42").unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn profile_id_transparent_serde() {
        let id = ProfileId(99);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "99");
    }

    // ─── Config serde round-trip ─────────────────────────────────────────────────

    #[test]
    fn config_serde_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.app_theme, deserialized.app_theme);
        assert_eq!(config.favorites, deserialized.favorites);
        assert_eq!(config.show_details, deserialized.show_details);
        assert_eq!(config.show_button_row, deserialized.show_button_row);
        assert_eq!(
            config.show_embedded_terminal,
            deserialized.show_embedded_terminal
        );
        assert_eq!(config.show_second_panel, deserialized.show_second_panel);
        assert_eq!(
            config.queue_file_operations,
            deserialized.queue_file_operations
        );
    }

    #[test]
    fn config_deserialize_from_empty_json_uses_defaults() {
        // `#[serde(default)]` on Config means an empty JSON object should work
        let deserialized: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(deserialized.app_theme, AppTheme::System);
        assert!(deserialized.favorites.len() == 6);
    }
}
