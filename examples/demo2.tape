# This is a vhs script. See https://github.com/charmbracelet/vhs for more info.
# To run this script, install vhs and run `vhs ./examples/demo.tape`
Output "target/demo2.gif"
Set Theme "OceanicMaterial"
# Github social preview size (1280x640 with 80px padding)
Set Width 1280
Set Height 640
Set Padding 80
Set PlaybackSpeed 0.5
Hide
Type "cargo run --example demo2"
Enter
Sleep 2s
Show
Sleep 3s
Down@0.25s 10
Sleep 1s
Right
Sleep 1s
Right
Sleep 2s
