# VHS tape file for sigye demo
# https://github.com/charmbracelet/vhs

Output demo.gif

Set Shell "zsh"
Set FontSize 18
Set Width 1600
Set Height 800
Set Theme "Dracula"

# Start the clock
Type "sigye"
Enter
Sleep 2s

# Cycle through color themes
Type "c"
Sleep 1.5s
Type "c"
Sleep 1.5s
Type "c"
Sleep 1.5s
Type "c"
Sleep 1.5s

# Toggle 12/24 hour format
Type "t"
Sleep 2s

# Cycle a few more themes
Type "c"
Sleep 1.5s
Type "c"
Sleep 1.5s

# Quit
Type "q"
Sleep 500ms
