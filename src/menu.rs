// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    iced::{Alignment, Background, Border, Length},
    theme,
    widget::{
        self, button, column, container, divider, horizontal_space,
        menu::{self, key_bind::KeyBind, ItemHeight, ItemWidth, MenuBar},
        text, Row,
    },
    Element,
};
use i18n_embed::LanguageLoader;
use mime_guess::Mime;
use std::collections::HashMap;

use crate::{
    app::{Action, Message},
    config::Config,
    fl,
    tab1::{self, HeadingOptions as HeadingOptions1, Location as Location1, LocationMenuAction as LocationMenuAction1, Tab as Tab1},
    tab2::{self, HeadingOptions as HeadingOptions2, Location as Location2, LocationMenuAction as LocationMenuAction2, Tab as Tab2},
};

macro_rules! menu_button {
    ($($x:expr),+ $(,)?) => (
        button::custom(
            Row::with_children(
                vec![$(Element::from($x)),+]
            )
            .height(Length::Fixed(24.0))
            .align_y(Alignment::Center)
        )
        .padding([theme::active().cosmic().spacing.space_xxs, 16])
        .width(Length::Fill)
        .class(theme::Button::MenuItem)
    );
}

fn menu_button_optional(
    label: String,
    action: Action,
    enabled: bool,
) -> menu::Item<Action, String> {
    if enabled {
        menu::Item::Button(label, None, action)
    } else {
        menu::Item::ButtonDisabled(label, None, action)
    }
}

