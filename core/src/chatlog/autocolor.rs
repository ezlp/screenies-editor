//! autocolor.rs — classify a line according to the active preset.
//!
//! Runs on the *plain* text (hex codes already stripped). Every rule is
//! gated by the preset, so different servers' formats are data, not code.
//!
//! Rule order (first match wins), each only if enabled:
//!   1. `*` prefix        → Me
//!   2. `((` prefix       → Ooc
//!   3. `TAG:` prefix     → System
//!   4. says / shouts     → Says / Shouts (always on — universal SA-MP)
//!   5. radio channels    → Says   (channels come from the preset)
//!   6. `…((Nama))` end   → Do
//!   7. otherwise         → Normal

use super::preset::ParsePreset;
use super::{systag, LineType};
use regex::Regex;
use std::sync::OnceLock;

fn says_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bsays(?:\s*\[[^\]]{1,24}\])?\s*:").expect("says regex"))
}
fn shouts_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\bshouts(?:\s*\[[^\]]{1,24}\])?\s*:").expect("shouts regex"))
}

/// Compiled per-parse context: preset + the radio regex built from it.
pub struct RuleCtx<'a> {
    pub preset: &'a ParsePreset,
    radio: Option<Regex>,
}

impl<'a> RuleCtx<'a> {
    pub fn new(preset: &'a ParsePreset) -> Self {
        let radio = if preset.radio_channels.is_empty() {
            None
        } else {
            let alts = preset
                .radio_channels
                .iter()
                .map(|c| regex::escape(c.trim()))
                .filter(|c| !c.is_empty())
                .collect::<Vec<_>>()
                .join("|");
            if alts.is_empty() {
                None
            } else {
                Regex::new(&format!(r"(?i)\[(?:{alts})[^\]]*\]\s*:")).ok()
            }
        };
        Self { preset, radio }
    }
}

/// Line type for the plain (hex-stripped) text, per the preset's rules.
pub fn classify(plain: &str, ctx: &RuleCtx) -> LineType {
    let p = ctx.preset;
    let t = plain.trim_start();

    if p.me_prefix && t.starts_with('*') {
        return LineType::Me;
    }
    if p.ooc_wrap && t.starts_with("((") {
        return LineType::Ooc;
    }
    if p.system_tags && systag::split(t).is_some() {
        return LineType::System;
    }
    if says_re().is_match(t) {
        return LineType::Says;
    }
    if shouts_re().is_match(t) {
        return LineType::Shouts;
    }
    if let Some(radio) = &ctx.radio {
        if radio.is_match(t) {
            return LineType::Says;
        }
    }
    if p.do_suffix && is_do_suffix(t) {
        return LineType::Do;
    }
    LineType::Normal
}

/// `action text ((Player Name))` — /do output on JGRP-style servers.
fn is_do_suffix(t: &str) -> bool {
    t.ends_with("))") && t.rfind("((").map_or(false, |i| i > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chatlog::preset;

    fn ty(s: &str, p: &ParsePreset) -> LineType {
        classify(s, &RuleCtx::new(p))
    }

    #[test]
    fn jgrp_classifies_real_log_shapes() {
        let p = preset::jgrp();
        assert_eq!(ty("* Daniel Ochoa showing Id Card", &p), LineType::Me);
        assert_eq!(ty("(( PM from [59] Rene: itu ga shift ))", &p), LineType::Ooc);
        assert_eq!(ty("VEHICLE: Seatbelts ON", &p), LineType::System);
        assert_eq!(ty("Coco Aguilar says [low]: Pak", &p), LineType::Says);
        assert_eq!(ty("Daniel Ochoa shouts: Tolongs!", &p), LineType::Shouts);
        assert_eq!(ty("Coco Aguilar [phone]: Dimana dokter?", &p), LineType::Says);
        assert_eq!(ty("relax ((Oswald Heisenberg))", &p), LineType::Do);
        assert_eq!(ty("Telepon terputus...", &p), LineType::Normal);
    }

    #[test]
    fn says_beats_do_suffix() {
        let p = preset::jgrp();
        assert_eq!(ty("Budi says: oke (( brb dulu ))", &p), LineType::Says);
    }

    #[test]
    fn toggles_gate_each_rule() {
        let p = preset::polos();
        assert_eq!(ty("* aksi", &p), LineType::Normal);
        assert_eq!(ty("(( ooc ))", &p), LineType::Normal);
        assert_eq!(ty("VEHICLE: x y", &p), LineType::Normal);
        assert_eq!(ty("relax ((Oswald))", &p), LineType::Normal);
        // says stays universal even in Polos
        assert_eq!(ty("Budi says: halo", &p), LineType::Says);
    }

    #[test]
    fn custom_radio_channels_work() {
        let mut p = preset::jgrp();
        p.radio_channels = vec!["dep".into(), "gov".into()];
        assert_eq!(ty("Coco [dep]: unit 12 siap", &p), LineType::Says);
        assert_eq!(ty("Coco [phone]: halo", &p), LineType::Normal); // phone removed
    }
}
