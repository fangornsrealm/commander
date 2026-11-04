# Changelog

All notable changes to Commander will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Open Bugs

- Open With gives a list of apps but if the list of programs is too long no option to end the dialog

### Added

- Store open tabs in config and restore on app startup
- Configuration of visible panels (button row, terminal, second file panel)
- Copy and paste from and to the terminal

### Fixed

- File watchers now rescan the location on change (dependency update problem)
- Ctrl+R now correctly triggers a refresh
- Navigation panel can now create new tabs with locations and change the location of the current tab
- Copy and move of tabs between panes works again
- Drag and drop from Commander to other applications now works
- Drag and drop of files into Commander now works (enabled the missing `wayland` feature)
- Scroll handle is now grabbable
- Settings for optional panels now correctly activate/deactivate the optional panels
- Tab scrollable no longer jumps back to the start at every update
- Long lists of files no longer crash the app with a `u16` overflow
- Function keys register reliably
- File operations now execute correctly
- View options now apply to both the left and right panels
- Terminal accepts middle-click paste
- Terminal has a context menu with copy/paste
- Terminal accepts Shift+Ctrl+C / Shift+Ctrl+V as copy/paste

## [0.1.0] - Initial Release

### Added

- Dual panel file manager for the COSMIC desktop environment
- Two file panels with copying/moving files and tabs between them
- Embedded terminal for running commands
- All features inherited from COSMIC Files
- Debian and RPM packaging
- Flatpak support
- Internationalization (en, de, sv)
- Multiple color schemes for the terminal
- Compression and extraction support (tar, zip, bzip2, lzma, zstd)
- GVFS / mounted drive support
- Desktop mode (file manager on the desktop)
