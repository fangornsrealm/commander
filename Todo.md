# Todo log

## Closed bugs

- Watchers do not rescan the location on change -> dependency update problem
- Ctrl-r does not refresh
- The nav-panel cannot create new tabs with locations or change the location of the current tab.
- Copy and move of tabs between panes does not work any more
- drag'n drop from Commander to other Apps is not working
- make the scroll-handle grabbable
- settings for optional panels do not de-/activate the optional panels
- tab scrollable jumps back to the start at every update
- long lists of files crash the app with an u16 overflow
- Fkeys don't register sometimes.
- File operations do not execute
- View options only apply to the left panel
- Terminal does not accept middle click paste
- Terminal does not have context menu copy paste
- Terminal does not accept key_binding Shift-Ctrl-C, Shift-Ctrl-V as copy paste

## Open bugs

- switching between the tabs does not work
- activation of a panel does not send key-presses to it but still Ctrl-R opens the backwards search
- zoom options do not work at all
- drag and drop with Commander as destination does not work for the tabs
- drag and drop with Commander as destination does not work for the terminal for String data

## implemented features - additional to COSMIC files

- store open tabs to config and restore on app startup
- allow configuration of visible panels (Button-row, Terminal, second file panel) and store it in config
- implement copy and paste from and to the Terminal

## open features

- drop files into commander panels
- drop paths / commands into the commander terminal
- switching tabs changes the directory in the terminal
- show number and size of files in the current directory for each tab
- save window size in config and update config on resize of the window

