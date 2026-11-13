//! Icons. Glyphs come from the icon font Windows ships (Segoe Fluent Icons, or
//! Segoe MDL2 Assets on Windows 10 — the code points used here exist in both).

pub const SEARCH: &str = "\u{E721}";
pub const REFRESH: &str = "\u{E72C}";
pub const SYNC: &str = "\u{EDAB}";
pub const TRASH: &str = "\u{E74D}";
pub const CHECK: &str = "\u{E73E}";
pub const WARN: &str = "\u{E7BA}";
pub const ADMIN: &str = "\u{E7EF}";
pub const LOCK: &str = "\u{E72E}";
pub const HISTORY: &str = "\u{E81C}";
pub const PALETTE: &str = "\u{E790}";
pub const POWER: &str = "\u{E7E8}";
pub const BOLT: &str = "\u{E945}";
pub const BATTERY: &str = "\u{E83F}";
pub const APP: &str = "\u{ECAA}";
pub const CMD: &str = "\u{E756}";
pub const CODE: &str = "\u{E943}";
pub const UNDO: &str = "\u{E7A7}";
pub const CLOCK: &str = "\u{E823}";
pub const CPU: &str = "\u{EEA1}";
pub const RAM: &str = "\u{E964}";
pub const DRIVE: &str = "\u{EDA2}";
pub const GPU: &str = "\u{F211}";
pub const PC: &str = "\u{E977}";
pub const FOLDER: &str = "\u{E8B7}";
pub const TILES: &str = "\u{ECA5}";
pub const OPEN: &str = "\u{E8A7}";
pub const MINIMIZE: &str = "\u{E921}";
pub const MAXIMIZE: &str = "\u{E922}";
pub const RESTORE: &str = "\u{E923}";
pub const CLOSE: &str = "\u{E8BB}";

/// Icon of a page in the sidebar.
pub fn nav(view: &str) -> &'static str {
    match view {
        "dashboard" => "\u{E80F}",
        "monitor" => "\u{E9D9}",
        "shortcuts" => "\u{F0E2}",
        "windows" => "\u{E770}",
        "gaming" => "\u{E7FC}",
        "gpu" => GPU,
        "power" => BOLT,
        "network" => "\u{E774}",
        "privacy" => LOCK,
        "personalize" => "\u{E771}",
        "contextmenu" => "\u{E8FD}",
        "apps" => "\u{EC91}",
        "brave" => "\u{EA18}",
        "cleaner" => "\u{EA99}",
        "powerplans" => BATTERY,
        "startup" => POWER,
        "history" => HISTORY,
        "cpu" => CPU,
        "memory" => RAM,
        "storage" => DRIVE,
        "usb" => "\u{ECF0}",
        "responsiveness" => "\u{EC4A}",
        "dns" => "\u{E968}",
        "security" => "\u{E83D}",
        "services" => "\u{E9F5}",
        "tasks" => CLOCK,
        "background" => "\u{E81E}",
        "features" => "\u{EA86}",
        "debloat" => TRASH,
        "edge" => "\u{E737}",
        _ => "\u{E713}",
    }
}