pub fn context_menu1<'a>(
    tab: &Tab1,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, tab1::Message> {
    let find_key = |action: &Action| -> String {
        for (key_bind, key_action) in key_binds.iter() {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    };

    let menu_item = |label, action| {
        let key = find_key(&action);
        menu_button!(text::body(label), horizontal_space(), text::body(key))
            .on_press(tab1::Message::ContextAction(action))
    };

    let (sort_name, sort_direction, _) = tab.sort_options();
    let sort_item = |label, variant| {
        menu_item(
            format!(
                "{} {}",
                label,
                match (sort_name == variant, sort_direction) {
                    (true, true) => "\u{2B07}",
                    (true, false) => "\u{2B06}",
                    _ => "",
                }
            ),
            Action::ToggleSortLeft(variant),
        )
        .into()
    };

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_trash_only = false;
    let mut selected_desktop_entry = None;
    let mut selected_types: Vec<Mime> = vec![];
    if let Some(items) = tab.items_opt() {
        for item in items.iter() {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_dir += 1;
                }
                match &item.location_opt {
                    Some(Location1::Trash) => selected_trash_only = true,
                    Some(Location1::Path(path)) => {
                        if selected == 1
                            && path.extension().and_then(|s| s.to_str()) == Some("desktop")
                        {
                            selected_desktop_entry = Some(&**path);
                        }
                    }
                    _ => (),
                }
                selected_types.push(item.mime.clone());
            }
        }
    };
    selected_types.sort_unstable();
    selected_types.dedup();
    selected_trash_only = selected_trash_only && selected == 1;
    // Parse the desktop entry if it is the only selection
    #[cfg(feature = "desktop")]
    let selected_desktop_entry = selected_desktop_entry.and_then(|path| {
        if selected == 1 {
            let lang_id = crate::localize::LANGUAGE_LOADER.current_language();
            let language = lang_id.language.as_str();
            // Cache?
            cosmic::desktop::load_desktop_file(Some(language), path)
        } else {
            None
        }
    });

    let mut children: Vec<Element<_>> = Vec::new();
    match (&tab.mode, &tab.location) {
        (
            tab1::Mode::App | tab1::Mode::Desktop,
            Location1::Desktop(..) | Location1::Path(..) | Location1::Search(..) | Location1::Recents,
        ) => {
            if selected_trash_only {
                children.push(menu_item(fl!("open"), Action::Open).into());
                if tab1::trash_entries() > 0 {
                    children.push(menu_item(fl!("empty-trash"), Action::EmptyTrash).into());
                }
            } else if let Some(entry) = selected_desktop_entry {
                children.push(menu_item(fl!("open"), Action::Open).into());
                #[cfg(feature = "desktop")]
                {
                    for (i, action) in entry.desktop_actions.into_iter().enumerate() {
                        children.push(menu_item(action.name, Action::ExecEntryAction(i)).into())
                    }
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                children.push(menu_item(fl!("copy"), Action::Copy).into());
                // Should this simply bypass trash and remove the shortcut?
                children.push(menu_item(fl!("move-to-trash"), Action::MoveToTrash).into());
            } else if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if selected == 1 {
                    children.push(menu_item(fl!("menu-open-with"), Action::OpenWith).into());
                    if selected_dir == 1 {
                        children
                            .push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                    }
                }
                if matches!(tab.location, Location1::Search(..) | Location1::Recents) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
                // All selected items are directories
                if selected == selected_dir && matches!(tab.mode, tab1::Mode::App) {
                    children.push(menu_item(fl!("open-in-new-tab"), Action::OpenInNewTab).into());
                    children
                        .push(menu_item(fl!("open-in-new-window"), Action::OpenInNewWindow).into());
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                children.push(menu_item(fl!("copy"), Action::Copy).into());

                children.push(divider::horizontal::light().into());
                let supported_archive_types = [
                    "application/gzip",
                    "application/x-compressed-tar",
                    "application/x-tar",
                    "application/zip",
                    #[cfg(feature = "bzip2")]
                    "application/x-bzip",
                    #[cfg(feature = "bzip2")]
                    "application/x-bzip-compressed-tar",
                    #[cfg(feature = "liblzma")]
                    "application/x-xz",
                    #[cfg(feature = "liblzma")]
                    "application/x-xz-compressed-tar",
                ]
                .iter()
                .filter_map(|mime_type| mime_type.parse::<Mime>().ok())
                .collect::<Vec<_>>();
                selected_types.retain(|t| !supported_archive_types.contains(t));
                if selected_types.is_empty() {
                    children.push(menu_item(fl!("extract-here"), Action::ExtractHere).into());
                }
                children.push(menu_item(fl!("compress"), Action::Compress).into());
                children.push(divider::horizontal::light().into());

                //TODO: Print?
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
                if matches!(tab.mode, tab1::Mode::App) {
                    children.push(divider::horizontal::light().into());
                    children.push(menu_item(fl!("add-to-sidebar"), Action::AddToSidebar).into());
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("move-to-trash"), Action::MoveToTrash).into());
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("new-tab"), Action::TabNew).into());
                children.push(menu_item(fl!("copy-tab"), Action::CopyTab).into());
                children.push(menu_item(fl!("move-tab"), Action::MoveTab).into());
                // zoom does not work!
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("zoom-in"), Action::ZoomIn).into());
                children.push(menu_item(fl!("default-size"), Action::ZoomDefault).into());                
                children.push(menu_item(fl!("zoom-out"), Action::ZoomOut).into());
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("grid-view"), Action::TabViewGrid).into());
                children.push(menu_item(fl!("list-view"), Action::TabViewList).into());
                children.push(divider::horizontal::light().into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions1::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions1::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions1::Size));
            } else {
                //TODO: need better designs for menu with no selection
                //TODO: have things like properties but they apply to the folder?
                children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                children.push(menu_item(fl!("new-file"), Action::NewFile).into());
                children.push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                children.push(divider::horizontal::light().into());
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                children.push(menu_item(fl!("paste"), Action::Paste).into());

                //TODO: only show if cosmic-settings is found?
                if matches!(tab.mode, tab1::Mode::Desktop) {
                    children.push(divider::horizontal::light().into());
                    children.push(
                        menu_item(fl!("change-wallpaper"), Action::CosmicSettingsWallpaper).into(),
                    );
                    children.push(
                        menu_item(fl!("desktop-appearance"), Action::CosmicSettingsAppearance)
                            .into(),
                    );
                    children.push(
                        menu_item(fl!("display-settings"), Action::CosmicSettingsDisplays).into(),
                    );
                }
                // zoom does not work!
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("zoom-in"), Action::ZoomIn).into());
                children.push(menu_item(fl!("default-size"), Action::ZoomDefault).into());                
                children.push(menu_item(fl!("zoom-out"), Action::ZoomOut).into());
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("grid-view"), Action::TabViewGrid).into());
                children.push(menu_item(fl!("list-view"), Action::TabViewList).into());
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("new-tab"), Action::TabNew).into());
                children.push(menu_item(fl!("copy-tab"), Action::CopyTab).into());
                children.push(menu_item(fl!("move-tab"), Action::MoveTab).into());

                children.push(divider::horizontal::light().into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions1::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions1::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions1::Size));
                if matches!(tab.location, Location1::Desktop(..)) {
                    children.push(divider::horizontal::light().into());
                    children.push(
                        menu_item(fl!("desktop-view-options"), Action::DesktopViewOptions).into(),
                    );
                }
            }
        }
        (
            tab1::Mode::Dialog(dialog_kind),
            Location1::Desktop(..) | Location1::Path(..) | Location1::Search(..) | Location1::Recents,
        ) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if matches!(tab.location, Location1::Search(..) | Location1::Recents) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
            } else {
                if dialog_kind.save() {
                    children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                }
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                if !children.is_empty() {
                    children.push(divider::horizontal::light().into());
                }
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions1::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions1::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions1::Size));
            }
        }
        (_, Location1::Network(..)) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
            } else {
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                if !children.is_empty() {
                    children.push(divider::horizontal::light().into());
                }
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions1::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions1::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions1::Size));
            }
        }
        (_, Location1::Trash) => {
            if tab.mode.multiple() {
                children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
            }
            if !children.is_empty() {
                children.push(divider::horizontal::light().into());
            }
            if selected > 0 {
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
                children.push(divider::horizontal::light().into());
                children
                    .push(menu_item(fl!("restore-from-trash"), Action::RestoreFromTrash).into());
            } else {
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions1::Name));
                children.push(sort_item(fl!("sort-by-trashed"), HeadingOptions1::TrashedOn));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions1::Size));
            }
        }
    }

    container(column::with_children(children))
        .padding(1)
        //TODO: move style to libcosmic
        .style(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Style {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: cosmic.radius_s().map(|x| x + 1.0).into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(360.0))
        .into()
}

