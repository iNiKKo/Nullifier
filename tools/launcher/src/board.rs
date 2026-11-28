//! The community board: its entries, the local copy and the published file, and
//! what a shared log keeps and removes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::run_log::{after, before, last_num, median_fps, strip_ansi};
use crate::settings::{data_dir, Settings};
use crate::LAYER_VERSION;

#[derive(Clone, Serialize, Deserialize, Default)]
pub(crate) struct CommunityEntry {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) game: String,
    pub(crate) profile: String,
    pub(crate) fps: f32,
    pub(crate) frame_ms: f32,
    pub(crate) gpu_ms: f32,
    pub(crate) draws: u32,
    #[serde(default)]
    pub(crate) area: String,
    #[serde(default)]
    pub(crate) notes: String,
    #[serde(default)]
    pub(crate) settings: String,
    #[serde(default)]
    pub(crate) hardware: String,
    /// The exact VICTORY_* settings of the run, so another user can try them.
    #[serde(default)]
    pub(crate) sets: BTreeMap<String, String>,
    /// The launcher that made the entry; `version` is the layer.
    #[serde(default)]
    pub(crate) launcher: String,
    pub(crate) when: u64,
    pub(crate) log: String,
    /// From the published board rather than this PC.
    #[serde(default)]
    pub(crate) remote: bool,
}

fn community_path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("community.json"))
}

fn known_game(game: &str) -> String {
    if game.starts_with("unknown") && game.contains("b99f876b") {
        "v1.0.2.1".to_string()
    } else {
        game.to_string()
    }
}

pub(crate) fn normalize_community(entries: &mut [CommunityEntry]) {
    for entry in entries {
        // Entries made by the launcher before it showed the layer version.
        if entry.launcher.is_empty() && entry.version.starts_with("0.0.") {
            entry.launcher = entry.version.clone();
            entry.version = LAYER_VERSION.to_string();
        }
        entry.game = known_game(&entry.game);
    }
}

pub(crate) fn load_community() -> Vec<CommunityEntry> {
    let mut entries: Vec<CommunityEntry> = community_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    normalize_community(&mut entries);
    entries
}

pub(crate) fn save_community(entries: &[CommunityEntry]) {
    let Some(path) = community_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let local: Vec<&CommunityEntry> = entries.iter().filter(|entry| !entry.remote).collect();
    if let Ok(text) = serde_json::to_string_pretty(&local) {
        let _ = std::fs::write(path, text);
    }
}

/// Where the published board lives (a file in the repository, served by
/// GitHub's raw host: no token, no server, and edits are visible in git).
const BOARD_URL: &str =
    "https://raw.githubusercontent.com/iNiKKo/Nullifier/main/community/leaderboard.json";

pub(crate) fn fetch_board() -> Option<Vec<CommunityEntry>> {
    let url = std::env::var("NULLIFIER_BOARD_URL").unwrap_or_else(|_| BOARD_URL.to_string());
    let out = if cfg!(windows) {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "try {{ (Invoke-WebRequest -UseBasicParsing -Uri '{url}' -TimeoutSec 8).Content }} catch {{ }}"
                ),
            ])
            .output()
    } else {
        Command::new("curl")
            .args(["-fsSL", "--max-time", "8", &url])
            .output()
    }
    .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut board: Vec<CommunityEntry> = serde_json::from_slice(&out.stdout).ok()?;
    // A published file is untrusted: keep the page responsive whatever it holds.
    board.retain(|entry| entry.name.len() <= 80 && entry.log.len() <= 200_000);
    board.truncate(500);
    Some(board)
}