/// First rule whose text appears in "category:id" wins, so specific rules come first.
const RULES: &[(&str, &str)] = &[
    // context menu
    ("contextmenu:show_more", "\u{E8FD}"),
    ("give_access", "\u{E902}"),
    ("contextmenu:share", "\u{E72D}"),
    ("cast_to", "\u{EC15}"),
    ("scan_defender", "\u{E83D}"),
    ("contextmenu:pin_", "\u{E718}"),
    ("previous_versions", HISTORY),
    ("copy_path", "\u{E8C8}"),
    ("cmd_here", CMD),
    ("powershell", CMD),
    ("notepad", "\u{E70F}"),
    ("ownership", ADMIN),
    ("contextmenu:print", "\u{E749}"),
    ("send_to", "\u{E724}"),
    ("copy_to", "\u{E8C8}"),
    ("move_to", "\u{E8DE}"),
    ("bing", SEARCH),
    // brave
    ("rewards", "\u{EE57}"),
    ("wallet", "\u{EB41}"),
    ("brave:disable_vpn", "\u{E705}"),
    ("ai_chat", "\u{F4A5}"),
    ("stats_ping", "\u{E9D9}"),
    ("metrics", "\u{E9D9}"),
    ("error_pages", "\u{E783}"),
    ("network_prediction", "\u{EB44}"),
    ("brave:disable_sync", SYNC),
    ("password", "\u{E8D7}"),
    ("autofill_address", "\u{E81D}"),
    ("autofill_card", "\u{EB41}"),
    ("homepage", "\u{E80F}"),
    ("restore_newtab", "\u{E737}"),
    ("safe_browsing", "\u{EA18}"),
    ("https", LOCK),
    ("apply_all", "\u{E9D5}"),
    // privacy
    ("location", "\u{E81D}"),
    ("camera", "\u{E722}"),
    ("microphone", "\u{E720}"),
    ("notif", "\u{EA8F}"),
    ("account", "\u{E77B}"),
    ("contacts", "\u{E716}"),
    ("calendar", "\u{E787}"),
    ("phone_calls", "\u{E717}"),
    ("call_history", HISTORY),
    ("email", "\u{E715}"),
    ("privacy_tasks", "\u{E9D5}"),
    ("messaging", "\u{E8BD}"),
    ("radios", "\u{E704}"),
    ("bluetooth", "\u{E702}"),
    ("diagnostics", "\u{E9D9}"),
    ("documents", "\u{E8A5}"),
    ("pictures", "\u{E8B9}"),
    ("videos", "\u{E714}"),
    ("filesystem", "\u{EC50}"),
    ("eyetracker", "\u{E890}"),
    ("activity", HISTORY),
    ("screenshot", "\u{E7A8}"),
    // personalize
    ("accent_", PALETTE),
    ("tint", PALETTE),
    ("small_icons", "\u{E8A9}"),
    ("combine", "\u{EB91}"),
    ("clock", CLOCK),
    ("tray", APP),
    ("start_recent", HISTORY),
    ("start_frequent", "\u{E8E1}"),
    ("thispc", "\u{EC4E}"),
    ("recyclebin", TRASH),
    ("userfiles", FOLDER),
    // network
    ("teredo", "\u{E968}"),
    ("ipv", "\u{E968}"),
    ("nagle", "\u{E8AB}"),
    ("nodelay", "\u{E8AB}"),
    ("disable_rss", "\u{EC05}"),
    ("disable_rsc", "\u{EC05}"),
    ("autotuning", "\u{E9E9}"),
    ("qos", "\u{E905}"),
    ("dns", "\u{E774}"),
    ("disable_ecn", "\u{E8AB}"),
    ("timestamps", "\u{E916}"),
    // power
    ("disk", DRIVE),
    ("display", "\u{E7F4}"),
    ("ultimate", "\u{EC4A}"),
    ("wifi", "\u{E701}"),
    ("bright", "\u{E706}"),
    ("power_button", POWER),
    ("lid_", "\u{E7F8}"),
    ("fast_startup", BOLT),
    ("wake", "\u{EA8F}"),
    ("bw_saving", "\u{E905}"),
    ("usb_dma", "\u{EA18}"),
    ("usb", "\u{ECF0}"),
    ("selective", "\u{ECF0}"),
    // gpu page
    ("storage_sense", DRIVE),
    ("bsod", "\u{EB55}"),
    ("cross_device", "\u{E8CC}"),
    ("logon_blur", "\u{E81E}"),
    ("logon_verbose", CMD),
    ("outlook", "\u{E715}"),
    ("mouse", "\u{E962}"),
    ("settings_home", "\u{E713}"),
    ("fullscreen", "\u{E9A6}"),
    ("utc", "\u{EC92}"),
    ("rdp", "\u{E703}"),
    ("vsync", "\u{E7F4}"),
    ("tdr", "\u{E916}"),
    ("timeout", "\u{E916}"),
    ("power_save", BATTERY),
    ("gpu_priority", "\u{EC4A}"),
    ("multi_gpu", "\u{EBD2}"),
    ("accel", BOLT),
    // gaming
    ("timer", "\u{E916}"),
    ("game_mode", "\u{E7FC}"),
    ("dvr", "\u{E714}"),
    ("game_bar", "\u{E8F9}"),
    ("memory_integrity", "\u{EA18}"),
    ("vbs", "\u{EA18}"),
    ("vrr", "\u{E7F4}"),
    ("optimize", "\u{EC4A}"),
    ("msi_mode", "\u{E950}"),
    ("hpet", "\u{E950}"),
    ("spectre", "\u{EA18}"),
    ("tpm", "\u{E8D7}"),
    ("prefetch", DRIVE),
    // windows
    ("telemetry", "\u{E9D9}"),
    ("actioncenter", "\u{EA8F}"),
    ("background", "\u{E9F5}"),
    ("cortana", "\u{E720}"),
    ("defender", "\u{E83D}"),
    ("sechealth", "\u{E83D}"),
    ("extensions", "\u{E7C3}"),
    ("firewall", "\u{EC1B}"),
    ("hidden", "\u{E890}"),
    ("remote", "\u{E703}"),
    ("restore", "\u{E777}"),
    ("store", "\u{E719}"),
    ("search", SEARCH),
    ("sleep", "\u{E708}"),
    ("hiber", "\u{E708}"),
    ("standby", "\u{E708}"),
    ("s0_", "\u{E708}"),
    ("xbox", "\u{E7FC}"),
    ("cooling", "\u{E9CA}"),
    ("boost", "\u{EC4A}"),
    ("performance", "\u{EC4A}"),
    ("cstates", CPU),
    ("processor", CPU),
    ("core_parking", CPU),
    ("hetero", CPU),
    ("idle", CPU),
    ("maxstate", CPU),
    ("minstate", CPU),
    ("throttle", CPU),
    ("mitigations", "\u{EA18}"),
    ("virt", "\u{F158}"),
    ("speech", "\u{E720}"),
    ("copilot", "\u{F4A5}"),
    ("recall", HISTORY),
    ("widgets", TILES),
    ("edge", "\u{E774}"),
    ("onedrive", "\u{E753}"),
    ("teams", "\u{E902}"),
    ("phone", "\u{E8EA}"),
    ("spotlight", "\u{E8B9}"),
    ("suggested", "\u{EC24}"),
    ("stickers", "\u{EB68}"),
    ("tabs", "\u{E737}"),
    ("chat", "\u{E8F2}"),
    ("dev_home", "\u{EC7A}"),
    ("co_creator", "\u{F354}"),
    ("compact", "\u{EA37}"),
    ("taskview", "\u{E7C4}"),
    ("taskbar", "\u{E90E}"),
    ("start", "\u{F0E2}"),
    ("snap", "\u{E8A0}"),
    ("anim", "\u{EB9D}"),
    ("transparency", "\u{E81E}"),
    ("shadow", "\u{E97C}"),
    ("thumb", "\u{E91B}"),
    ("aero", "\u{E8A1}"),
    ("listbox", "\u{E8CB}"),
    ("font", "\u{E8D2}"),
    ("glow", "\u{E706}"),
    ("border", "\u{E739}"),
    ("end_task", "\u{F140}"),
    ("home_gallery", "\u{E80F}"),
    ("folder", FOLDER),
    ("classic_menu", "\u{E8FD}"),
    ("dark_theme", "\u{EC46}"),
    ("numlock", "\u{E765}"),
    ("scrollbars", "\u{E8CB}"),
    ("sticky", "\u{E765}"),
    ("battery", BATTERY),
];