pub fn context_menu2<'a>(
    tab: &Tab2,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, tab2::Message> {
    let find_key = |action: &Action| -> String {
        for (key_bind, key_action) in key_binds.iter() {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    };

    let menu_item = |label, action| {
        let key = find_key(&action);
        menu_button!(text::body(label), horizontal_space(), text::body(key))
            .on_press(tab2::Message::ContextAction(action))
    };

    let (sort_name, sort_direction, _) = tab.sort_options();
    let sort_item = |label, variant| {
        menu_item(
            format!(
                "{} {}",
                label,
                match (sort_name == variant, sort_direction) {
                    (true, true) => "\u{2B07}",
                    (true, false) => "\u{2B06}",
                    _ => "",
                }
            ),
            Action::ToggleSortRight(variant),
        )
        .into()
    };

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_trash_only = false;
    let mut selected_desktop_entry = None;
    let mut selected_types: Vec<Mime> = vec![];
    if let Some(items) = tab.items_opt() {
        for item in items.iter() {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_dir += 1;
                }
                match &item.location_opt {
                    Some(Location2::Trash) => selected_trash_only = true,
                    Some(Location2::Path(path)) => {
                        if selected == 1
                            && path.extension().and_then(|s| s.to_str()) == Some("desktop")
                        {
                            selected_desktop_entry = Some(&**path);
                        }
                    }
                    _ => (),
                }
                selected_types.push(item.mime.clone());
            }
        }
    };
    selected_types.sort_unstable();
    selected_types.dedup();
    selected_trash_only = selected_trash_only && selected == 1;
    // Parse the desktop entry if it is the only selection
    #[cfg(feature = "desktop")]
    let selected_desktop_entry = selected_desktop_entry.and_then(|path| {
        if selected == 1 {
            let lang_id = crate::localize::LANGUAGE_LOADER.current_language();
            let language = lang_id.language.as_str();
            // Cache?
            cosmic::desktop::load_desktop_file(Some(language), path)
        } else {
            None
        }
    });

    let mut children: Vec<Element<_>> = Vec::new();
    match (&tab.mode, &tab.location) {
        (
            tab2::Mode::App | tab2::Mode::Desktop,
            Location2::Desktop(..) | Location2::Path(..) | Location2::Search(..) | Location2::Recents,
        ) => {
            if selected_trash_only {
                children.push(menu_item(fl!("open"), Action::Open).into());
                if tab2::trash_entries() > 0 {
                    children.push(menu_item(fl!("empty-trash"), Action::EmptyTrash).into());
                }
            } else if let Some(entry) = selected_desktop_entry {
                children.push(menu_item(fl!("open"), Action::Open).into());
                #[cfg(feature = "desktop")]
                {
                    for (i, action) in entry.desktop_actions.into_iter().enumerate() {
                        children.push(menu_item(action.name, Action::ExecEntryAction(i)).into())
                    }
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                children.push(menu_item(fl!("copy"), Action::Copy).into());
                // Should this simply bypass trash and remove the shortcut?
                children.push(menu_item(fl!("move-to-trash"), Action::MoveToTrash).into());
            } else if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if selected == 1 {
                    children.push(menu_item(fl!("menu-open-with"), Action::OpenWith).into());
                    if selected_dir == 1 {
                        children
                            .push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                    }
                }
                if matches!(tab.location, Location2::Search(..) | Location2::Recents) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
                // All selected items are directories
                if selected == selected_dir && matches!(tab.mode, tab2::Mode::App) {
                    children.push(menu_item(fl!("open-in-new-tab"), Action::OpenInNewTab).into());
                    children
                        .push(menu_item(fl!("open-in-new-window"), Action::OpenInNewWindow).into());
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("rename"), Action::Rename).into());
                children.push(menu_item(fl!("cut"), Action::Cut).into());
                children.push(menu_item(fl!("copy"), Action::Copy).into());

                children.push(divider::horizontal::light().into());
                let supported_archive_types = [
                    "application/gzip",
                    "application/x-compressed-tar",
                    "application/x-tar",
                    "application/zip",
                    #[cfg(feature = "bzip2")]
                    "application/x-bzip",
                    #[cfg(feature = "bzip2")]
                    "application/x-bzip-compressed-tar",
                    #[cfg(feature = "liblzma")]
                    "application/x-xz",
                    #[cfg(feature = "liblzma")]
                    "application/x-xz-compressed-tar",
                ]
                .iter()
                .filter_map(|mime_type| mime_type.parse::<Mime>().ok())
                .collect::<Vec<_>>();
                selected_types.retain(|t| !supported_archive_types.contains(t));
                if selected_types.is_empty() {
                    children.push(menu_item(fl!("extract-here"), Action::ExtractHere).into());
                }
                children.push(menu_item(fl!("compress"), Action::Compress).into());
                children.push(divider::horizontal::light().into());

                //TODO: Print?
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
                if matches!(tab.mode, tab2::Mode::App) {
                    children.push(divider::horizontal::light().into());
                    children.push(menu_item(fl!("add-to-sidebar"), Action::AddToSidebar).into());
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("move-to-trash"), Action::MoveToTrash).into());
                // zoom does not work!
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("zoom-in"), Action::ZoomIn).into());
                children.push(menu_item(fl!("default-size"), Action::ZoomDefault).into());                
                children.push(menu_item(fl!("zoom-out"), Action::ZoomOut).into());
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("grid-view"), Action::TabViewGrid).into());
                children.push(menu_item(fl!("list-view"), Action::TabViewList).into());
                children.push(divider::horizontal::light().into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions2::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions2::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions2::Size));
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("new-tab"), Action::TabNew).into());
                children.push(menu_item(fl!("copy-tab"), Action::CopyTab).into());
                children.push(menu_item(fl!("move-tab"), Action::MoveTab).into());
            } else {
                //TODO: need better designs for menu with no selection
                //TODO: have things like properties but they apply to the folder?
                children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                children.push(menu_item(fl!("new-file"), Action::NewFile).into());
                children.push(menu_item(fl!("open-in-terminal"), Action::OpenTerminal).into());
                children.push(divider::horizontal::light().into());
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                children.push(menu_item(fl!("paste"), Action::Paste).into());

                //TODO: only show if cosmic-settings is found?
                if matches!(tab.mode, tab2::Mode::Desktop) {
                    children.push(divider::horizontal::light().into());
                    children.push(
                        menu_item(fl!("change-wallpaper"), Action::CosmicSettingsWallpaper).into(),
                    );
                    children.push(
                        menu_item(fl!("desktop-appearance"), Action::CosmicSettingsAppearance)
                            .into(),
                    );
                    children.push(
                        menu_item(fl!("display-settings"), Action::CosmicSettingsDisplays).into(),
                    );
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("new-tab"), Action::TabNew).into());
                children.push(menu_item(fl!("copy-tab"), Action::CopyTab).into());
                children.push(menu_item(fl!("move-tab"), Action::MoveTab).into());
                // zoom does not work!
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("zoom-in"), Action::ZoomIn).into());
                children.push(menu_item(fl!("default-size"), Action::ZoomDefault).into());                
                children.push(menu_item(fl!("zoom-out"), Action::ZoomOut).into());
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("grid-view"), Action::TabViewGrid).into());
                children.push(menu_item(fl!("list-view"), Action::TabViewList).into());
                children.push(divider::horizontal::light().into());
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions2::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions2::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions2::Size));
                if matches!(tab.location, Location2::Desktop(..)) {
                    children.push(divider::horizontal::light().into());
                    children.push(
                        menu_item(fl!("desktop-view-options"), Action::DesktopViewOptions).into(),
                    );
                }
            }
        }
        (
            tab2::Mode::Dialog(dialog_kind),
            Location2::Desktop(..) | Location2::Path(..) | Location2::Search(..) | Location2::Recents,
        ) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
                if matches!(tab.location, Location2::Search(..) | Location2::Recents) {
                    children.push(
                        menu_item(fl!("open-item-location"), Action::OpenItemLocation).into(),
                    );
                }
                children.push(divider::horizontal::light().into());
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
            } else {
                if dialog_kind.save() {
                    children.push(menu_item(fl!("new-folder"), Action::NewFolder).into());
                }
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                if !children.is_empty() {
                    children.push(divider::horizontal::light().into());
                }
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions2::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions2::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions2::Size));
            }
        }
        (_, Location2::Network(..)) => {
            if selected > 0 {
                if selected_dir == 1 && selected == 1 || selected_dir == 0 {
                    children.push(menu_item(fl!("open"), Action::Open).into());
                }
            } else {
                if tab.mode.multiple() {
                    children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
                }
                if !children.is_empty() {
                    children.push(divider::horizontal::light().into());
                }
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions2::Name));
                children.push(sort_item(fl!("sort-by-modified"), HeadingOptions2::Modified));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions2::Size));
            }
        }
        (_, Location2::Trash) => {
            if tab.mode.multiple() {
                children.push(menu_item(fl!("select-all"), Action::SelectAll).into());
            }
            if !children.is_empty() {
                children.push(divider::horizontal::light().into());
            }
            if selected > 0 {
                children.push(menu_item(fl!("show-details"), Action::Preview).into());
                children.push(divider::horizontal::light().into());
                children
                    .push(menu_item(fl!("restore-from-trash"), Action::RestoreFromTrash).into());
            } else {
                // TODO: Nested menu
                children.push(sort_item(fl!("sort-by-name"), HeadingOptions2::Name));
                children.push(sort_item(fl!("sort-by-trashed"), HeadingOptions2::TrashedOn));
                children.push(sort_item(fl!("sort-by-size"), HeadingOptions2::Size));
            }
        }
    }

    container(column::with_children(children))
        .padding(1)
        //TODO: move style to libcosmic
        .style(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Style {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: cosmic.radius_s().map(|x| x + 1.0).into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(360.0))
        .into()
}

