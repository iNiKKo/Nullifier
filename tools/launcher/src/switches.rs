//! The `VICTORY_*` switches in words: what each one does, example values, and the
//! warnings a set of switches gives on this machine.

use std::collections::BTreeMap;

use crate::hardware::{vram_label, Hardware};
use crate::settings::Settings;

pub(crate) fn readable_value(key: &str, value: &str) -> String {
    if matches!(
        key,
        "VICTORY_VSYNC"
            | "VICTORY_TEXTURE_NO_WAIT"
            | "VICTORY_CARD_HASH_LANES"
            | "VICTORY_WIDE_BACK"
            | "VICTORY_JOIN_COPIES"
            | "VICTORY_INDIRECT_RUNS"
            | "VICTORY_INDIRECT_WINDOW"
            | "VICTORY_RELAY_QUEUE"
            | "VICTORY_SYNC_GROUPED"
            | "VICTORY_BACK_OWN_VIEW"
            | "VICTORY_ALIAS_LAZY"
            | "VICTORY_FILL_AHEAD"
            | "VICTORY_CULL_RANGES"
    ) {
        return match value {
            "1" => "on".into(),
            "0" => "off".into(),
            other => other.into(),
        };
    }
    if key.ends_with("_MB") {
        return format!("{value} MB");
    }
    if key.ends_with("_KB") {
        return format!("{value} KB");
    }
    if key.ends_with("_MS") {
        return format!("{value} ms");
    }
    value.to_string()
}

pub(crate) fn settings_summary(settings: &Settings, hardware: &Hardware) -> String {
    let mut parts: Vec<String> = settings
        .sets
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    parts.push(format!(
        "output={}",
        if settings.output.is_empty() {
            "native"
        } else {
            &settings.output
        }
    ));
    if settings.render_scale > 0.0 {
        parts.push(format!("render_scale={:.2}", settings.render_scale));
    }
    if settings.input_mode != "auto" {
        parts.push(format!("input={}", settings.input_mode));
    }
    if settings.mouse_look {
        parts.push("mouse_look=1".into());
        if !settings.mouse_buttons {
            parts.push("mouse_buttons=0".into());
        }
    }
    parts.push(format!("profile={}", settings.profile));
    format!(
        "{} | {} {}, {}",
        parts.join(" "),
        hardware.vendor.to_uppercase(),
        vram_label(hardware.vram_mb),
        std::env::consts::OS
    )
}

pub(crate) struct Desc {
    pub(crate) name: &'static str,
    pub(crate) plain: &'static str,
    pub(crate) safe: &'static str,
}

