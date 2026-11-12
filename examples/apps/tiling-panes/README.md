# Tiling panes demo

A multiplexer-style shell drawn with the experimental [cell border protocol]: tabbed panes that
tile like a window manager, a menu bar, a context menu and an about dialog. Drag a pane with the
mouse and drop it on the left, right, top or bottom of another pane to split there, or on its
centre to swap the two. The drop target is shaded and outlined while you hover.

Every line sits on the edges of cells, so it takes no columns or rows: tabs that join the pane
below them, one-row buttons, a one-cell checkbox, underline-only inputs, table column rules with
no separator columns, and a focus ring that wins every shared edge. On terminals without the
protocol the demo falls back to box-drawing characters.

```shell
cargo run -p tiling-panes
```

Keys:

- `n` / `v`: split the focused pane right or down
- `x`: close the focused pane
- right click: pane menu; `Esc` closes menus
- `?`: about dialog
- `b`: switch between cell borders and box drawing
- `g`: shade alternate cells, to show where the cell edges are
- `o`: fill outside rounded corners (on by default when the terminal supports it)
- `q`: quit

Set `NO_CELL_BORDERS=1` to skip detection and use box-drawing characters.

See [`CELL-BORDERS.md`](../../../CELL-BORDERS.md) for a terminal that supports the protocol.

[cell border protocol]: https://github.com/joshka/ghostty/blob/joshka/cell-borders/specs/cell-border-protocol.md