pub fn context_menu_term<'a>(
    _config: &Config,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, Message> {
    use cosmic::widget::menu::menu_button;
    use cosmic::{
        iced::{
            widget::{column, horizontal_space},
            Background, Length,
        },
        iced_core::Border,
        widget
    };
        let find_key = |action: &Action| -> String {
        for (key_bind, key_action) in key_binds {
            if action == key_action {
                return key_bind.to_string();
            }
        }
        String::new()
    };

    let menu_item = |label, action| {
        let key = find_key(&action);
        menu_button(vec![
            widget::text(label).into(),
            horizontal_space().into(),
            widget::text(key).into(),
        ])
        .on_press(Message::TermContextAction(action))
    };

    widget::container(column!(
        menu_item(fl!("copy"), Action::CopyTerminal),
        menu_item(fl!("paste"), Action::PasteTerminal),
    ))
    .padding(1)
    //TODO: move style to libcosmic
    .style(|theme| {
        let cosmic = theme.cosmic();
        let component = &cosmic.background.component;
        widget::container::Style {
            icon_color: Some(component.on.into()),
            text_color: Some(component.on.into()),
            background: Some(Background::Color(component.base.into())),
            border: Border {
                radius: cosmic.radius_s().map(|x| x + 1.0).into(),
                width: 1.0,
                color: component.divider.into(),
            },
            ..Default::default()
        }
    })
    .width(Length::Fixed(240.0))
    .into()
}

