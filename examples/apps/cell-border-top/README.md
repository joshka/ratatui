# Cell border top

A [btop]-style system monitor drawn with the experimental [cell border protocol]. Box outlines,
labelled rules, nested panes, column rules, graph gridlines, the selection ring and the scrollbar
all sit on cell edges, so they take no columns or rows, and btop's information fits in less space.
On terminals without the protocol it falls back to box-drawing characters.

```shell
cargo run -p cell-border-top
```

It reads the system with [sysinfo]. On macOS it also uses [libproc] for thread counts, and on
Apple Silicon [macmon] for GPU use, power, temperatures and cluster frequencies. The graphs start
with made-up history so the screen is full from the first frame. This example needs Rust 1.95,
newer than the workspace minimum, because of sysinfo.

Keys:

- `1`–`4`: show or hide the cpu, mem, net and proc boxes
- `Esc`: menu, or close the open popup or details; `h` or `?`: help
- `↑`/`↓`, `j`/`k`, page up/down, the mouse wheel or a click: select a process
- `Enter`: details for the selected process
- `←`/`→`: change the sort column; `c`/`m`: sort by cpu or memory; `r`: reverse; `t`: tree
- `f` or `/`: filter processes; `p`: pause the process list
- `i`: next network interface; `-`/`+`: refresh slower or faster
- `b`: switch between cell borders and box drawing; `g`: shade alternate cells; `o`: fill outside
  rounded corners
- `q`: quit

Labels in the box headers and footers are clickable, like btop's buttons.

Set `NO_CELL_BORDERS=1` to skip detection and use box-drawing characters.

See [`CELL-BORDERS.md`](../../../CELL-BORDERS.md) for a terminal that supports the protocol.

[btop]: https://github.com/aristocratos/btop
[cell border protocol]: https://github.com/joshka/ghostty/blob/joshka/cell-borders/specs/cell-border-protocol.md
[sysinfo]: https://crates.io/crates/sysinfo
[libproc]: https://crates.io/crates/libproc
[macmon]: https://crates.io/crates/macmon
