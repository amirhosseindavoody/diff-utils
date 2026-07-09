Output diff-tool-python-demo.raw.mp4

Set Shell "bash"
Set FontSize 20
Set Width 1400
Set Height 800
Set Padding 40
Set Theme "Catppuccin Mocha"
Set TypingSpeed 0ms

Env NO_COLOR ""
Env COLORTERM "truecolor"
Env TERM "xterm-256color"

Hide
Type "cd /workspace/demo"
Enter
Show

Type "/workspace/target/release/diff-tool /workspace/demo/project-a/calculator_old.py /workspace/demo/project-b/calculator_new.py"
Enter
Sleep 2.5s

# Scroll through the Python diff
Down@200ms 8
Sleep 1s

# Open sibling-file switcher from the path title (o = keyboard; mouse click also works)
Type "o"
Sleep 2.5s

# Move to utils.py and open it on the left panel
Down@200ms 1
Sleep 800ms
Enter
Sleep 2.5s

# Switch back to calculator_old.py via the path dropdown
Type "o"
Sleep 1.5s
Up@200ms 1
Sleep 500ms
Enter
Sleep 2s

# Close left panel file → file browser (cwd: project-a/)
Type "q"
Sleep 2.5s

# Jump to calculator_old.py by typing a path (paste works the same way)
Type "/"
Type "/workspace/demo/project-a/calculator_old.py"
Enter
Sleep 2.5s

# Scroll the restored side-by-side diff
Down@150ms 6
Sleep 1s

# Switch to the right panel
Tab
Sleep 1s

# Close right panel file → file browser (cwd: project-b/)
Type "q"
Sleep 2.5s

# Go up to demo/ and select diff-demo.tape
Type "h"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "j"
Sleep 1s

# Open the tape file on the right panel
Type "l"
Sleep 2.5s

# Quit the app
Type "Q"