/// Icon for one tweak; falls back to its page's icon, so every tweak has one.
pub fn tweak(cat: &str, id: &str) -> &'static str {
    // one-per-item pages read better with one icon
    if ["services", "tasks", "features"].contains(&cat) {
        return nav(cat);
    }
    let key = format!("{cat}:{id}");
    RULES.iter().find(|r| key.contains(r.0)).map(|r| r.1).unwrap_or_else(|| nav(cat))
}

/// Tweaks that switch off or remove one specific app: (app, glyph, brand colour).
/// shortcut: glyph-on-brand-colour tiles, not the apps' real logos — those apps
/// are not installed on the build PC to take the logo from; swap in PNGs when available.
pub fn brand(cat: &str, id: &str) -> Option<(&'static str, &'static str, u32)> {
    Some(match (cat, id) {
        ("windows", "disable_cortana") => ("CORTANA", "\u{E93C}", 0x0078d4),
        ("windows", "disable_xbox") => ("XBOX", "\u{E7FC}", 0x107c10),
        ("windows", "disable_copilot") => ("COPILOT", "\u{F4A5}", 0x7b5cfa),
        ("windows", "disable_recall") => ("RECALL", HISTORY, 0x2b88d8),
        ("windows", "disable_widgets_win11") => ("WIDGETS", TILES, 0x0a84c6),
        ("windows", "disable_edge_bg_win11") => ("EDGE", "\u{E774}", 0x0c8a6e),
        ("windows", "disable_onedrive_win11") => ("ONEDRIVE", "\u{E753}", 0x0364b8),
        ("windows", "remove_teams") => ("TEAMS", "\u{E902}", 0x4b53bc),
        ("windows", "remove_phone_link") => ("PHONE LINK", "\u{E8EA}", 0x0067c0),
        ("windows", "remove_chat") => ("CHAT", "\u{E8F2}", 0x6264a7),
        ("windows", "remove_dev_home") => ("DEV HOME", "\u{EC7A}", 0x3a3a8c),
        ("windows", "disable_co_creator") => ("PAINT COCREATOR", "\u{F354}", 0xd9488b),
        ("windows", "disable_storeauto") | ("power", "disable_store_auto") => ("MICROSOFT STORE", "\u{E719}", 0x0f6cbd),
        ("gpu", "disable_outlook_new") => ("OUTLOOK", "\u{E715}", 0x0f6cbd),
        ("gaming", "disable_game_bar") => ("GAME BAR", "\u{E8F9}", 0x107c10),
        _ => return None,
    })
}