pub fn dialog_menu1(
    tab: &Tab1,
    key_binds: &HashMap<KeyBind, Action>,
    show_details: bool,
) -> Element<'static, Message> {
    let (sort_name, sort_direction, _) = tab.sort_options();
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            None,
            sort_name == sort && sort_direction == dir,
            Action::SetSort(sort, dir),
        )
    };
    let in_trash = tab.location == Location1::Trash;

    let mut selected_gallery = 0;
    if let Some(items) = tab.items_opt() {
        for item in items.iter() {
            if item.selected && item.can_gallery() {
                selected_gallery += 1;
            }
        }
    };

    MenuBar::new(vec![
        menu::Tree::with_children(
            widget::button::icon(widget::icon::from_name(match tab.config.view {
                tab1::View::Grid => "view-grid-symbolic",
                tab1::View::List => "view-list-symbolic",
            }))
            // This prevents the button from being shown as insensitive
            .on_press(Message::None)
            .padding(8),
            menu::items(
                key_binds,
                vec![
                    menu::Item::CheckBox(
                        fl!("grid-view"),
                        None,
                        matches!(tab.config.view, tab1::View::Grid),
                        Action::TabViewGrid,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-view"),
                        None,
                        matches!(tab.config.view, tab1::View::List),
                        Action::TabViewList,
                    ),
                ],
            ),
        ),
        menu::Tree::with_children(
            widget::button::icon(widget::icon::from_name(if sort_direction {
                "view-sort-ascending-symbolic"
            } else {
                "view-sort-descending-symbolic"
            }))
            // This prevents the button from being shown as insensitive
            .on_press(Message::None)
            .padding(8),
            menu::items(
                key_binds,
                vec![
                    sort_item(fl!("sort-a-z"), tab1::HeadingOptions::Name, true),
                    sort_item(fl!("sort-z-a"), tab1::HeadingOptions::Name, false),
                    sort_item(
                        fl!("sort-newest-first"),
                        if in_trash {
                            tab1::HeadingOptions::TrashedOn
                        } else {
                            tab1::HeadingOptions::Modified
                        },
                        false,
                    ),
                    sort_item(
                        fl!("sort-oldest-first"),
                        if in_trash {
                            tab1::HeadingOptions::TrashedOn
                        } else {
                            tab1::HeadingOptions::Modified
                        },
                        true,
                    ),
                    sort_item(
                        fl!("sort-smallest-to-largest"),
                        tab1::HeadingOptions::Size,
                        true,
                    ),
                    sort_item(
                        fl!("sort-largest-to-smallest"),
                        tab1::HeadingOptions::Size,
                        false,
                    ),
                    //TODO: sort by type
                ],
            ),
        ),
        menu::Tree::with_children(
            widget::button::icon(widget::icon::from_name("view-more-symbolic"))
                // This prevents the button from being shown as insensitive
                .on_press(Message::None)
                .padding(8),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("zoom-in"), None, Action::ZoomIn),
                    menu::Item::Button(fl!("default-size"), None, Action::ZoomDefault),
                    menu::Item::Button(fl!("zoom-out"), None, Action::ZoomOut),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("show-hidden-files"),
                        None,
                        tab.config.show_hidden,
                        Action::ToggleShowHidden,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-directories-first"),
                        None,
                        tab.config.folders_first,
                        Action::ToggleFoldersFirst,
                    ),
                    menu::Item::CheckBox(fl!("show-details"), None, show_details, Action::Preview),
                    menu::Item::Divider,
                    menu_button_optional(
                        fl!("gallery-preview"),
                        Action::Gallery,
                        selected_gallery > 0,
                    ),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(theme::active().cosmic().spacing.space_xxxs.into())
    .into()
}

