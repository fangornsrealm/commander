// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use clap::Parser;
use cosmic::{app::Settings, iced::Limits};
use std::{fs, path::PathBuf, process};

use app::{App, Flags};
pub mod app;
pub mod clipboard;
use config::Config;
mod commanderpanegrid;
pub mod config;
pub mod dialog;
pub mod dnd;
mod key_bind;
mod localize;
mod menu;
mod mime_app;
pub mod mime_icon;
mod mounter;
mod mouse_area;
mod mouse_reporter;
pub mod operation;
mod pane_grid;
mod recipe;
mod spawn_detached;
use tab1::Location;
mod tab;
pub mod tab1;
pub mod tab2;
mod terminal;
mod terminal_box;
mod terminal_theme;
mod thumbnailer;
//pub mod terminal;

pub(crate) fn err_str<T: ToString>(err: T) -> String {
    err.to_string()
}

pub fn desktop_dir() -> PathBuf {
    match dirs::desktop_dir() {
        Some(path) => path,
        None => {
            let path = home_dir().join("Desktop");
            log::warn!("failed to locate desktop directory, falling back to {path:?}");
            path
        }
    }
}

pub fn home_dir() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => home,
        None => {
            let path = PathBuf::from("/");
            log::warn!("failed to locate home directory, falling back to {path:?}");
            path
        }
    }
}

pub fn sum_files_in_dir(path: &PathBuf) -> (u64, u64) {
    let mut num = 0;
    let mut size = 0;
    match std::fs::read_dir(path) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!("failed to read entry in {:?}: {}", path, err);
                        continue;
                    }
                };
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        num += 1;
                        size += metadata.len();
                    }
                }
            }
        }
        Err(error) => {
            log::error!("Failed to read contents of directory! {}", error);
            return (0, 0);
        }
    }
    (num, size)
}

pub fn sum_selected_files_in_dir(paths: Vec<PathBuf>) -> (u64, u64) {
    let mut num = 0;
    let mut size = 0;
    for path in paths {
        if let Ok(metadata) = path.metadata() {
            if metadata.is_file() {
                num += 1;
                size += metadata.len();
            }
        }
    }
    (num, size)
}

/// Runs application in desktop mode
#[rustfmt::skip]
pub fn desktop() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();

    let (config_handler, config) = Config::load();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);
    settings = settings.transparent(true);
    #[cfg(feature = "wayland")]
    {
        settings = settings.no_main_window(true);
    }

    let locations1 = vec![Location::Desktop(desktop_dir(), String::new(), config.desktop)];
    let locations2 = Vec::new();
    let flags = Flags {
        config_handler,
        config,
        mode: app::Mode::Desktop,
        locations1,
        locations2,
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}

/// Runs application with these settings
#[rustfmt::skip]
/// Command-line interface parsed by `clap`.
#[derive(clap::Parser)]
#[command(name = "commander", about = "Dual panel file manager for COSMIC")]
struct Cli {
    /// Paths to open (directories or files)
    #[arg(trailing_var_arg = true)]
    paths: Vec<String>,

    /// Open the trash folder
    #[arg(long)]
    trash: bool,

    /// Run in the foreground (do not daemonize)
    #[arg(long)]
    no_daemon: bool,
}

/// Runs application with these settings
#[rustfmt::skip]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();

    let cli = Cli::parse();
    let daemonize = !cli.no_daemon;

    let (config_handler, config) = Config::load();

    let mut locations = Vec::new();
    if cli.trash {
        locations.push(Location::Trash);
    }
    for arg in &cli.paths {
        //TODO: support more URLs
        let path = match url::Url::parse(arg) {
            Ok(url) => match url.to_file_path() {
                Ok(path) => path,
                Err(()) => {
                    log::warn!("invalid argument {arg:?}");
                    continue;
                }
            },
            Err(_) => PathBuf::from(arg),
        };
        match fs::canonicalize(&path) {
            Ok(absolute) => locations.push(Location::Path(absolute)),
            Err(err) => log::warn!("failed to canonicalize {path:?}: {err}"),
        }
    }

    if daemonize {
        #[cfg(all(unix, not(target_os = "redox")))]
        match fork::daemon(true, true) {
            Ok(fork::Fork::Child) => (),
            Ok(fork::Fork::Parent(_child_pid)) => process::exit(0),
            Err(err) => {
                eprintln!("failed to daemonize: {err:?}");
                process::exit(1);
            }
        }
    }

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);

    let flags = Flags {
        config_handler,
        config,
        mode: app::Mode::App,
        locations1: locations,
        locations2: Vec::new(),
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}