pub const BRAVE_ORANGE: u32 = 0xfb542b;

pub fn cleaner(id: &str) -> &'static str {
    match id {
        "win_temp" => "\u{EA99}",
        "prefetch" => "\u{EB9D}",
        "recent" => CLOCK,
        "recycle_bin" => TRASH,
        "event_logs" => "\u{E9F9}",
        "dns_cache" => "\u{E774}",
        "wu_cache" => "\u{E777}",
        "thumb_cache" => "\u{E91B}",
        _ => "\u{EB55}",
    }
}

pub fn shortcut_group(name: &str) -> &'static str {
    match name {
        "SYSTEM" => PC,
        "UPDATE & SECURITY" => "\u{EA18}",
        "HARDWARE" => "\u{E950}",
        "NETWORK" => "\u{E774}",
        "APPS" => APP,
        "PERSONALIZATION" => PALETTE,
        "ACCOUNTS & PRIVACY" => "\u{E77B}",
        "STORAGE" => DRIVE,
        _ => "\u{E90F}",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rules_pick_the_specific_icon() {
        assert_eq!(tweak("windows", "disable_restore"), "\u{E777}"); // not the Store bag
        assert_eq!(tweak("power", "disable_store_auto"), "\u{E719}");
        assert_eq!(tweak("privacy", "privacy_phone_calls"), "\u{E717}");
        assert_eq!(tweak("power", "enable_fast_startup"), BOLT);
        assert_eq!(tweak("gpu", "disable_fxaa"), GPU); // page icon as fallback
    }
}
