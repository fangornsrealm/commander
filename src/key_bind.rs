use cosmic::{
    iced::keyboard::Key,
    iced::core::keyboard::key::Named,
    widget::menu::key_bind::{KeyBind, Modifier},
};
use std::collections::HashMap;

use crate::{app::Action, tab1};

//TODO: load from config
pub fn key_binds(mode: &tab1::Mode) -> HashMap<KeyBind, Action> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                Action::$action,
            );
        }};
    }

    // Common keys
    bind!([], Key::Character(" ".into()), Gallery);
    bind!([Shift], Key::Named(Named::Tab), SwapPanels);
    bind!([], Key::Named(Named::F2), F2Rename);
    bind!([], Key::Named(Named::F3), F3View);
    bind!([], Key::Named(Named::F4), F4Edit);
    bind!([], Key::Named(Named::F5), F5Copy);
    bind!([], Key::Named(Named::F6), F6Move);
    bind!([], Key::Named(Named::F7), F7Mkdir);
    bind!([], Key::Named(Named::F8), F8Delete);
    bind!([], Key::Named(Named::F9), F9Terminal);
    bind!([], Key::Named(Named::F10), F10Quit);

    bind!([], Key::Named(Named::ArrowDown), ItemDown);
    bind!([], Key::Named(Named::ArrowLeft), ItemLeft);
    bind!([], Key::Named(Named::ArrowRight), ItemRight);
    bind!([], Key::Named(Named::ArrowUp), ItemUp);
    bind!([], Key::Named(Named::Home), SelectFirst);
    bind!([], Key::Named(Named::End), SelectLast);
    bind!([Shift], Key::Named(Named::ArrowDown), ItemDown);
    bind!([Shift], Key::Named(Named::ArrowLeft), ItemLeft);
    bind!([Shift], Key::Named(Named::ArrowRight), ItemRight);
    bind!([Shift], Key::Named(Named::ArrowUp), ItemUp);
    bind!([Shift], Key::Named(Named::Home), SelectFirst);
    bind!([Shift], Key::Named(Named::End), SelectLast);
    bind!([Ctrl, Shift], Key::Character("n".into()), NewFolder);
    bind!([], Key::Named(Named::Enter), Open);
    bind!([Ctrl], Key::Character(" ".into()), Preview);
    bind!([Ctrl], Key::Character("h".into()), ToggleShowHidden);
    bind!([Ctrl], Key::Character("a".into()), SelectAll);
    bind!([Ctrl], Key::Character("=".into()), ZoomIn);
    bind!([Ctrl], Key::Character("+".into()), ZoomIn);
    bind!([Ctrl], Key::Character("0".into()), ZoomDefault);
    bind!([Ctrl], Key::Character("-".into()), ZoomOut);
    bind!([Ctrl], Key::Character("_".into()), ZoomOut);

    // App-only keys
    if matches!(mode, tab1::Mode::App) {
        bind!([Ctrl], Key::Character("d".into()), AddToSidebar);
        bind!([Ctrl], Key::Named(Named::Enter), OpenInNewTab);
        bind!([Ctrl], Key::Named(Named::F5), TabRescan);
        bind!([Ctrl], Key::Character("r".into()), TabRescan);
        bind!([Ctrl], Key::Character(",".into()), Settings);
        bind!([Ctrl], Key::Character("w".into()), TabClose);
        bind!([Ctrl], Key::Character("s".into()), SwapPanels);
        bind!([Ctrl], Key::Character("t".into()), TabNew);
        bind!([Ctrl], Key::Named(Named::Tab), TabNext);
        bind!([Ctrl, Shift], Key::Named(Named::Tab), TabPrev);
        bind!([Ctrl], Key::Character("q".into()), WindowClose);
        bind!([Ctrl], Key::Character("n".into()), WindowNew);
        //bind!([Ctrl], Key::Character("r".into()), TabReload);
    }

    // App and desktop only keys
    if matches!(mode, tab1::Mode::App | tab1::Mode::Desktop) {
        bind!([Ctrl], Key::Character("c".into()), Copy);
        bind!([Ctrl], Key::Character("x".into()), Cut);
        bind!([], Key::Named(Named::Delete), MoveToTrash);
        bind!([Shift], Key::Named(Named::Enter), OpenInNewWindow);
        bind!([Ctrl], Key::Character("v".into()), Paste);
        bind!([], Key::Named(Named::F2), Rename);
    }

    // App and dialog only keys
    if matches!(mode, tab1::Mode::App | tab1::Mode::Dialog(_)) {
        bind!([Ctrl], Key::Character("l".into()), EditLocation);
        bind!([Alt], Key::Named(Named::ArrowRight), HistoryNext);
        bind!([Alt], Key::Named(Named::ArrowLeft), HistoryPrevious);
        bind!([], Key::Named(Named::Backspace), HistoryPrevious);
        bind!([Alt], Key::Named(Named::ArrowUp), LocationUp);
        bind!([Ctrl], Key::Character("f".into()), SearchActivate);
    }

    key_binds
}

pub fn key_binds_terminal() -> HashMap<KeyBind, Action> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                Action::$action,
            );
        }};
    }

    bind!([Shift], Key::Named(Named::Tab), SwapPanels);
    bind!([], Key::Named(Named::F2), F2Rename);
    bind!([], Key::Named(Named::F3), F3View);
    bind!([], Key::Named(Named::F4), F4Edit);
    bind!([], Key::Named(Named::F5), F5Copy);
    bind!([], Key::Named(Named::F6), F6Move);
    bind!([], Key::Named(Named::F7), F7Mkdir);
    bind!([], Key::Named(Named::F8), F8Delete);
    bind!([], Key::Named(Named::F9), F9Terminal);
    bind!([], Key::Named(Named::F10), F10Quit);

    // Standard key bindings
    bind!([Ctrl, Shift], Key::Character("A".into()), SelectAll);
    bind!([Ctrl, Shift], Key::Character("C".into()), CopyTerminal);
    bind!([Ctrl], Key::Character("c".into()), CopyOrSigint);
    bind!([Ctrl, Shift], Key::Character("V".into()), PasteTerminal);
    bind!([Shift], Key::Named(Named::Insert), PastePrimaryTerminal);

    // Ctrl+Tab and Ctrl+Shift+Tab cycle through tabs
    // Ctrl+Tab is not a special key for terminals and is free to use
    bind!([Ctrl], Key::Named(Named::Tab), TabNext);
    bind!([Ctrl, Shift], Key::Named(Named::Tab), TabPrev);

    // Ctrl+0, Ctrl+-, and Ctrl+= are not special keys for terminals and are free to use
    bind!([Ctrl], Key::Character("-".into()), ZoomOut);
    bind!([Ctrl], Key::Character("=".into()), ZoomIn);
    bind!([Ctrl], Key::Character("+".into()), ZoomIn);

    // CTRL+Alt+L clears the scrollback.
    bind!([Ctrl, Alt], Key::Character("L".into()), ClearScrollback);

    key_binds
}
