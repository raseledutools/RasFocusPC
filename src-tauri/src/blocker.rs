use crate::state::{AppState, ScheduleEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
const BLOCK_MARKER_START: &str = "# === RasFocusPC START ===";
const BLOCK_MARKER_END: &str = "# === RasFocusPC END ===";
const LOOPBACK: &str = "127.0.0.1";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub domains: Vec<String>,
}

fn get_presets_map() -> HashMap<String, Preset> {
    let mut map = HashMap::new();

    map.insert(
        "adult".to_string(),
        Preset {
            id: "adult".to_string(),
            name: "Adult Content".to_string(),
            description: "Blocks pornographic and adult websites".to_string(),
            icon: "🔞".to_string(),
            domains: vec![
                "pornhub.com".to_string(),
                "www.pornhub.com".to_string(),
                "xvideos.com".to_string(),
                "www.xvideos.com".to_string(),
                "xnxx.com".to_string(),
                "www.xnxx.com".to_string(),
                "xhamster.com".to_string(),
                "www.xhamster.com".to_string(),
                "redtube.com".to_string(),
                "www.redtube.com".to_string(),
                "youporn.com".to_string(),
                "www.youporn.com".to_string(),
                "tube8.com".to_string(),
                "www.tube8.com".to_string(),
                "spankbang.com".to_string(),
                "www.spankbang.com".to_string(),
                "eporner.com".to_string(),
                "www.eporner.com".to_string(),
                "tnaflix.com".to_string(),
                "www.tnaflix.com".to_string(),
                "brazzers.com".to_string(),
                "www.brazzers.com".to_string(),
                "beeg.com".to_string(),
                "www.beeg.com".to_string(),
                "bangbros.com".to_string(),
                "www.bangbros.com".to_string(),
                "realitykings.com".to_string(),
                "www.realitykings.com".to_string(),
                "onlyfans.com".to_string(),
                "www.onlyfans.com".to_string(),
                "chaturbate.com".to_string(),
                "www.chaturbate.com".to_string(),
                "livejasmin.com".to_string(),
                "www.livejasmin.com".to_string(),
            ],
        },
    );

    map.insert(
        "social".to_string(),
        Preset {
            id: "social".to_string(),
            name: "Social Media".to_string(),
            description: "Blocks major social media platforms".to_string(),
            icon: "📱".to_string(),
            domains: vec![
                "facebook.com".to_string(),
                "www.facebook.com".to_string(),
                "m.facebook.com".to_string(),
                "instagram.com".to_string(),
                "www.instagram.com".to_string(),
                "twitter.com".to_string(),
                "www.twitter.com".to_string(),
                "x.com".to_string(),
                "www.x.com".to_string(),
                "tiktok.com".to_string(),
                "www.tiktok.com".to_string(),
                "snapchat.com".to_string(),
                "www.snapchat.com".to_string(),
                "reddit.com".to_string(),
                "www.reddit.com".to_string(),
                "pinterest.com".to_string(),
                "www.pinterest.com".to_string(),
                "tumblr.com".to_string(),
                "www.tumblr.com".to_string(),
                "linkedin.com".to_string(),
                "www.linkedin.com".to_string(),
                "twitch.tv".to_string(),
                "www.twitch.tv".to_string(),
                "discord.com".to_string(),
                "www.discord.com".to_string(),
            ],
        },
    );

    map.insert(
        "shorts".to_string(),
        Preset {
            id: "shorts".to_string(),
            name: "YouTube Shorts".to_string(),
            description: "Blocks YouTube Shorts (allows normal YouTube)".to_string(),
            icon: "📺".to_string(),
            domains: vec![
                "shorts.youtube.com".to_string(),
                "www.youtube.com/shorts".to_string(),
            ],
        },
    );

    map.insert(
        "youtube".to_string(),
        Preset {
            id: "youtube".to_string(),
            name: "YouTube (Full)".to_string(),
            description: "Blocks entire YouTube".to_string(),
            icon: "▶️".to_string(),
            domains: vec![
                "youtube.com".to_string(),
                "www.youtube.com".to_string(),
                "m.youtube.com".to_string(),
                "youtu.be".to_string(),
                "shorts.youtube.com".to_string(),
                "music.youtube.com".to_string(),
            ],
        },
    );

    map.insert(
        "youtube_ads".to_string(),
        Preset {
            id: "youtube_ads".to_string(),
            name: "YouTube (Ads-Free)".to_string(),
            description: "Blocks YouTube ad servers — YouTube still works, ads don't load".to_string(),
            icon: "🚫".to_string(),
            domains: vec![
                // Google ad delivery & tracking
                "googleadservices.com".to_string(),
                "www.googleadservices.com".to_string(),
                "googlesyndication.com".to_string(),
                "www.googlesyndication.com".to_string(),
                "doubleclick.net".to_string(),
                "www.doubleclick.net".to_string(),
                "ad.doubleclick.net".to_string(),
                "pagead2.googlesyndication.com".to_string(),
                "googleads.g.doubleclick.net".to_string(),
                "www.googletagservices.com".to_string(),
                "googletagservices.com".to_string(),
                // YouTube ad endpoints
                "ads.youtube.com".to_string(),
                "www.ads.youtube.com".to_string(),
                // Google marketing/analytics used for ad targeting
                "marketingplatform.google.com".to_string(),
                "adservice.google.com".to_string(),
                "adservice.google.com.bd".to_string(),
                // Additional ad networks
                "ade.googlesyndication.com".to_string(),
                "tpc.googlesyndication.com".to_string(),
                "video-stats.l.doubleclick.net".to_string(),
                "r.googlesyndication.com".to_string(),
            ],
        },
    );

    map.insert(
        "gambling".to_string(),
        Preset {
            id: "gambling".to_string(),
            name: "Gambling".to_string(),
            description: "Blocks betting and casino sites".to_string(),
            icon: "🎰".to_string(),
            domains: vec![
                "bet365.com".to_string(),
                "www.bet365.com".to_string(),
                "pokerstars.com".to_string(),
                "www.pokerstars.com".to_string(),
                "draftkings.com".to_string(),
                "www.draftkings.com".to_string(),
                "fanduel.com".to_string(),
                "www.fanduel.com".to_string(),
                "caesars.com".to_string(),
                "www.caesars.com".to_string(),
                "betmgm.com".to_string(),
                "www.betmgm.com".to_string(),
            ],
        },
    );

    map
}

