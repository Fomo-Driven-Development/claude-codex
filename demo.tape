# VHS Tape file for Codex Demo
# This tape demonstrates the basic navigation features of codex

# Set up the terminal
Output demo.gif
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"

# Launch codex
Type "codex-rs/target/release/codex"
Enter
Sleep 2s

# Press / to enter search mode
Type "/"
Sleep 1s

# Press Up arrow 10 times to navigate through history
Up
Sleep 300ms
Up
Sleep 300ms
Up
Sleep 300ms
Tab
Sleep 2s
Type "gpt-5-codex"
Sleep 2s
Enter
Sleep 10s

# Press Ctrl+C twice to exit
Ctrl+c
Sleep 500ms
Ctrl+c
Sleep 1s
