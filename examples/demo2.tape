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
Sleep 2s
Show
# About screen
Sleep 3s
Tab
# Email
Sleep 3s
Tab
# Trace route
Sleep 3s
Tab
# Misc
Sleep 3s
Tab