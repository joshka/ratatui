# This is a vhs script. See https://github.com/charmbracelet/vhs for more info.
# To run this script, install vhs and run `vhs ./examples/demo.tape`
Output "target/demo2.gif"
Set Theme "OceanicMaterial"
# Github social preview size (1280x640 with 80px padding)
Set Width 1280
Set Height 640
Set Padding 80
Hide
Type "cargo run --example demo2"
Enter
Set TypingSpeed 200ms
Sleep 2s
Show
Sleep 2s
Down 10
Right
Sleep 2s
Down 20
Right
Sleep 2s
Down@50ms 80
Right
Sleep 3s
