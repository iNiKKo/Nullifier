//! Nullifier's graphical launcher.
//!
//! A rail of four pages, one primary button:
//!   Play      - the game folder, the auto-detected profile and one big Launch,
//!               with the few settings a player touches next to it.
//!   Graphics  - the measured switches, in plain words.
//!   Advanced  - every switch, free-form environment, and what is not built.
//!   Debug     - detect, logging, captures, scripted keys and a benchmark.
//!
//! The launchers honour `VICTORY_PROFILES`, so this front end writes an exact
//! profile and forces it: a switch that is off is absent, never `=0`, and no
//! reader that tests for presence can be fooled.

mod app;
mod board;
mod game_settings;
mod hardware;
mod install;
mod keys;
mod launch;
mod os;
mod profiles;
mod run_log;
mod settings;
mod switches;
mod update;
mod widgets;

use eframe::egui;

use app::{App, Tab};
use game_settings::{apply_game, load_game_settings};
use install::find_root;
use launch::check;
use settings::Settings;

const TITLE: &str = "Nullifier";
const ACCENT: egui::Color32 = egui::Color32::from_rgb(77, 122, 235);
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The translation layer's build string. Kept beside `BUILD` in
/// `crates/victory-gpu/src/host.rs` (bumped by hand there).
const LAYER_VERSION: &str = "0.011";

fn apply_theme(ctx: &egui::Context) {
    use egui::{Color32, FontFamily, FontId, Stroke, TextStyle};

    // Shared visual tokens. Keep the palette deliberately restrained so the
    // launcher feels at home on KDE while remaining consistent on Windows.
    let bg = Color32::from_rgb(23, 25, 31);
    let surface = Color32::from_rgb(34, 38, 48);
    let raised = Color32::from_rgb(41, 46, 57);
    let border = Color32::from_rgb(52, 58, 71);
    let text = Color32::from_rgb(230, 233, 240);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(13.0, 8.0);
    style.spacing.interact_size = egui::vec2(40.0, 34.0);
    style.spacing.indent = 18.0;
    style.spacing.window_margin = egui::Margin::same(18);
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(23.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(11.5, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        ),
    ]
    .into();

    let mut v = egui::Visuals::dark();
    v.panel_fill = bg;
    v.window_fill = surface;
    v.faint_bg_color = raised;
    v.extreme_bg_color = Color32::from_rgb(19, 21, 26);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, text);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, text);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, text);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, text);
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = Color32::from_rgb(54, 83, 143);
    v.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(132, 164, 235));
    v.window_corner_radius = 10.into();
    v.menu_corner_radius = 8.into();
    v.window_stroke = Stroke::new(1.0_f32, border);
    v.popup_shadow = egui::Shadow::NONE;

    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = 7.into();
    }
    v.widgets.noninteractive.bg_fill = surface;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border);
    v.widgets.inactive.bg_fill = raised;
    v.widgets.inactive.weak_bg_fill = surface;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border);
    v.widgets.hovered.bg_fill = Color32::from_rgb(49, 56, 69);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(49, 56, 69);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(76, 88, 108));
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(111, 151, 246));
    v.widgets.open.bg_fill = raised;
    v.widgets.open.bg_stroke = Stroke::new(1.0_f32, border);

    style.visuals = v;
    ctx.set_style(style);
}

fn main() -> eframe::Result<()> {
    let root = find_root();
    let mut settings = Settings::load(root.as_deref());

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--game-list") {
        match load_game_settings() {
            Some(game) => {
                println!("# {}", game.path.display());
                for s in &game.settings {
                    println!("{}/{}={}", s.section, s.key, s.value);
                }
            }
            None => println!("no settings.xml found"),
        }
        return Ok(());
    }
    let mut changes: Vec<(String, String)> = Vec::new();
    for i in 0..args.len() {
        if args[i] == "--game-set" {
            if let Some((k, v)) = args.get(i + 1).and_then(|a| a.split_once('=')) {
                changes.push((k.to_string(), v.to_string()));
            }
        }
    }
    if !changes.is_empty() {
        let Some(mut game) = load_game_settings() else {
            eprintln!("no settings.xml found");
            return Ok(());
        };
        let mut ok = true;
        for (name, value) in changes {
            if game_settings::set(&mut game, &name, &value) {
                println!("{name}={value}");
            } else {
                eprintln!("no such setting: {name}");
                ok = false;
            }
        }
        if ok {
            match apply_game(&mut game) {
                Ok(()) => println!("saved (settings.xml.bak kept)"),
                Err(e) => eprintln!("could not save: {e}"),
            }
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--check") {
        check(root.as_deref(), &settings);
        return Ok(());
    }
    let start_tab = std::env::args()
        .position(|a| a == "--tab")
        .and_then(|i| std::env::args().nth(i + 1));
    let start_info = std::env::args()
        .position(|a| a == "--info")
        .and_then(|i| std::env::args().nth(i + 1));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 720.0])
            .with_min_inner_size([820.0, 560.0])
            .with_title(TITLE),
        ..Default::default()
    };
    eframe::run_native(
        TITLE,
        options,
        Box::new(move |cc| {
            apply_theme(&cc.egui_ctx);
            settings.root = root.clone();
            let mut app = App::new(settings);
            app.tab = match start_tab.as_deref() {
                Some("game") => Tab::Graphics,
                Some("graphics") => Tab::Graphics,
                Some("controls") => Tab::Controls,
                Some("community") => Tab::Community,
                Some("rendering") => Tab::Rendering,
                Some("advanced") => Tab::Advanced,
                Some("debug") | Some("tools") => Tab::Debug,
                _ => Tab::Play,
            };
            if matches!(app.tab, Tab::Rendering | Tab::Advanced | Tab::Debug) {
                app.settings.show_advanced = true;
            }
            app.info = start_info;
            if std::env::args().any(|a| a == "--bench") {
                app.bench_now();
            }
            Ok(Box::new(app))
        }),
    )
}
