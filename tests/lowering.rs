//! THM-008: the adapter's contract — every colour it emits is byte-identical
//! to the role the gate approved. This is the test that makes the design
//! system real across toolkits (ADR-0004 §2.3).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use theme_egui::{seed_theme, seed_visuals, to_visuals, SEED};
use theme_kit::{audit, Role, TokenGraph};

fn expected(theme: &TokenGraph, role: Role) -> egui::Color32 {
    let view = theme.resolve(role).unwrap().to_srgb8().unwrap();
    egui::Color32::from_rgba_unmultiplied(view.r, view.g, view.b, view.a)
}

#[test]
fn the_seed_theme_passes_the_gate() {
    audit(&seed_theme().unwrap(), None).expect("the seed palette is gated");
}

#[test]
fn every_placed_colour_is_the_approved_role() {
    let theme = seed_theme().unwrap();
    audit(&theme, None).expect("gate first, always");
    let v = to_visuals(&theme).unwrap();

    assert_eq!(v.window_fill, expected(&theme, Role::Surface));
    assert_eq!(v.panel_fill, expected(&theme, Role::SurfaceRaised));
    assert_eq!(v.extreme_bg_color, expected(&theme, Role::Surface));
    assert_eq!(v.faint_bg_color, expected(&theme, Role::SurfaceRaised));
    assert_eq!(
        v.override_text_color,
        Some(expected(&theme, Role::OnSurface))
    );
    assert_eq!(
        v.weak_text_color,
        Some(expected(&theme, Role::OnSurfaceMuted))
    );
    assert_eq!(v.hyperlink_color, expected(&theme, Role::Accent));
    assert_eq!(v.warn_fg_color, expected(&theme, Role::Warn));
    assert_eq!(v.error_fg_color, expected(&theme, Role::Error));
    assert_eq!(v.window_stroke.color, expected(&theme, Role::Border));
    assert_eq!(v.selection.bg_fill, expected(&theme, Role::Selection));
    assert_eq!(v.selection.stroke.color, expected(&theme, Role::FocusRing));
    assert_eq!(v.widgets.active.bg_fill, expected(&theme, Role::Accent));
    assert_eq!(
        v.widgets.active.fg_stroke.color,
        expected(&theme, Role::OnAccent)
    );
    assert_eq!(
        v.widgets.hovered.bg_stroke.color,
        expected(&theme, Role::FocusRing)
    );
    assert_eq!(
        v.widgets.noninteractive.bg_stroke.color,
        expected(&theme, Role::Border)
    );
}

#[test]
fn the_dark_seed_lowers_without_maths() {
    let v = seed_visuals().unwrap();
    assert_eq!(v.window_fill, egui::Color32::from_rgb(0x0d, 0x11, 0x17));
    assert!(v.dark_mode, "a dark surface picks egui's dark base");
}

#[test]
fn a_light_theme_picks_the_light_base() {
    let mut theme = seed_theme().unwrap();
    for (role, hex) in [
        (Role::Surface, "#fdfdfd"),
        (Role::SurfaceRaised, "#f0f2f5"),
        (Role::OnSurface, "#101418"),
        (Role::OnSurfaceMuted, "#4a5058"),
        (Role::Accent, "#0b5ed7"),
        (Role::OnAccent, "#ffffff"),
        (Role::Selection, "#cfe3ff"),
        (Role::Warn, "#8a5a00"),
        (Role::Error, "#a40e26"),
        (Role::Success, "#1a7f37"),
        (Role::Border, "#57606a"),
        (Role::FocusRing, "#0b5ed7"),
    ] {
        theme.set_hex(role, hex).unwrap();
    }
    audit(&theme, None).expect("the light theme is gated too");
    let v = to_visuals(&theme).unwrap();
    assert!(!v.dark_mode, "a light surface picks egui's light base");
    assert_eq!(v.window_fill, expected(&theme, Role::Surface));
    assert_eq!(v.override_text_color, Some(expected(&theme, Role::OnSurface)));
}

#[test]
fn unset_roles_are_errors_not_invented_colours() {
    let partial = TokenGraph::from_dtcg_json(
        "partial",
        r##"{"color":{"surface":{"$value":"#0d1117"},"on-surface":{"$value":"#ffffff"}}}"##,
    )
    .unwrap();
    assert!(to_visuals(&partial).is_err(), "no role, no Visuals");
}

#[test]
fn the_seed_constant_is_the_whole_vocabulary() {
    assert_eq!(SEED.len(), 12, "every Role has a seed colour");
    let mut seen: Vec<Role> = SEED.iter().map(|(r, _)| *r).collect();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), Role::ALL.len(), "no role is missing from the seed");
}
