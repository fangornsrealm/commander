//! Right-pane tab — a thin compatibility shim over the shared [`crate::tab`] module.
//!
//! Historically this was a full copy of the (now-generic) tab implementation. It is kept
//! as a re-export module so that existing `tab2::X` references across the codebase
//! (e.g. `tab2::Tab`, `tab2::Item`, `tab2::Message`, `tab2::Location`) continue to work
//! unchanged. `tab1` is an identical shim; both panels now share one implementation.

use super::tab;

pub use tab::{
    folder_icon, folder_icon_symbolic, item_from_entry, item_from_path, parse_desktop_file,
    respond_to_scroll_direction, scan_desktop, scan_network, scan_path, scan_recents, scan_search,
    scan_trash, trash_entries, trash_icon, trash_icon_symbolic, ArcElementWrapper, Command,
    DirSize, EditLocation, HeadingOptions, Item, ItemMetadata, ItemThumbnail, Location,
    LocationMenuAction, Message, Mode, SearchContextWrapper, Tab, TaskWrapper, View,
    DOUBLE_CLICK_DURATION, HOVER_DURATION,
};