pub(crate) fn date_of(epoch: u64) -> String {
    let days = (epoch / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn mask_ipv4(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let token = &text[start..i];
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() == 4
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.parse::<u8>().is_ok())
            {
                out.push_str("<ip>");
            } else {
                out.push_str(token);
            }
        } else {
            let ch = text[i..].chars().next().unwrap_or(' ');
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn mask_emails(line: &str) -> String {
    line.split_whitespace()
        .map(|word| {
            if word.contains('@') && word.contains('.') && !word.starts_with("http") {
                "<email>"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The log as it is safe to share: paths, the machine's name and addresses
/// are replaced, and only the interesting lines are kept (header, frame
/// lines, failures, the resolution patch) so an entry is a few dozen KB.
pub(crate) fn sanitize_log(raw: &str, settings: &Settings) -> String {
    let clean = strip_ansi(raw);
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    let user = Path::new(&home)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    let prefix = data_dir()
        .map(|dir| dir.display().to_string())
        .unwrap_or_default();
    let game = settings.game_dir.trim().to_string();
    let mut kept: Vec<String> = Vec::new();
    for (index, line) in clean.lines().enumerate() {
        let keep = index < 25
            || line.contains(" fps,")
            || line.contains("device removed")
            || line.contains("unhandled exception")
            || line.contains("panicked")
            || line.contains(" ERROR ")
            || line.contains("resolution:")
            || line.contains("wineserver:")
            || line.contains("build ");
        if !keep {
            continue;
        }
        let mut line = line.trim_end().to_string();
        if !game.is_empty() {
            line = line.replace(&game, "<game>");
        }
        if !prefix.is_empty() {
            line = line.replace(&prefix, "<prefix>");
        }
        if !home.is_empty() {
            line = line.replace(&home, "<home>");
        }
        if !user.is_empty() {
            line = line.replace(&format!("/home/{user}"), "/home/<user>");
            line = line.replace(&format!(r"\Users\{user}"), r"\Users\<user>");
            line = line.replace(&format!("Users\\{user}"), "Users\\<user>");
        }
        line = mask_ipv4(&line);
        line = mask_emails(&line);
        kept.push(line);
    }
    if kept.len() > 240 {
        let mut cut = kept[..20].to_vec();
        cut.extend(kept[kept.len() - 220..].iter().cloned());
        cut.insert(
            20,
            format!("... {} lines omitted ...", kept.len() - cut.len()),
        );
        cut.join("\n")
    } else {
        kept.join("\n")
    }
}

/// Median of the world frame lines (under 10 fps; menus are 30), with the
/// last frame time, GPU time and draw count.
pub(crate) fn world_stats(raw: &str) -> (Option<f32>, Option<f32>, Option<f32>, Option<u32>) {
    let clean = strip_ansi(raw);
    let mut values: Vec<f32> = Vec::new();
    let (mut frame, mut gpu, mut draws) = (None, None, None);
    for line in clean.lines() {
        let Some(rest) = after(line, "host: ") else {
            continue;
        };
        let line_draws = before(rest, " draws/frame")
            .and_then(last_num)
            .map(|v| v as u32);
        if let Some(value) = before(rest, " fps,").and_then(|v| v.trim().parse().ok()) {
            if line_draws.is_some_and(|draws| draws >= 200) {
                values.push(value);
            }
        }
        if let Some(value) = before(rest, " ms/frame").and_then(last_num) {
            frame = Some(value);
        }
        if let Some(value) = before(rest, " on the GPU").and_then(last_num) {
            gpu = Some(value);
        }
        if let Some(value) = before(rest, " draws/frame").and_then(last_num) {
            draws = Some(value as u32);
        }
    }
    (median_fps(values), frame, gpu, draws)
}

pub(crate) fn game_label(raw: &str) -> String {
    let clean = strip_ansi(raw);
    for line in clean.lines().take(40) {
        if let Some(rest) = after(line, "build          ") {
            let label = known_game(rest.trim());
            return label.chars().take(40).collect();
        }
        if let Some(rest) = after(line, "build ") {
            if rest.contains('(') {
                let label = known_game(rest.trim());
                return label.chars().take(40).collect();
            }
        }
    }
    "unknown".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shared_log_has_no_home_paths_addresses_or_mail() {
        // The same lookup sanitize_log makes: HOME, then USERPROFILE on
        // Windows, where HOME is often unset.
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/home/tester".into());
        let settings = Settings {
            game_dir: "/games/title".into(),
            ..Settings::default()
        };
        let raw = format!(
            "6.7 fps, 150.0 ms/frame at {home}/secret/file and /games/title/x\n\
             contact bob@example.com from 192.168.1.42 and 10.0.0.7\n"
        );
        let out = sanitize_log(&raw, &settings);
        assert!(!out.contains(&home), "{out}");
        assert!(!out.contains("/games/title"), "{out}");
        assert!(!out.contains("bob@example.com"), "{out}");
        assert!(!out.contains("192.168.1.42"), "{out}");
        assert!(out.contains("<ip>"), "{out}");
        assert!(out.contains("6.7 fps"), "{out}");
    }

    #[test]
    fn shared_stats_take_world_frames_only() {
        let raw = "\
30.0 fps, 33.3 ms/frame (0.1 preparing: copies 0.1 ms 0.1 MB; 0.2 on the GPU; 0.0 copying back), 2 draws/frame\n\
INFO victory_gpu::host: 6.6 fps, 151.0 ms/frame (60.0 preparing: copies 50.0 ms 200.0 MB; 30.0 on the GPU; 0.0 copying back), 2904 draws/frame\n\
INFO victory_gpu::host: 4.4 fps, 227.0 ms/frame (90.0 preparing: copies 70.0 ms 245.0 MB; 35.0 on the GPU; 0.0 copying back), 3000 draws/frame\n";
        let (fps, frame, gpu, draws) = world_stats(raw);
        assert!((fps.unwrap() - 5.5).abs() < 0.01, "{fps:?}");
        assert_eq!(frame, Some(227.0));
        assert_eq!(gpu, Some(35.0));
        assert_eq!(draws, Some(3000));
    }

    #[test]
    fn menus_are_not_in_game_frames() {
        let raw = "INFO victory_gpu::host: 30.0 fps, 33.3 ms/frame (0.1 preparing; 0.2 on the GPU), 2 draws/frame\n";
        assert_eq!(world_stats(raw).0, None);
        let mixed = "INFO victory_gpu::host: 30.0 fps, 33.3 ms/frame, 2 draws/frame\n                     INFO victory_gpu::host: 4.4 fps, 227.0 ms/frame, 3000 draws/frame\n";
        assert_eq!(world_stats(mixed).0, Some(4.4));
    }

    #[test]
    fn dates_are_civil() {
        assert_eq!(date_of(0), "1970-01-01");
        assert_eq!(date_of(1_700_000_000), "2023-11-14");
    }
}