fn read_hosts() -> std::io::Result<String> {
    fs::read_to_string(HOSTS_PATH)
}

fn write_hosts(content: &str) -> std::io::Result<()> {
    let path = PathBuf::from(HOSTS_PATH);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn extract_base_hosts(content: &str) -> String {
    let mut result = Vec::new();
    let mut in_block = false;
    for line in content.lines() {
        if line.trim() == BLOCK_MARKER_START {
            in_block = true;
            continue;
        }
        if line.trim() == BLOCK_MARKER_END {
            in_block = false;
            continue;
        }
        if !in_block {
            result.push(line);
        }
    }
    result.join("\n")
}

fn apply_to_hosts(domains: &[String]) -> Result<(), String> {
    let current = read_hosts().map_err(|e| format!("Cannot read hosts: {}", e))?;
    let base = extract_base_hosts(&current);

    let mut new_content = base.trim_end().to_string();
    new_content.push('\n');

    if !domains.is_empty() {
        new_content.push('\n');
        new_content.push_str(BLOCK_MARKER_START);
        new_content.push('\n');
        for domain in domains {
            // Skip paths (YouTube Shorts path can't be blocked via hosts)
            if !domain.contains('/') {
                new_content.push_str(&format!("{} {}\n", LOOPBACK, domain));
            }
        }
        new_content.push_str(BLOCK_MARKER_END);
        new_content.push('\n');
    }

    write_hosts(&new_content).map_err(|e| format!("Cannot write hosts (run as admin?): {}", e))?;
    Ok(())
}

fn clear_hosts() -> Result<(), String> {
    apply_to_hosts(&[])
}

// ─── Tauri Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_presets() -> Vec<Preset> {
    get_presets_map().into_values().collect()
}

#[tauri::command]
pub fn get_blocked_sites(state: State<Mutex<AppState>>) -> Vec<String> {
    state.lock().unwrap().custom_sites.clone()
}

#[tauri::command]
pub fn add_site(site: String, state: State<Mutex<AppState>>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    let site = site.trim().to_lowercase();
    let site = site.replace("https://", "").replace("http://", "").replace("www.", "");
    let site = site.split('/').next().unwrap_or("").to_string();
    if site.is_empty() {
        return Err("Invalid domain".to_string());
    }
    if !s.custom_sites.contains(&site) {
        s.custom_sites.push(site);
    }
    if s.blocking_active {
        let domains = collect_all_domains(&s);
        drop(s);
        apply_to_hosts(&domains)?;
    }
    Ok(())
}

#[tauri::command]
pub fn remove_site(site: String, state: State<Mutex<AppState>>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    s.custom_sites.retain(|x| x != &site);
    if s.blocking_active {
        let domains = collect_all_domains(&s);
        drop(s);
        apply_to_hosts(&domains)?;
    }
    Ok(())
}

#[tauri::command]
pub fn apply_preset(preset_id: String, enable: bool, state: State<Mutex<AppState>>) -> Result<(), String> {
    let presets = get_presets_map();
    if !presets.contains_key(&preset_id) {
        return Err(format!("Unknown preset: {}", preset_id));
    }
    let mut s = state.lock().unwrap();
    if enable {
        if !s.active_presets.contains(&preset_id) {
            s.active_presets.push(preset_id);
        }
    } else {
        s.active_presets.retain(|p| p != &preset_id);
    }
    if s.blocking_active {
        let domains = collect_all_domains(&s);
        drop(s);
        apply_to_hosts(&domains)?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_blocking(active: bool, state: State<Mutex<AppState>>) -> Result<(), String> {
    let mut s = state.lock().unwrap();
    s.blocking_active = active;
    if active {
        let domains = collect_all_domains(&s);
        drop(s);
        apply_to_hosts(&domains)?;
    } else {
        drop(s);
        clear_hosts()?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_blocking_status(state: State<Mutex<AppState>>) -> serde_json::Value {
    let s = state.lock().unwrap();
    serde_json::json!({
        "active": s.blocking_active,
        "activePresets": s.active_presets,
        "customSiteCount": s.custom_sites.len(),
    })
}

#[tauri::command]
pub fn get_schedule(state: State<Mutex<AppState>>) -> Option<ScheduleEntry> {
    state.lock().unwrap().schedule.clone()
}

#[tauri::command]
pub fn save_schedule(schedule: ScheduleEntry, state: State<Mutex<AppState>>) -> Result<(), String> {
    state.lock().unwrap().schedule = Some(schedule);
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn collect_all_domains(s: &AppState) -> Vec<String> {
    let presets = get_presets_map();
    let mut domains: Vec<String> = Vec::new();

    for preset_id in &s.active_presets {
        if let Some(preset) = presets.get(preset_id) {
            for d in &preset.domains {
                if !domains.contains(d) {
                    domains.push(d.clone());
                }
            }
        }
    }

    for site in &s.custom_sites {
        if !domains.contains(site) {
            domains.push(site.clone());
        }
        let www = format!("www.{}", site);
        if !domains.contains(&www) {
            domains.push(www);
        }
    }

    domains
}
