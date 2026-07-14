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

# Open file-switcher dropdown from the path title (o = keyboard; mouse click also works)
Type "o"
Sleep 2s

# Go to parent directory with Left arrow (../ is also clickable)
Left
Sleep 2s

# Enter project-a again and open utils.py
Down@150ms 4
Sleep 500ms
Enter
Sleep 1s
Down@150ms 1
Sleep 400ms
Enter
Sleep 2s

# Switch back to calculator_old.py via the path dropdown
Type "o"
Sleep 1.2s
Up@200ms 1
Sleep 400ms
Enter
Sleep 1.5s

# Close left panel file → file browser (cwd: project-a/)
Type "q"
Sleep 2s

# Jump to calculator_old.py by typing a path (paste / click also work)
Type "/"
Type "/workspace/demo/project-a/calculator_old.py"
Enter
Sleep 2s

# Scroll the restored side-by-side diff
Down@150ms 6
Sleep 1s

# Switch to the right panel
Tab
Sleep 1s

# Close right panel file → file browser (cwd: project-b/)
Type "q"
Sleep 2s

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
