# theme-egui

egui/epaint lowering for [`theme-kit`](https://github.com/WyattAu/theme-kit)
(ADR-0004 §2.3, THM-008) — the adapter that makes the estate token contract
real in `chisel-doc` and any egui app.

```rust
use theme_egui::seed_visuals;

let visuals = seed_visuals()?;   // the shroom-derived midnight palette
// … then: ctx.set_visuals(visuals) — done, once, at mount.
```

## The adapter contract

Narrow and machine-checked: it **places** resolved roles into egui's
`Visuals` and performs **no colour maths**. Every colour in the output is
byte-identical to the role `theme-kit`'s gate approved — which is asserted
per field in `tests/lowering.rs`:

| egui field | estate role |
|---|---|
| `window_fill`, `extreme_bg_color` | `surface` |
| `panel_fill`, `faint_bg_color` | `surface-raised` |
| `override_text_color` | `on-surface` |
| `weak_text_color` | `on-surface-muted` |
| `hyperlink_color` | `accent` |
| `warn_fg_color` / `error_fg_color` | `warn` / `error` |
| `selection.bg_fill` | `selection` |
| `widgets.active.bg_fill` + `fg_stroke` | `accent` + `on-accent` |
| `widgets.hovered.bg_stroke`, `selection.stroke` | `focus-ring` |
| `window_stroke`, `widgets.noninteractive.bg_stroke` | `border` |

Unset roles are errors, never invented colours. Dark vs light is decided by
the surface's lightness, mirroring `theme-kit`'s VS Code emitter.

## Why it is its own crate

Adapters are the only place in the L1/L2 half that names a toolkit
(ADR-0004 §2.3). One crate each — `theme-ratatui`, `theme-leptos`,
`theme-slint` — so swapping a toolkit never touches `theme-kit`, and the
cross-adapter property test ("same role → same RGBA everywhere") has a place
to live.

Tests: 6 + doctest, clippy `-D warnings` clean. Targets egui 0.37.
Licensed under MIT OR Apache-2.0.