/// What a sensible value looks like, for fields that take a bare number.
pub(crate) fn example(key: &str) -> Option<&'static str> {
    match key {
        "VICTORY_TEXTURE_BUDGET_MB" => Some("2048, 3072 or 4096 (MB)"),
        "VICTORY_CARD_BUFFERS_MB" => Some("256, 512, 1024 or 2048 (MB)"),
        "VICTORY_CARD_BUFFERS_MIN_MB" => Some("1 (MB)"),
        "VICTORY_CARD_BUFFERS_MIN_KB" => Some("64 (KiB): every buffer"),
        "VICTORY_SYNC_GROUPED" | "VICTORY_FILL_AHEAD" | "VICTORY_CULL_RANGES" => Some("1"),
        "VICTORY_HEAP_RECHECK_EVERY" => Some("300, 600 or 1200 (frames)"),
        "VICTORY_LATE_WAIT" => Some("0 to 3"),
        "VICTORY_CARD_REHASH_RUNS" => Some("0, 1 or 2"),
        "VICTORY_INDIRECT_RUNS" | "VICTORY_INDIRECT_WINDOW" | "VICTORY_RELAY_QUEUE" => Some("1"),
        "VICTORY_BACK_OWN_VIEW" | "VICTORY_ALIAS_LAZY" => Some("1"),
        "VICTORY_RELAY" => Some("0, 1 or 2"),
        "VICTORY_FPS_CAP" => Some("30, 60, 120 or 0 for unlimited"),
        "VICTORY_VSYNC" => Some("1 = on"),
        "gamma" => Some("1-50; 15 is the default"),
        "graphicsQualityPreset" => Some("1 low, 2 medium, 3 high, 4 ultra; 0 is custom"),
        "hdrIntensity" => Some("0-100; 50 is the default"),
        "hdrBlackLevels" => Some("0-100; 50 is the default"),
        "hdrPeakBrightness" => Some("600-1600 nits"),
        "anisotropicFiltering" => Some("0, 4, 8 or 16"),
        "frameRateLimit" => Some("30, 60, 120, 144, 240 or 0 for unlimited"),
        "desiredFPS" => Some("30, 60 or 120"),
        "vSync" => Some("0 = off, 1 = on, 2 = adaptive"),
        "fsrQuality" => Some("0 = off, 1 = quality, 2 = balanced, 3 = performance"),
        "fsrSharpen" => Some("0.0 to 1.0 (0.2 is the default)"),
        "sharpenIntensity" => Some("0.0 to 2.0 (1.0 is the default)"),
        "lodScale" | "grassLod" => Some("1.0 is the default"),
        "texLodScale_Global" => Some("1.0 is the default"),
        "texLodScale_LOD" => Some("2.0 is the default"),
        "texLodScale_SLOD1" => Some("4.0 is the default"),
        "texLodScale_SLOD2" => Some("8.0 is the default"),
        "texLodScale_SLOD3" => Some("16.0 is the default"),
        "texLodScale_SLOD4" => Some("32.0 is the default"),
        "texLodScale_Mat_High" => Some("0.25 is the default"),
        "texLodScale_Mat_Med" => Some("0.5 is the default"),
        "texLodScale_Mat_Low" => Some("1.0 is the default"),
        "texLodScale_Buildings"
        | "texLodScale_Objects"
        | "texLodScale_HD"
        | "texLodScale_OrphanHD" => Some("1.0 is the default"),
        "vehicleLodBias" => Some("0.0 is the default"),
        "manualMipBias" => Some("0.0 is the default; negative is sharper"),
        "waterReflectionResX" => Some("128, 256, 512 or 1024"),
        "waterReflectionResY" => Some("64, 128, 256 or 512"),
        "width" => Some("1920, 2560 or 3840"),
        "height" => Some("1080, 1440 or 2160"),
        "motionBlurLimit" => Some("8, 16 or 32"),
        "motionBlurScale" => Some("0.5 to 2.0 (1.0 is the default)"),
        "maxTexUpgradesPerFrame" => Some("5 is the default"),
        "ssroSampleCount" => Some("2 or 4"),
        "vehicleHDCount" => Some("1, 2 or 4"),
        "lensFlaresScreenResDivExponent" => Some("2 is the default"),
        _ => None,
    }
}

pub(crate) const KNOWN_SWITCHES: &[(&str, &str)] = &[
    ("VICTORY_VSYNC", "0"),
    ("VICTORY_FPS_CAP", "0"),
    ("VICTORY_TEXTURE_BUDGET_MB", "4096"),
    ("VICTORY_CARD_BUFFERS_MB", "1024"),
    ("VICTORY_CARD_BUFFERS_MIN_MB", "1"),
    ("VICTORY_CARD_BUFFERS_MIN_KB", "64"),
    ("VICTORY_SYNC_GROUPED", "1"),
    ("VICTORY_HEAP_RECHECK_EVERY", "600"),
    ("VICTORY_TEXTURE_GUARD", "0"),
    ("VICTORY_KEEP_VIEWS", "0"),
    ("VICTORY_VIEW_GUARD", "0"),
    ("VICTORY_TEXTURE_SWEEP", "0"),
    ("VICTORY_TEXTURE_NO_WAIT", "1"),
    ("VICTORY_CARD_HASH_LANES", "1"),
    ("VICTORY_WIDE_BACK", "1"),
    ("VICTORY_BACK_OWN_VIEW", "1"),
    ("VICTORY_ALIAS_LAZY", "1"),
    ("VICTORY_CARD_REHASH_RUNS", "1"),
    ("VICTORY_INDIRECT_WINDOW", "1"),
    ("VICTORY_INDIRECT_RUNS", "1"),
    ("VICTORY_JOIN_COPIES", "1"),
    ("VICTORY_RELAY", "2"),
    ("VICTORY_RELAY_QUEUE", "1"),
    ("VICTORY_FILL_AHEAD", "1"),
    ("VICTORY_CULL_RANGES", "1"),
    ("VICTORY_LATE_WAIT", "1"),
    ("VICTORY_LOG", "info"),
    ("VICTORY_WATER_LIGHT", "1"),
    ("VICTORY_FRAME_INTERVAL_MS", "16.667"),
    ("VICTORY_PIPELINE_WAIT_ONCE", "0"),
    ("VICTORY_HOST_WAVE", "32"),
];

