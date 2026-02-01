use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

use rusqlite::{Connection, Result as SqliteResult};
use std::fs;

#[derive(Debug, Clone)]
pub enum Browser {
    Chrome,
    Firefox,
    Brave,
    Edge,
    Zen, // <- best browser
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub url: String,
}

impl Browser {
    pub fn detect() -> Self {
        // Try xdg-settings first
        if let Ok(output) = Command::new("xdg-settings")
            .args(["get", "default-web-browser"])
            .output()
        {
            let browser_str = String::from_utf8_lossy(&output.stdout).to_lowercase();

            if browser_str.contains("zen") {
                return Browser::Zen;
            } else if browser_str.contains("chrome") {
                return Browser::Chrome;
            } else if browser_str.contains("firefox") {
                return Browser::Firefox;
            } else if browser_str.contains("brave") {
                return Browser::Brave;
            } else if browser_str.contains("edge") {
                return Browser::Edge;
            }
        }

        Browser::Unknown
    }

    pub fn bookmark_path(&self) -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;

        match self {
            Browser::Chrome => Some(PathBuf::from(format!(
                "{}/.config/google-chrome/Default/Bookmarks",
                home
            ))),
            Browser::Brave => Some(PathBuf::from(format!(
                "{}/.config/BraveSoftware/Brave-Browser/Default/Bookmarks",
                home
            ))),
            Browser::Edge => Some(PathBuf::from(format!(
                "{}/.config/microsoft-edge/Default/Bookmarks",
                home
            ))),
            Browser::Firefox | Browser::Zen => Some(PathBuf::from(home)),
            Browser::Unknown => None,
        }
    }
}

//PARSING
pub fn parse_chromium_bookmarks(
    path: &PathBuf,
) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&content)?;

    let mut bookmarks = Vec::new();

    // Chrome bookmarks have a "roots" object with bookmark_bar, other, synced
    if let Some(roots) = json.get("roots") {
        if let Some(bookmark_bar) = roots.get("bookmark_bar") {
            extract_bookmarks(bookmark_bar, &mut bookmarks);
        }
        if let Some(other) = roots.get("other") {
            extract_bookmarks(other, &mut bookmarks);
        }
        if let Some(synced) = roots.get("synced") {
            extract_bookmarks(synced, &mut bookmarks);
        }
    }

    Ok(bookmarks)
}

fn extract_bookmarks(node: &Value, bookmarks: &mut Vec<Bookmark>) {
    if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
        match node_type {
            "url" => {
                if let (Some(name), Some(url)) = (
                    node.get("name").and_then(|n| n.as_str()),
                    node.get("url").and_then(|u| u.as_str()),
                ) {
                    bookmarks.push(Bookmark {
                        name: name.to_string(),
                        url: url.to_string(),
                    });
                }
            }
            "folder" => {
                if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
                    for child in children {
                        extract_bookmarks(child, bookmarks);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn get_bookmarks() -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
    let browser = Browser::detect();
    println!("Detected browser: {:?}", browser);

    match browser {
        Browser::Chrome | Browser::Brave | Browser::Edge => {
            let path = browser
                .bookmark_path()
                .ok_or("Could not determine bookmark path")?;

            if !path.exists() {
                return Err(format!("Bookmark file not found at {:?}", path).into());
            }

            parse_chromium_bookmarks(&path)
        }
        Browser::Firefox | Browser::Zen => {
            let profile_path =
                find_firefox_profile(&browser).ok_or("Could not find Firefox/Zen profile")?;

            println!("Using profile: {:?}", profile_path);
            parse_firefox_bookmarks(&profile_path)
        }
        Browser::Unknown => Err("Could not detect browser".into()),
    }
}
fn find_firefox_profile(browser: &Browser) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    let profile_dir = match browser {
        Browser::Zen => PathBuf::from(format!("{}/.zen", home)),
        Browser::Firefox => PathBuf::from(format!("{}/.mozilla/firefox", home)),
        _ => return None,
    };

    // Read profiles.ini to find default profile
    let profiles_ini = profile_dir.join("profiles.ini");
    if !profiles_ini.exists() {
        return None;
    }

    let content = fs::read_to_string(&profiles_ini).ok()?;

    // Look for Default=1 profile or first profile with Path=
    let mut default_path = None;
    for line in content.lines() {
        if line.starts_with("Path=") {
            default_path = Some(line.trim_start_matches("Path=").to_string());
        }
    }

    if let Some(path) = default_path {
        Some(profile_dir.join(path))
    } else {
        None
    }
}

pub fn parse_firefox_bookmarks(
    profile_path: &PathBuf,
) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
    let places_db = profile_path.join("places.sqlite");

    if !places_db.exists() {
        return Err(format!("places.sqlite not found at {:?}", places_db).into());
    }

    // Copy the database because Firefox might have it locked
    let temp_db = std::env::temp_dir().join("places_temp.sqlite");
    fs::copy(&places_db, &temp_db)?;

    let conn = Connection::open(&temp_db)?;

    let mut stmt = conn.prepare(
        "SELECT mb.title, mp.url 
         FROM moz_bookmarks mb 
         JOIN moz_places mp ON mb.fk = mp.id 
         WHERE mb.type = 1 AND mp.url IS NOT NULL",
    )?;

    let bookmarks = stmt
        .query_map([], |row| {
            Ok(Bookmark {
                name: row.get(0).unwrap_or_else(|_| String::from("Untitled")),
                url: row.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Clean up temp file
    let _ = fs::remove_file(temp_db);

    Ok(bookmarks)
}
