# egui-winit Clipboard Soundness Bug Demo

Minimal reproduction of a soundness bug in `egui-winit`'s Wayland clipboard handling.

## The Bug

`egui_winit::clipboard::Clipboard::new()` is a **safe** function that internally calls the **unsafe** `smithay_clipboard::Clipboard::new()`.

smithay-clipboard's safety requirement states:
> `display` must be a valid `*mut wl_display` pointer, and it must remain valid for as long as `Clipboard` object is alive.

egui-winit **cannot guarantee this invariant** because the caller controls the window/display lifetime. This means 100% safe Rust code can trigger undefined behavior (segfault).

## Running

```bash
cargo run --release
```

The window will auto-close after 1 second and segfault on Wayland.

## Relevant Code

- [smithay-clipboard `Clipboard::new` (unsafe)](https://github.com/Smithay/smithay-clipboard/blob/26c2f53f15f6bdc4f41a442d0ae2c2d63bbc617c/src/lib.rs#L34)
- [egui-winit `Clipboard::new` (safe wrapper)](https://github.com/emilk/egui/blob/8b8595b45b4c283a2a654ada081342079170e3ab/crates/egui-winit/src/clipboard.rs#L31)
- [egui-winit calls unsafe smithay function](https://github.com/emilk/egui/blob/8b8595b45b4c283a2a654ada081342079170e3ab/crates/egui-winit/src/clipboard.rs#L188)

## Note on Field Ordering

Swapping the field order in the `App` struct (putting `clipboard` before `window`) would prevent the segfault because Rust drops fields in declaration order. However, a safe API should not segfault based on field ordering, and the compiler provides no warning about this.

---

*This repository was created in cooperation with [Claude Code](https://claude.ai).*