pub(crate) fn describe(key: &str) -> Option<Desc> {
    let d = |name, plain, safe| Some(Desc { name, plain, safe });
    match key {
        "VICTORY_TEXTURE_BUDGET_MB" => d(
            "Texture memory budget",
            "Texture data kept on the card. Higher looks sharper in motion; lower leaves room for other apps.",
            "2048-4096 MB on a 16 GB card. Never more than your card's VRAM.",
        ),
        "VICTORY_TEXTURE_QUEUE_MB" => d(
            "Texture bytes a frame may spend",
            "Texture uploads in a frame are paced by a byte budget: a higher budget sends more at once, a lower one spreads them over frames.",
            "128, 192 or 256 (MB). The default is 192.",
        ),
        "VICTORY_TEXTURE_QUEUE_MS" => d(
            "Texture upload time a frame may spend",
            "The same pacing measured in milliseconds per frame, for when the uploads need bounding by time rather than by bytes.",
            "4, 8 or 16 (ms).",
        ),
        "VICTORY_CARD_BUFFERS_MB" => d(
            "World memory kept on the card",
            "The game's working memory mirrored on the card. Too much competes with textures.",
            "512 MB measured best on 16 GB; 256 MB on an 8 GB card.",
        ),
        "VICTORY_CARD_BUFFERS_MIN_KB" => d(
            "Smallest buffer kept on the card",
            "The game's buffers of this size or more (in KiB) have their copy in the card's memory. 64 puts every buffer there, which keeps small counters that shaders add to off the link between the PC and the card.",
            "64 on a 16 GB NVIDIA card on Linux (linux-nvidia-16gb), with World memory kept on the card at 2048 MB. Leave it unset elsewhere.",
        ),
        "VICTORY_SYNC_GROUPED" => d(
            "Grouped updates of the card's copies",
            "The changes sent to the card's copies before each batch of work are recorded together and then checked together, so the card waits once and not once for each copy.",
            "On with a 16 GB NVIDIA card on Linux (linux-nvidia-16gb). Not measured elsewhere.",
        ),
        "VICTORY_PIPELINE_WAIT_ONCE" => d(
            "Wait once for a pipeline",
            "A host pipeline is waited for at most one budget over the whole run, not one budget every frame. After that its draws are dropped until it is made. Prevents a slow driver compile from stalling every frame it is in view.",
            "On. Turn it off only if a pipeline must be waited for again and again.",
        ),
        "VICTORY_HOST_WAVE" => d(
            "Host wave width",
            "Run shaders at 32 or 64 lanes even if the card allows both. 32 on a 64-lane card puts every console wave64 program in two host halves: exact for the face network, slower.",
            "32 or 64. Leave it unset for the card's maximum.",
        ),
        "VICTORY_WATER_LIGHT" => d(
            "Water's tiled light",
            "The title's two tiled water-light programs. On a 32-lane host they leave whole tiles of open water dark and cost about 15 ms a frame; off (the default) drops them.",
            "Off. Turn it on to compare; the water is greener but has dark blocks.",
        ),
        "VICTORY_FRAME_INTERVAL_MS" => d(
            "Title frame rate",
            "Patches the title's own 30 fps setting and makes Nullifier's frame clock follow it. Off leaves the title at 30.",
            "16.667 (60 fps), 8.333 (120), 6.944 (144), 0 (no wait). The world must reach the rate to see a difference.",
        ),
        "VICTORY_CARD_BUFFERS_MIN_MB" => d(
            "Smallest buffer kept on the card",
            "Buffers at least this large are mirrored in card memory.",
            "1 MB. Leave it.",
        ),
        "VICTORY_LATE_WAIT" => d(
            "Frames prepared ahead",
            "Frames handed to the card ahead of the last. 2 and above can crash on Linux with an AMD card.",
            "0 or 1 with an AMD card on Linux. 3 is measured with a 16 GB NVIDIA card on Linux (linux-nvidia-16gb), and is what the Windows profiles set.",
        ),
        "VICTORY_TEXTURE_NO_WAIT" => d(
            "Do not wait for textures",
            "Show a frame without waiting for a texture upload. Faster; a texture can be black until it arrives. Off waits, which is slower but has no black pop-in.",
            "On. Turn it off only if textures look wrong.",
        ),
        "VICTORY_HEAP_RECHECK_EVERY" => d(
            "Look at streamed textures again",
            "An uploaded texture is checked for a few frames after loading, then not again. This keeps checking every N frames, so a texture the title rewrites later does not stay black or wrong. Costs a hash of those textures.",
            "600 is once every ~10 s at 60 fps. Unset is off. Higher is cheaper, lower catches a change sooner.",
        ),
        "VICTORY_TEXTURE_GUARD" => d(
            "Check every texture every frame",
            "Textures in the title's own memory are guarded and hashed only every eighth frame, so a write the guard misses can show a picture up to eight frames old. 0 hashes each at every check: slower, and the closer comparison when textures flicker.",
            "Leave it unset. 0 is the comparison for flickering or black textures.",
        ),
        "VICTORY_KEEP_VIEWS" => d(
            "Keep a texture's old view while it sharpens",
            "A slot keeps showing the old view of a streamed texture until the new one is made, instead of going black for a frame. 0 makes it black at once.",
            "On. 0 is the comparison.",
        ),
        "VICTORY_VIEW_GUARD" => d(
            "Guard every view of a shared page",
            "A write through another address of the same physical page is seen. 0 leaves those writes unseen.",
            "On.",
        ),
        "VICTORY_TEXTURE_SWEEP" => d(
            "Look for black texture slots",
            "Goes over the title's texture views and queues those whose host slot is empty, so a texture that nothing wrote again is not black for the rest of the run. 0 turns it off.",
            "On. 0 is the comparison.",
        ),
        "VICTORY_CARD_HASH_LANES" => d(
            "Faster card checks",
            "Check card copies with a team of threads. Without it the check is the largest thing on the card.",
            "On. 4.45 -> 6.6 fps here, no crashes.",
        ),
        "VICTORY_WIDE_BACK" => d(
            "Copy results back in whole ranges",
            "Copy shader results back in whole ranges instead of small pieces.",
            "On.",
        ),
        "VICTORY_BACK_OWN_VIEW" => d(
            "Copy results back through a view of our own",
            "Shader results go back to the title's memory through a view of the layer's own, so they are not sent up to the card again.",
            "On with a 16 GB NVIDIA card on Linux (linux-nvidia-16gb). Not measured elsewhere.",
        ),
        "VICTORY_ALIAS_LAZY" => d(
            "Guard only the views that are written",
            "A write through one view of a guarded page leaves the page's other views read-only until they are written too.",
            "On with a 16 GB NVIDIA card on Linux (linux-nvidia-16gb). Not measured elsewhere.",
        ),
        "VICTORY_CARD_REHASH_RUNS" => d(
            "Re-check only changed ranges",
            "Only re-check card copies that changed.",
            "On is safe; the effect is small.",
        ),
        "VICTORY_INDIRECT_WINDOW" => d(
            "Batch indirect draws",
            "Handle many draw records in windows. Measured about 25% faster here.",
            "On.",
        ),
        "VICTORY_INDIRECT_RUNS" => d(
            "Batch single draw records",
            "One pass per run of single draw records. Measured about 18% faster here.",
            "On.",
        ),
        "VICTORY_JOIN_COPIES" => d(
            "Join neighbouring copies",
            "Send adjacent copies as one. Has crashed the driver before.",
            "Off unless you are testing.",
        ),
        "VICTORY_RELAY" => d(
            "Hand results back beside writes",
            "Hand results back beside writes. 2 is faster but has crashed before.",
            "0, or 2 if your machine stays stable with it.",
        ),
        "VICTORY_RELAY_QUEUE" => d(
            "A queue of its own for handed-back results",
            "With Relay on, results are fetched on a copy queue of their own, so they do not wait behind the next frame's work.",
            "On with Relay 2 on a 16 GB NVIDIA card on Linux (linux-nvidia-16gb). It does nothing with Relay 0.",
        ),
        "VICTORY_FILL_AHEAD" => d(
            "Constants read without waiting",
            "With frames prepared ahead, the constants of a draw are read without waiting for the card to finish earlier work. A check compares afterwards; if it ever finds a difference it says so in the log and waits again for the rest of the run.",
            "On with a 16 GB NVIDIA card on Linux (linux-nvidia-16gb). It does nothing with Frames prepared ahead at 0.",
        ),
        "VICTORY_CULL_RANGES" => d(
            "Culling writes narrowed",
            "The game's culling programs may write anywhere in its largest buffer, so other work on that buffer waited for them. With this on, the layer reads from the game's own records which parts they write, and only those parts wait.",
            "On with a 16 GB NVIDIA card on Linux (linux-nvidia-16gb). It applies to the culling programs the layer knows and does nothing for any other.",
        ),
        "VICTORY_VSYNC" => d(
            "V-Sync",
            "Wait for the display. Removes tearing, adds a little delay.",
            "Off unless you see tearing; it matters most above 30 fps.",
        ),
        "VICTORY_FPS_CAP" => d(
            "Frame rate limit",
            "Hold the game under this rate. 0 is unlimited; lower limits steady heat and noise.",
            "0 (off), 60, 120 or 144. Below 30 the picture feels heavy.",
        ),
        "VICTORY_LOG" => d(
            "Log detail",
            "How much is written to the log. More detail is slower.",
            "info for play; warn if you want quiet.",
        ),
        _ => None,
    }
}