pub fn menu_bar<'a>(
    tab_opt: Option<&Tab1>,
    config: &Config,
    key_binds: &HashMap<KeyBind, Action>,
) -> Element<'a, Message> {
    let sort_options = tab_opt.map(|tab| tab.sort_options());
    let sort_item = |label, sort, dir| {
        menu::Item::CheckBox(
            label,
            None,
            sort_options.map_or(false, |(sort_name, sort_direction, _)| {
                sort_name == sort && sort_direction == dir
            }),
            Action::SetSort(sort, dir),
        )
    };
    let in_trash = tab_opt.map_or(false, |tab| tab.location == Location1::Trash);

    let mut selected_dir = 0;
    let mut selected = 0;
    let mut selected_gallery = 0;
    if let Some(items) = tab_opt.and_then(|tab| tab.items_opt()) {
        for item in items.iter() {
            if item.selected {
                selected += 1;
                if item.metadata.is_dir() {
                    selected_dir += 1;
                }
                if item.can_gallery() {
                    selected_gallery += 1;
                }
            }
        }
    };

    MenuBar::new(vec![
        menu::Tree::with_children(
            menu::root(fl!("file")),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("new-tab"), None, Action::TabNew),
                    menu::Item::Button(fl!("copy-tab"), None, Action::TabNew),
                    menu::Item::Button(fl!("move-tab"), None, Action::TabNew),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("new-window"), None, Action::WindowNew),
                    menu::Item::Button(fl!("new-folder"), None, Action::NewFolder),
                    menu::Item::Button(fl!("new-file"), None, Action::NewFile),
                    menu_button_optional(
                        fl!("open"),
                        Action::Open,
                        (selected > 0 && selected_dir == 0) || (selected_dir == 1 && selected == 1),
                    ),
                    menu_button_optional(fl!("menu-open-with"), Action::OpenWith, selected == 1),
                    menu::Item::Divider,
                    menu_button_optional(fl!("rename"), Action::F2Rename, selected > 0),
                    menu_button_optional(fl!("f5-copy"), Action::F5Copy, selected > 0),
                    menu_button_optional(fl!("f6-move"), Action::F6Move, selected > 0),
                    menu::Item::Divider,
                    menu_button_optional(fl!("add-to-sidebar"), Action::AddToSidebar, selected > 0),
                    menu::Item::Divider,
                    menu_button_optional(fl!("move-to-trash"), Action::MoveToTrash, selected > 0),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("close-tab"), None, Action::TabClose),
                    menu::Item::Button(fl!("quit"), None, Action::WindowClose),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("edit")),
            menu::items(
                key_binds,
                vec![
                    menu_button_optional(fl!("cut"), Action::Cut, selected > 0),
                    menu_button_optional(fl!("copy"), Action::Copy, selected > 0),
                    menu_button_optional(fl!("paste"), Action::Paste, selected > 0),
                    menu::Item::Button(fl!("select-all"), None, Action::SelectAll),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("history"), None, Action::EditHistory),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                key_binds,
                vec![
                    menu::Item::Button(fl!("zoom-in"), None, Action::ZoomIn),
                    menu::Item::Button(fl!("default-size"), None, Action::ZoomDefault),
                    menu::Item::Button(fl!("zoom-out"), None, Action::ZoomOut),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("grid-view"),
                        None,
                        tab_opt.map_or(false, |tab| matches!(tab.config.view, tab1::View::Grid)),
                        Action::TabViewGrid,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-view"),
                        None,
                        tab_opt.map_or(false, |tab| matches!(tab.config.view, tab1::View::List)),
                        Action::TabViewList,
                    ),
                    menu::Item::Divider,
                    menu::Item::CheckBox(
                        fl!("show-hidden-files"),
                        None,
                        tab_opt.map_or(false, |tab| tab.config.show_hidden),
                        Action::ToggleShowHidden,
                    ),
                    menu::Item::CheckBox(
                        fl!("list-directories-first"),
                        None,
                        tab_opt.map_or(false, |tab| tab.config.folders_first),
                        Action::ToggleFoldersFirst,
                    ),
                    menu::Item::CheckBox(
                        fl!("show-details"),
                        None,
                        config.show_details,
                        Action::Preview,
                    ),
                    menu::Item::Divider,
                    menu_button_optional(
                        fl!("gallery-preview"),
                        Action::Gallery,
                        selected_gallery > 0,
                    ),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-settings"), None, Action::Settings),
                    menu::Item::Divider,
                    menu::Item::Button(fl!("menu-about"), None, Action::About),
                ],
            ),
        ),
        menu::Tree::with_children(
            menu::root(fl!("sort")),
            menu::items(
                key_binds,
                vec![
                    sort_item(fl!("sort-a-z"), tab1::HeadingOptions::Name, true),
                    sort_item(fl!("sort-z-a"), tab1::HeadingOptions::Name, false),
                    sort_item(
                        fl!("sort-newest-first"),
                        if in_trash {
                            tab1::HeadingOptions::TrashedOn
                        } else {
                            tab1::HeadingOptions::Modified
                        },
                        false,
                    ),
                    sort_item(
                        fl!("sort-oldest-first"),
                        if in_trash {
                            tab1::HeadingOptions::TrashedOn
                        } else {
                            tab1::HeadingOptions::Modified
                        },
                        true,
                    ),
                    sort_item(
                        fl!("sort-smallest-to-largest"),
                        tab1::HeadingOptions::Size,
                        true,
                    ),
                    sort_item(
                        fl!("sort-largest-to-smallest"),
                        tab1::HeadingOptions::Size,
                        false,
                    ),
                    //TODO: sort by type
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(360))
    .spacing(theme::active().cosmic().spacing.space_xxxs.into())
    .into()
}

pub fn location_context_menu1<'a>(ancestor_index: usize) -> Element<'a, tab1::Message> {
    //TODO: only add some of these when in App mode
    let children = vec![
        menu_button!(text::body(fl!("open-in-new-tab")))
            .on_press(tab1::Message::LocationMenuAction(
                LocationMenuAction1::OpenInNewTab(ancestor_index),
            ))
            .into(),
        menu_button!(text::body(fl!("open-in-new-window")))
            .on_press(tab1::Message::LocationMenuAction(
                LocationMenuAction1::OpenInNewWindow(ancestor_index),
            ))
            .into(),
        divider::horizontal::light().into(),
        menu_button!(text::body(fl!("show-details")))
            .on_press(tab1::Message::LocationMenuAction(
                LocationMenuAction1::Preview(ancestor_index),
            ))
            .into(),
        divider::horizontal::light().into(),
        menu_button!(text::body(fl!("add-to-sidebar")))
            .on_press(tab1::Message::LocationMenuAction(
                LocationMenuAction1::AddToSidebar(ancestor_index),
            ))
            .into(),
    ];

    container(column::with_children(children))
        .padding(1)
        .style(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Style {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: cosmic.radius_s().map(|x| x + 1.0).into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(360.0))
        .into()
}

pub fn location_context_menu2<'a>(ancestor_index: usize) -> Element<'a, tab2::Message> {
    //TODO: only add some of these when in App mode
    let children = vec![
        menu_button!(text::body(fl!("open-in-new-tab")))
            .on_press(tab2::Message::LocationMenuAction(
                LocationMenuAction2::OpenInNewTab(ancestor_index),
            ))
            .into(),
        menu_button!(text::body(fl!("open-in-new-window")))
            .on_press(tab2::Message::LocationMenuAction(
                LocationMenuAction2::OpenInNewWindow(ancestor_index),
            ))
            .into(),
        divider::horizontal::light().into(),
        menu_button!(text::body(fl!("show-details")))
            .on_press(tab2::Message::LocationMenuAction(
                LocationMenuAction2::Preview(ancestor_index),
            ))
            .into(),
        divider::horizontal::light().into(),
        menu_button!(text::body(fl!("add-to-sidebar")))
            .on_press(tab2::Message::LocationMenuAction(
                LocationMenuAction2::AddToSidebar(ancestor_index),
            ))
            .into(),
    ];

    container(column::with_children(children))
        .padding(1)
        .style(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background.component;
            container::Style {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: Border {
                    radius: cosmic.radius_s().map(|x| x + 1.0).into(),
                    width: 1.0,
                    color: component.divider.into(),
                },
                ..Default::default()
            }
        })
        .width(Length::Fixed(360.0))
        .into()
}
