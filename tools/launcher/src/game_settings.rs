//! The title's own `settings.xml`: where it is, reading and writing it, and how
//! each of its settings is shown.

use std::path::PathBuf;

use crate::settings::data_dir;

#[derive(Clone)]
pub(crate) struct GameSetting {
    pub(crate) section: String,
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) line: usize,
    pub(crate) indent: String,
}

pub(crate) struct GameSettings {
    pub(crate) path: PathBuf,
    pub(crate) lines: Vec<String>,
    pub(crate) settings: Vec<GameSetting>,
    pub(crate) dirty: bool,
    pub(crate) note: String,
}

fn find_game_settings() -> Option<PathBuf> {
    let roots: Vec<PathBuf> = [
        data_dir().map(|d| d.join("prefix/pfx/drive_c/users")),
        data_dir().map(|d| d.join("data")),
    ]
    .into_iter()
    .flatten()
    .collect();
    newest_settings(&roots)
}

/// The newest `settings.xml` under `<root>/*/storage/settings`, if there is one.
/// The title writes an `appdata0.rpf`-style container per profile, and the one
/// it used last is the one whose settings it will read next.
fn newest_settings(roots: &[PathBuf]) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for root in roots {
        let mut stack = vec![(root.clone(), 0usize)];
        let mut seen = 0usize;
        while let Some((dir, depth)) = stack.pop() {
            if depth > 12 || seen > 40_000 {
                continue;
            }
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                seen += 1;
                let path = entry.path();
                let Ok(kind) = entry.file_type() else {
                    continue;
                };
                if kind.is_dir() {
                    stack.push((path, depth + 1));
                } else if path.file_name().and_then(|n| n.to_str()) == Some("settings.xml")
                    && dir.ends_with("storage/settings")
                {
                    if let Ok(meta) = entry.metadata() {
                        let when = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                        if best.as_ref().is_none_or(|(t, _)| when > *t) {
                            best = Some((when, path));
                        }
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

pub(crate) fn load_game_settings() -> Option<GameSettings> {
    let path = find_game_settings()?;
    let text = std::fs::read_to_string(&path).ok()?;
    Some(parse(path, &text))
}

/// Reads the values out of the text of a `settings.xml`. Every element that
/// carries a `value` attribute is a setting; its section is the chain of
/// elements above it, without the root.
fn parse(path: PathBuf, text: &str) -> GameSettings {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut settings = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with('<') || trimmed.starts_with("<?") || trimmed.starts_with("<!") {
            continue;
        }
        let self_closing = trimmed.ends_with("/>");
        let name = trimmed
            .trim_start_matches('<')
            .trim_start_matches('/')
            .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        if trimmed.starts_with("</") {
            if stack.last().map(String::as_str) == Some(name.as_str()) {
                stack.pop();
            }
            continue;
        }
        if let Some(value) = attribute(trimmed, "value") {
            let indent = line[..line.len() - line.trim_start().len()].to_string();
            let section = stack.iter().skip(1).cloned().collect::<Vec<_>>().join("/");
            settings.push(GameSetting {
                section,
                key: name.clone(),
                value,
                line: i,
                indent,
            });
            if !self_closing {
                stack.push(name);
            }
        } else if !self_closing && !trimmed.contains(&format!("</{name}>")) {
            stack.push(name);
        }
    }
    let note = format!("{} settings from settings.xml", settings.len());
    GameSettings {
        path,
        lines,
        settings,
        dirty: false,
        note,
    }
}

fn attribute(line: &str, name: &str) -> Option<String> {
    let at = line.find(&format!("{name}=\""))? + name.len() + 2;
    let rest = &line[at..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn pretty(key: &str) -> String {
    let mut out = String::new();
    for (i, c) in key.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push(' ');
        }
        out.push(c);
    }
    let mut chars = out.chars();
    let mut out = match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    for (from, to) in [
        ("Hdr", "HDR"),
        ("Ui", "UI"),
        ("U I", "UI"),
        ("Fps", "FPS"),
        ("Fsr", "FSR"),
        ("Msaa", "MSAA"),
        ("Ssro", "SSRO"),
        ("Vrs", "VRS"),
        ("S lod", "SLOD"),
    ] {
        out = out.replace(from, to);
    }
    out
}

pub(crate) fn section_title(section: &str) -> &'static str {
    match section {
        "graphics" => "Graphics",
        "advancedGraphics" => "Advanced graphics",
        "video" => "Video",
        "audio" => "Audio",
        "population" => "Population",
        "physics" => "Physics",
        "ai" => "AI",
        _ if section.ends_with("resolutionFullscreen") => "Fullscreen resolution",
        _ if section.ends_with("uiResolution") => "UI resolution",
        _ => "Other",
    }
}

/// The settings a player looks for first, in the order they normally appear in
/// a game's options. Everything else goes under Advanced graphics settings.
pub(crate) const DISPLAY_KEYS: &[(&str, &str)] = &[("advancedGraphics", "motionBlur")];

pub(crate) const GRAPHICS_KEYS: &[(&str, &str)] = &[
    ("graphics", "graphicsQualityPreset"),
    ("graphics", "hdr"),
    ("graphics", "gamma"),
    ("graphics", "anisotropicFiltering"),
    ("graphics", "fsrQuality"),
    ("advancedGraphics", "fsrSharpen"),
    ("advancedGraphics", "sharpenEnabled"),
    ("advancedGraphics", "sharpenIntensity"),
    ("advancedGraphics", "lodScale"),
    ("advancedGraphics", "grassLod"),
    ("advancedGraphics", "texLodScale_Global"),
    ("advancedGraphics", "treeTessellationEnabled"),
];

pub(crate) const SHADOW_KEYS: &[(&str, &str)] = &[
    ("advancedGraphics", "directionalScreenSpaceShadowQuality"),
    ("advancedGraphics", "worldHeightShadowQuality"),
    ("advancedGraphics", "waterReflectionResX"),
    ("advancedGraphics", "waterReflectionResY"),
    ("advancedGraphics", "waterReflectionSSR"),
];

pub(crate) fn is_curated(section: &str, key: &str) -> bool {
    DISPLAY_KEYS
        .iter()
        .chain(GRAPHICS_KEYS)
        .chain(SHADOW_KEYS)
        .any(|(s, k)| *s == section && *k == key)
        || (section == "video/resolutionFullscreen" && (key == "width" || key == "height"))
        || (section == "video" && (key == "vSync" || key == "frameRateLimit"))
}

pub(crate) fn apply_game(game: &mut GameSettings) -> std::io::Result<()> {
    let backup = game.path.with_file_name("settings.xml.bak");
    std::fs::copy(&game.path, &backup)?;
    let mut text = game.lines.join("\n");
    text.push('\n');
    std::fs::write(&game.path, text)
}

/// One `--game-set NAME=VALUE`: find the setting it names and rewrite its line.
/// `NAME` may be `section/key`, a bare key, or end in `/key`, as on the command
/// line. Returns false when nothing matches, so the caller reports it and
/// leaves the file alone.
pub(crate) fn set(game: &mut GameSettings, name: &str, value: &str) -> bool {
    let suffix = format!("/{name}");
    let Some(setting) = game.settings.iter_mut().find(|s| {
        let full = format!("{}/{}", s.section, s.key);
        full == name || s.key == name || full.ends_with(&suffix)
    }) else {
        return false;
    };
    setting.value = value.to_string();
    let line = format!(
        "{}<{} value=\"{}\" />",
        setting.indent, setting.key, setting.value
    );
    game.lines[setting.line] = line;
    true
}

pub(crate) fn label_of(setting: &GameSetting) -> String {
    match (setting.section.as_str(), setting.key.as_str()) {
        ("video/resolutionFullscreen", "width") => "Fullscreen width".into(),
        ("video/resolutionFullscreen", "height") => "Fullscreen height".into(),
        ("video/uiResolution", "width") => "UI width".into(),
        ("video/uiResolution", "height") => "UI height".into(),
        (_, key) => pretty(key),
    }
}

/// Settings the title stores with decimals. Deciding by parsing would turn a
/// slider at 0 or 1 into an integer text box, because `1` parses as both.
pub(crate) fn float_key(key: &str) -> bool {
    matches!(
        key,
        "graphicsQualityPreset"
            | "directionalScreenSpaceShadowQuality"
            | "fsrSharpen"
            | "grassLod"
            | "lodScale"
            | "texLodScale_Global"
            | "texLodScale_Mat_High"
            | "texLodScale_Mat_Med"
            | "texLodScale_Mat_Low"
            | "texLodScale_Buildings"
            | "texLodScale_Objects"
            | "texLodScale_HD"
            | "texLodScale_OrphanHD"
            | "texLodScale_LOD"
            | "texLodScale_SLOD1"
            | "texLodScale_SLOD2"
            | "texLodScale_SLOD3"
            | "texLodScale_SLOD4"
            | "smallCullThreshold"
            | "motionBlurLimit"
            | "motionBlurScale"
            | "sharpenIntensity"
            | "vehicleLodBias"
            | "worldHeightShadowQuality"
            | "manualMipBias"
    )
}

pub(crate) fn float_range(key: &str) -> (f32, f32, f32) {
    match key {
        "fsrSharpen" | "motionBlurScale" => (0.0, 1.0, 0.01),
        "sharpenIntensity" => (0.0, 2.0, 0.05),
        "motionBlurLimit" => (0.0, 32.0, 1.0),
        "smallCullThreshold" => (0.0, 4.0, 0.05),
        "graphicsQualityPreset" => (0.0, 4.0, 1.0),
        "texLodScale_LOD" => (0.0, 4.0, 0.1),
        "texLodScale_SLOD1" => (0.0, 8.0, 0.1),
        "texLodScale_SLOD2" => (0.0, 16.0, 0.5),
        "texLodScale_SLOD3" => (0.0, 32.0, 0.5),
        "texLodScale_SLOD4" => (0.0, 64.0, 0.5),
        "lodScale"
        | "grassLod"
        | "vehicleLodBias"
        | "texLodScale_Global"
        | "texLodScale_Mat_Low"
        | "texLodScale_Buildings"
        | "texLodScale_Objects"
        | "texLodScale_HD"
        | "texLodScale_OrphanHD"
        | "directionalScreenSpaceShadowQuality"
        | "worldHeightShadowQuality" => (0.0, 2.0, 0.05),
        _ => limits(key)
            .map(|(min, max, step)| (min as f32, max as f32, step as f32))
            .unwrap_or((0.0, 1.0, 0.01)),
    }
}

/// Settings with a small, named set of values: a drop-down, never a slider.
pub(crate) fn choices(key: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match key {
        "vSync" => Some(&[("Off", "0"), ("On", "1"), ("Adaptive", "2")]),
        "frameRateLimit" => Some(&[
            ("Unlimited", "0"),
            ("30 fps", "30"),
            ("60 fps", "60"),
            ("120 fps", "120"),
            ("144 fps", "144"),
            ("240 fps", "240"),
        ]),
        "desiredFPS" => Some(&[("30 fps", "30"), ("60 fps", "60"), ("120 fps", "120")]),
        "anisotropicFiltering" => Some(&[("Off", "0"), ("4×", "4"), ("8×", "8"), ("16×", "16")]),
        "fsrQuality" => Some(&[
            ("Off", "0"),
            ("Quality", "1"),
            ("Balanced", "2"),
            ("Performance", "3"),
        ]),
        "graphicsQualityPreset" => Some(&[
            ("Custom (0)", "0"),
            ("Low (1)", "1"),
            ("Medium (2)", "2"),
            ("High (3)", "3"),
            ("Ultra (4)", "4"),
        ]),
        "waterReflectionResX" => Some(&[
            ("128 px", "128"),
            ("256 px", "256"),
            ("512 px", "512"),
            ("1024 px", "1024"),
        ]),
        "waterReflectionResY" => Some(&[
            ("64 px", "64"),
            ("128 px", "128"),
            ("256 px", "256"),
            ("512 px", "512"),
        ]),
        _ => None,
    }
}

/// Whole-number ranges that read better as a slider than a text box.
pub(crate) fn int_slider(key: &str) -> Option<(i64, i64, &'static str)> {
    match key {
        "gamma" => Some((1, 50, "")),
        "hdrIntensity" | "hdrBlackLevels" => Some((0, 100, "")),
        "hdrPeakBrightness"
        | "hdrPeakBrightnessUIDefaultNits"
        | "hdrPeakBrightnessGameDefaultNits" => Some((100, 2000, " nits")),
        "decorationResolutionShift" | "lensFlaresScreenResDivExponent" => Some((0, 4, "")),
        "maxTexUpgradesPerFrame" => Some((0, 10, "")),
        "ssroSampleCount" | "vehicleHDCount" => Some((0, 8, "")),
        "adapterIndex" | "outputIndex" => Some((0, 4, "")),
        _ => None,
    }
}

pub(crate) fn limits(key: &str) -> Option<(f64, f64, f64)> {
    match key {
        "gamma" => Some((0.0, 50.0, 1.0)),
        "anisotropicFiltering" => Some((0.0, 16.0, 1.0)),
        "desiredFPS" | "frameRateLimit" => Some((0.0, 240.0, 1.0)),
        "vSync" => Some((0.0, 2.0, 1.0)),
        "fsrQuality" => Some((0.0, 4.0, 1.0)),
        "fsrSharpen" | "manualMipBias" => Some((-5.0, 5.0, 0.05)),
        "lodScale"
        | "grassLod"
        | "texLodScale_Global"
        | "texLodScale_LOD"
        | "texLodScale_SLOD1"
        | "texLodScale_SLOD2"
        | "texLodScale_SLOD3"
        | "texLodScale_SLOD4"
        | "texLodScale_Mat_High"
        | "texLodScale_Mat_Med"
        | "texLodScale_Mat_Low"
        | "texLodScale_Buildings"
        | "texLodScale_Objects"
        | "texLodScale_HD"
        | "texLodScale_OrphanHD"
        | "vehicleLodBias" => Some((0.0, 64.0, 0.05)),
        "width" | "height" => Some((0.0, 7680.0, 1.0)),
        _ => None,
    }
}

/// The title ignores these in the current build. Measured on 2026-11-27
/// (docs/FINDINGS.md 3.29c): neither frame rate, draws nor copy-back moved;
/// only gamma is confirmed to change the picture.
pub(crate) fn no_effect(key: &str) -> bool {
    matches!(
        key,
        "motionBlur"
            | "rtDynamicShadows"
            | "rtLODLightShadows"
            | "lodScale"
            | "grassLod"
            | "fsrQuality"
            | "asyncComputeEnabled"
            | "waterReflectionSSR"
            | "shadowParticleShadows"
            | "shadowAlphaTintedShadows"
            | "graphicsQualityPreset"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, SystemTime};

    /// A settings.xml shaped like the title's: a root element, sections, a
    /// self-closing setting, a paired one, and a comment that carries a
    /// `value` attribute and must not become a setting.
    const SAMPLE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Settings>
  <graphics>
    <hdr value="true" />
    <gamma value="25" />
    <graphicsQualityPreset value="3" />
  </graphics>
  <video>
    <resolutionFullscreen>
      <width value="1920" />
      <height value="1080" />
    </resolutionFullscreen>
    <vSync value="1" />
  </video>
  <!-- <gamma value="99" /> is only a comment -->
  <audio>
    <musicVolume value="0.8"></musicVolume>
  </audio>
</Settings>
"#;

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "nullifier-settings-{}-{}-{tag}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, relative: &str, text: &str) -> PathBuf {
            let path = self.0.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, text).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn value_of<'a>(game: &'a GameSettings, section: &str, key: &str) -> &'a str {
        game.settings
            .iter()
            .find(|s| s.section == section && s.key == key)
            .unwrap_or_else(|| panic!("no {section}/{key}"))
            .value
            .as_str()
    }

    #[test]
    fn parse_reads_sections_values_lines_and_indent() {
        let game = parse(PathBuf::from("settings.xml"), SAMPLE);

        assert_eq!(game.settings.len(), 7, "the comment is not a setting");
        assert_eq!(value_of(&game, "graphics", "hdr"), "true");
        assert_eq!(value_of(&game, "graphics", "gamma"), "25");
        assert_eq!(
            value_of(&game, "video/resolutionFullscreen", "width"),
            "1920"
        );
        assert_eq!(
            value_of(&game, "video/resolutionFullscreen", "height"),
            "1080"
        );
        assert_eq!(value_of(&game, "video", "vSync"), "1");
        assert_eq!(value_of(&game, "audio", "musicVolume"), "0.8");

        let gamma = game
            .settings
            .iter()
            .find(|s| s.key == "gamma")
            .expect("gamma");
        assert_eq!(gamma.line, 4, "the line the value is written back to");
        assert_eq!(gamma.indent, "    ");
        assert!(game.lines[gamma.line].trim_start().starts_with("<gamma "));

        assert!(game.note.contains('7'), "{}", game.note);
        assert!(!game.dirty);
    }

    #[test]
    fn set_takes_a_section_key_a_bare_key_or_a_suffix() {
        let mut game = parse(PathBuf::from("settings.xml"), SAMPLE);

        assert!(set(&mut game, "graphics/gamma", "30"));
        assert_eq!(value_of(&game, "graphics", "gamma"), "30");
        assert!(game.lines[4].contains("value=\"30\""));

        assert!(set(&mut game, "hdr", "false"));
        assert_eq!(value_of(&game, "graphics", "hdr"), "false");

        assert!(set(&mut game, "resolutionFullscreen/width", "2560"));
        assert_eq!(
            value_of(&game, "video/resolutionFullscreen", "width"),
            "2560"
        );

        // The line keeps its indentation and the shape the writer expects.
        assert_eq!(
            game.lines[9], "      <width value=\"2560\" />",
            "indent and self-closing form are preserved"
        );
    }

    #[test]
    fn set_unknown_key_reports_and_changes_nothing() {
        let mut game = parse(PathBuf::from("settings.xml"), SAMPLE);
        let before = game.lines.clone();

        assert!(!set(&mut game, "noSuchSetting", "1"));
        assert!(!set(&mut game, "graphics/noSuchSetting", "1"));
        assert_eq!(game.lines, before);
        assert_eq!(value_of(&game, "graphics", "gamma"), "25");
    }

    #[test]
    fn apply_writes_the_changed_line_and_keeps_the_original_as_bak() {
        let tmp = TempDir::new("apply");
        let path = tmp.write("storage/settings/settings.xml", SAMPLE);
        let mut game = parse(path.clone(), &std::fs::read_to_string(&path).unwrap());

        assert!(set(&mut game, "graphics/gamma", "30"));
        apply_game(&mut game).expect("apply");

        let after = std::fs::read_to_string(&path).unwrap();
        assert!(after.contains("    <gamma value=\"30\" />"));
        assert!(after.contains("    <hdr value=\"true\" />"));
        assert!(!after.contains("value=\"25\""));
        assert!(after.ends_with('\n'));
        assert_eq!(
            after.lines().count(),
            SAMPLE.lines().count(),
            "no line is added or dropped"
        );

        let backup = path.with_file_name("settings.xml.bak");
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            SAMPLE,
            "the backup is the file as it was"
        );

        // And what was written parses back to what was set.
        let again = parse(path, &after);
        assert_eq!(again.settings.len(), 7);
        assert_eq!(value_of(&again, "graphics", "gamma"), "30");
    }

    #[test]
    fn newest_settings_only_reads_a_storage_settings_folder() {
        let tmp = TempDir::new("find");
        let wanted = tmp.write("profile-a/storage/settings/settings.xml", SAMPLE);
        tmp.write("profile-a/storage/other/settings.xml", SAMPLE);
        tmp.write("settings.xml", SAMPLE);

        let found = newest_settings(&[tmp.path().to_path_buf()]);
        assert_eq!(found, Some(wanted));
    }

    #[test]
    fn newest_settings_prefers_the_file_the_title_wrote_last() {
        let tmp = TempDir::new("newest");
        let older = tmp.write("a/storage/settings/settings.xml", SAMPLE);
        let newer = tmp.write("b/storage/settings/settings.xml", SAMPLE);

        let stamp = |path: &Path, secs: u64| {
            let file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
            file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
                .unwrap();
        };
        stamp(&older, 1_000);
        stamp(&newer, 2_000);

        assert_eq!(
            newest_settings(&[tmp.path().to_path_buf()]),
            Some(newer.clone())
        );

        stamp(&older, 3_000);
        assert_eq!(newest_settings(&[tmp.path().to_path_buf()]), Some(older));
    }

    #[test]
    fn newest_settings_is_none_when_there_is_no_file() {
        let tmp = TempDir::new("absent");
        assert_eq!(newest_settings(&[tmp.path().to_path_buf()]), None);
        assert_eq!(
            newest_settings(&[tmp.path().join("missing").to_path_buf()]),
            None
        );
        assert_eq!(newest_settings(&[]), None);
    }
}