pub(crate) fn setting_value(sets: &BTreeMap<String, String>, key: &str) -> Option<u32> {
    sets.get(key).and_then(|v| v.trim().parse().ok())
}

/// Values that cannot do what the player expects on this machine.
pub(crate) fn warnings(settings: &Settings, hw: &Hardware) -> Vec<String> {
    let sets = &settings.sets;
    let vram = hw.vram_mb;
    let mut out = Vec::new();
    if let Some(budget) = setting_value(sets, "VICTORY_TEXTURE_BUDGET_MB") {
        if budget == 0 {
            out.push(
                "Texture budget is 0: no texture can be copied, the world will stay untextured."
                    .into(),
            );
        } else if vram > 0 && budget > vram {
            out.push(format!(
                "Texture budget is {budget} MB, more than your card's {vram} MB. It spills into system memory and can stall or fail."
            ));
        } else if budget < 1024 {
            out.push("Texture budget below 1024 MB: textures may arrive late or stay black. 2048 MB or more is safer.".into());
        }
    }
    if let Some(buffers) = setting_value(sets, "VICTORY_CARD_BUFFERS_MB") {
        if vram > 0 && buffers > vram / 2 {
            out.push(format!(
                "World memory on the card is {buffers} MB, over half of your {vram} MB. Textures will have little room left."
            ));
        }
    }
    if let Some(late) = setting_value(sets, "VICTORY_LATE_WAIT") {
        // On Linux with an AMD card (RADV) 2 and 3 remove the device within
        // minutes. With an NVIDIA card 3 is the measured value of the
        // linux-nvidia-16gb profile (docs/FORK.md), and the Windows
        // profiles set 3.
        if late >= 2 && !cfg!(windows) && hw.vendor != "nvidia" {
            out.push("Frames prepared ahead is 2 or more: this is known to remove the device on Linux with an AMD card. Use 0 or 1.".into());
        }
    }
    if sets.contains_key("VICTORY_JOIN_COPIES") {
        out.push(
            "Joined copies can crash the graphics driver; it did before on this project.".into(),
        );
    }
    // With an NVIDIA card on Linux, 2 is the measured value of the
    // linux-nvidia-16gb profile, together with VICTORY_RELAY_QUEUE=1
    // (docs/FORK.md).
    if setting_value(sets, "VICTORY_RELAY") == Some(2) && !cfg!(windows) && hw.vendor != "nvidia" {
        out.push(
            "Relay 2 has crashed the driver before on Linux. If a run dies, set it to 0.".into(),
        );
    }
    for key in sets.keys() {
        if key.starts_with("VICTORY_") && describe(key).is_none() {
            out.push(format!(
                "{key} is not a switch this build knows; it will be ignored."
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::describe;
    use std::path::Path;

    /// Every `VICTORY_` switch that a profile of `launcher/profiles.json`
    /// sets has a description. One without is reported on the Play and
    /// Rendering pages as "not a switch this build knows; it will be
    /// ignored", which is not true of a switch that a profile sets.
    #[test]
    fn every_switch_a_profile_sets_is_described() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profiles = crate::profiles::profiles(&root);
        assert!(!profiles.is_empty(), "launcher/profiles.json was not read");
        let mut missing: Vec<&str> = profiles
            .iter()
            .flat_map(|p| p.sets.keys())
            .map(String::as_str)
            .filter(|key| key.starts_with("VICTORY_") && describe(key).is_none())
            .collect();
        missing.sort_unstable();
        missing.dedup();
        assert!(missing.is_empty(), "no description for {missing:?}");
    }
}
