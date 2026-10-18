//! theme-egui — egui/epaint lowering for [`theme-kit`] (ADR-0004 §2.3,
//! THM-008).
//!
//! The adapter's contract is narrow and testable: it **places** resolved
//! roles into egui's `Visuals` and performs **no colour maths** — every
//! colour in the output is byte-identical to the role the gate approved.
//! Compositing and contrast decisions stay in `theme-kit`, where they are
//! audited.
//!
//! ```
//! use theme_egui::to_visuals;
//! use theme_kit::{audit, Role, TokenGraph};
//!
//! let mut theme = TokenGraph::new("midnight");
//! theme.set_hex(Role::Surface, "#0d1117").unwrap();
//! theme.set_hex(Role::SurfaceRaised, "#161b22").unwrap();
//! theme.set_hex(Role::OnSurface, "#e6edf3").unwrap();
//! theme.set_hex(Role::OnSurfaceMuted, "#9da7b3").unwrap();
//! theme.set_hex(Role::Accent, "#1f6feb").unwrap();
//! theme.set_hex(Role::OnAccent, "#ffffff").unwrap();
//! theme.set_hex(Role::Warn, "#d29922").unwrap();
//! theme.set_hex(Role::Error, "#f85149").unwrap();
//! theme.set_hex(Role::Success, "#3fb950").unwrap();
//! theme.set_hex(Role::Selection, "#1f3a5f").unwrap();
//! theme.set_hex(Role::Border, "#8b949e").unwrap();
//! theme.set_hex(Role::FocusRing, "#58a6ff").unwrap();
//!
//! audit(&theme, None).expect("the gate runs before lowering, always");
//! let visuals = to_visuals(&theme)?;
//! assert_eq!(
//!     visuals.window_fill,
//!     egui::Color32::from_rgb(0x0d, 0x11, 0x17)
//! );
//! # Ok::<(), theme_kit::ThemeError>(())
//! ```
//!
//! # Layer
//!
//! L2 — domain. Adapters are the *only* place in the L1/L2 half that names
//! a toolkit, deliberately isolated one crate per toolkit so swapping egui
//! for Slint, Leptos, or ratatui never touches `theme-kit`.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

use color_kit::Rgba8;
use egui::style::WidgetVisuals;
use egui::{Color32, CornerRadius, Stroke, Visuals};
use theme_kit::{Role, ThemeError, TokenGraph};

/// The crate's version, for callers that report it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn color32(c: Rgba8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

fn role(theme: &TokenGraph, r: Role) -> Result<Color32, ThemeError> {
    let view = theme.require(r)?.to_srgb8()?;
    Ok(color32(view))
}

fn stroke(color: Color32, width: f32) -> Stroke {
    Stroke::new(width, color)
}

fn widget_visuals(
    bg: Color32,
    bg_stroke: Stroke,
    fg_stroke: Stroke,
) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke,
        corner_radius: CornerRadius::same(3),
        fg_stroke,
        expansion: 0.0,
    }
}

/// Lower a gated theme into egui's `Visuals`.
///
/// The base is egui's own `dark()`/`light()` defaults — chosen by the
/// surface's lightness — with estate roles placed over them. Unset roles are
/// errors: an adapter that invents colours breaks the token contract.
///
/// # Errors
/// [`ThemeError::UnsetRole`] when a role the adapter needs was never set;
/// colour errors propagate from `color-kit`.
pub fn to_visuals(theme: &TokenGraph) -> Result<Visuals, ThemeError> {
    let surface = theme.require(Role::Surface)?;
    let surface_l = surface.oklab()?.l;
    let mut v = if surface_l < 0.5 {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    // Surfaces and text — direct role placement, no maths.
    v.window_fill = role(theme, Role::Surface)?;
    v.panel_fill = role(theme, Role::SurfaceRaised)?;
    v.extreme_bg_color = role(theme, Role::Surface)?;
    v.faint_bg_color = role(theme, Role::SurfaceRaised)?;
    v.override_text_color = Some(role(theme, Role::OnSurface)?);
    v.weak_text_color = Some(role(theme, Role::OnSurfaceMuted)?);
    v.hyperlink_color = role(theme, Role::Accent)?;
    v.warn_fg_color = role(theme, Role::Warn)?;
    v.error_fg_color = role(theme, Role::Error)?;
    v.window_stroke = stroke(role(theme, Role::Border)?, 1.0);
    v.striped = false;

    let selection_fill = role(theme, Role::Selection)?;
    let focus = role(theme, Role::FocusRing)?;
    let accent = role(theme, Role::Accent)?;
    let border = role(theme, Role::Border)?;
    let raised = role(theme, Role::SurfaceRaised)?;
    let on_surface = role(theme, Role::OnSurface)?;

    v.selection.bg_fill = selection_fill;
    v.selection.stroke = stroke(focus, 1.0);

    v.widgets.noninteractive =
        widget_visuals(raised, stroke(border, 1.0), stroke(on_surface, 1.0));
    v.widgets.inactive = widget_visuals(raised, stroke(border, 1.0), stroke(on_surface, 1.0));
    v.widgets.hovered = widget_visuals(raised, stroke(focus, 1.0), stroke(on_surface, 1.0));
    v.widgets.active = widget_visuals(
        accent,
        stroke(focus, 1.0),
        stroke(role(theme, Role::OnAccent)?, 1.0),
    );
    v.widgets.open = widget_visuals(raised, stroke(border, 1.0), stroke(on_surface, 1.0));
    Ok(v)
}

/// The estate's dark seed as egui `Visuals` — the convenience the apps use
/// (`chisel-doc` mounts this at startup).
///
/// # Errors
/// As [`to_visuals`].
pub fn seed_visuals() -> Result<Visuals, ThemeError> {
    to_visuals(&seed_theme()?)
}

/// The estate's dark seed graph (the shroom-derived midnight palette),
/// gated and ready to derive variants from.
///
/// # Errors
/// Only if a literal in [`seed_visuals`] is unparseable — a compile-time
/// constant mistake, not runtime input.
pub fn seed_theme() -> Result<TokenGraph, ThemeError> {
    let mut theme = TokenGraph::new("midnight");
    for (role, hex) in SEED {
        theme.set_hex(*role, hex)?;
    }
    Ok(theme)
}

/// The seed palette: every role the estate's dark theme defines.
pub const SEED: &[(Role, &str)] = &[
    (Role::Surface, "#0d1117"),
    (Role::SurfaceRaised, "#161b22"),
    (Role::OnSurface, "#e6edf3"),
    (Role::OnSurfaceMuted, "#9da7b3"),
    (Role::Accent, "#1f6feb"),
    (Role::OnAccent, "#ffffff"),
    (Role::Warn, "#d29922"),
    (Role::Error, "#f85149"),
    (Role::Success, "#3fb950"),
    (Role::Selection, "#1f3a5f"),
    (Role::Border, "#8b949e"),
    (Role::FocusRing, "#58a6ff"),
];
