# This is a vhs script. See https://github.com/charmbracelet/vhs for more info.
# To run this script, install vhs and run `vhs ./examples/demo.tape`
Output "target/demo2.mp4"
Set Theme "OceanicMaterial"
# Github social preview size (1280x640 with 80px padding)
Set Width 1280
Set Height 640
Set Padding 80
Hide
Type "cargo run --example demo2"
Enter
Sleep 2s
Set TypingSpeed 1s
Show
# Recipe screen
Sleep 3s
Right
# Words
Sleep 3s
Right
# Bars
Sleep 2s
Right
# Chart
Sleep 1s
Right
# Email
Down 4
Right
# Traceroute
Sleep 3s