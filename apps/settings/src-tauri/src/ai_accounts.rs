//! The AI accounts whose plan usage the menu bar shows, for Settings.
//!
//! Bahia never asks for a password or key. Each tool signs in with its own
//! official login, which Bahia only opens in a terminal; the dock then reads
//! the session that tool saved, as it always has. Here Settings only checks
//! whether such a session exists: it never reads a token's value out.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use bahia_shell::launcher;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy)]
struct Provider {
    id: &'static str,
    name: &'static str,
    /// The command-line tool that signs in.
    tool: &'static str,
    /// The arguments of its own login, run in a terminal window.
    login: &'static [&'static str],
    /// Where to get the tool when it is not installed.
    install: &'static str,
}

const PROVIDERS: [Provider; 5] = [
    Provider {
        id: "claude",
        name: "Claude Code",
        tool: "claude",
        // Claude Code asks to sign in when it starts without a session, and
        // with one its `/login` command switches accounts.
        login: &[],
        install: "https://docs.anthropic.com/en/docs/claude-code/setup",
    },
    Provider {
        id: "codex",
        name: "Codex",
        tool: "codex",
        login: &["login"],
        install: "https://github.com/openai/codex",
    },
    Provider {
        id: "opencode",
        name: "OpenCode Go",
        tool: "opencode",
        login: &["auth", "login"],
        install: "https://opencode.ai/docs",
    },
    Provider {
        id: "zai",
        name: "Z.ai GLM Coding Plan",
        tool: "opencode",
        login: &["auth", "login"],
        install: "https://opencode.ai/docs",
    },
    Provider {
        id: "gemini",
        name: "Gemini CLI",
        tool: "gemini",
        login: &[],
        install: "https://github.com/google-gemini/gemini-cli",
    },
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    id: &'static str,
    name: &'static str,
    signed_in: bool,
    installed: bool,
}

#[tauri::command]
pub async fn ai_accounts() -> Vec<Account> {
    tauri::async_runtime::spawn_blocking(|| {
        let path = fresh_path();
        PROVIDERS
            .iter()
            .map(|provider| {
                let signed_in = signed_in(provider.id);
                Account {
                    id: provider.id,
                    name: provider.name,
                    signed_in,
                    // A saved session means the tool is there, wherever it lives.
                    installed: signed_in || find_tool(provider.tool, &path).is_some(),
                }
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

/// Opens the provider's own login in a terminal, or its install page when
/// the tool is missing. `id` must be one of the fixed providers above.
#[tauri::command]
pub async fn ai_sign_in(id: String) -> Result<(), String> {
    let provider = *PROVIDERS
        .iter()
        .find(|provider| provider.id == id)
        .ok_or("proveedor desconocido")?;
    tauri::async_runtime::spawn_blocking(move || {
        let path = fresh_path();
        let Some(tool) = find_tool(provider.tool, &path) else {
            return launcher::open(provider.install).map_err(|error| error.to_string());
        };
        // `start` gives the login its own console window that stays open, so
        // the user can follow the tool's instructions and read its result.
        // The tool runs by its full path, with today's PATH for what it needs.
        Command::new("cmd")
            .args(["/c", "start", provider.name, "cmd", "/k"])
            .arg(&tool)
            .args(provider.login)
            .env("PATH", &path)
            .current_dir(home().unwrap_or_else(env::temp_dir))
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn home() -> Option<PathBuf> {
    env::var_os("USERPROFILE").map(PathBuf::from)
}

/// PATH as Windows has it now. The dock starts at sign-in and Settings
/// inherits its environment, so a tool installed later (which adds its
/// folder to the user's PATH) would otherwise stay invisible until the next
/// sign-in. This process's PATH comes first, then the saved user and system
/// ones.
fn fresh_path() -> std::ffi::OsString {
    let mut folders: Vec<PathBuf> = env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default();
    for saved in [
        registry::text(registry::Root::CurrentUser, "Environment", "Path"),
        registry::text(
            registry::Root::LocalMachine,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
            "Path",
        ),
    ]
    .into_iter()
    .flatten()
    {
        for folder in env::split_paths(&saved) {
            if !folder.as_os_str().is_empty() && !folders.contains(&folder) {
                folders.push(folder);
            }
        }
    }
    env::join_paths(folders).unwrap_or_default()
}

/// The tool's path: searched on PATH as `cmd` would (each folder, each
/// PATHEXT), then in the places its installers use when PATH does not list
/// them.
fn find_tool(tool: &str, path: &std::ffi::OsStr) -> Option<PathBuf> {
    let extensions = env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
    env::split_paths(path)
        .find_map(|folder| {
            extensions
                .split(';')
                .filter(|extension| !extension.is_empty())
                .map(|extension| folder.join(format!("{tool}{extension}")))
                .find(|candidate| candidate.is_file())
        })
        .or_else(|| {
            known_locations(tool)
                .into_iter()
                .find(|candidate| candidate.is_file())
        })
}

fn known_locations(tool: &str) -> Vec<PathBuf> {
    let home = home();
    let app_data = env::var_os("APPDATA").map(PathBuf::from);
    let mut places: Vec<PathBuf> = Vec::new();
    if let Some(home) = &home {
        places.push(home.join(format!(".local/bin/{tool}.exe")));
    }
    if let Some(app_data) = &app_data {
        places.push(app_data.join(format!("npm/{tool}.cmd")));
    }
    match tool {
        "claude" => {
            if let Some(home) = &home {
                places.push(home.join(".claude/local/claude.cmd"));
                places.push(home.join(".claude/local/claude.exe"));
                // The Claude desktop app runs its own Claude Code from its
                // data folder; its updater also leaves copies here.
                places.extend(desktop_app_claude());
                places.extend(newest_download(&home.join(".claude/downloads")));
            }
        }
        "opencode" => {
            if let Some(home) = &home {
                places.push(home.join(".opencode/bin/opencode.exe"));
            }
        }
        _ => {}
    }
    places
}

/// The Claude Code the Claude desktop app runs:
/// `%LOCALAPPDATA%\Claude-Data\<profile>\claude-code\<version>\<build>\claude.exe`,
/// the most recently written complete one.
fn desktop_app_claude() -> Option<PathBuf> {
    let data = PathBuf::from(env::var_os("LOCALAPPDATA")?).join("Claude-Data");
    let children = |folder: &Path| -> Vec<PathBuf> {
        std::fs::read_dir(folder)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.is_dir())
                    .collect()
            })
            .unwrap_or_default()
    };
    children(&data)
        .iter()
        .flat_map(|profile| children(&profile.join("claude-code")))
        .flat_map(|version| children(&version))
        .map(|build| build.join("claude.exe"))
        .filter(|exe| exe.is_file() && complete_executable(exe))
        .filter_map(|exe| Some((exe.metadata().ok()?.modified().ok()?, exe)))
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, exe)| exe)
}

/// The most recently written complete `claude-*.exe` in `folder`. A download
/// cut short keeps its name but not its body, and Windows refuses to run it.
fn newest_download(folder: &Path) -> Option<PathBuf> {
    std::fs::read_dir(folder)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            name.starts_with("claude-") && name.ends_with(".exe")
        })
        .filter(|entry| complete_executable(&entry.path()))
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

/// Whether `path` is a whole Windows program: its headers are there and
/// every section they list ends inside the file.
fn complete_executable(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let Ok(length) = file.metadata().map(|metadata| metadata.len()) else {
        return false;
    };
    let mut head = vec![0u8; 4096];
    let Ok(read) = file.read(&mut head) else {
        return false;
    };
    pe_sections_end(&head[..read]).is_some_and(|end| end > 0 && end <= length)
}

/// Where the last section of a PE image ends, from its first bytes.
fn pe_sections_end(head: &[u8]) -> Option<u64> {
    let u16_at = |at: usize| Some(u16::from_le_bytes(head.get(at..at + 2)?.try_into().ok()?));
    let u32_at = |at: usize| Some(u32::from_le_bytes(head.get(at..at + 4)?.try_into().ok()?));
    if head.get(..2)? != b"MZ" {
        return None;
    }
    let pe = u32_at(0x3C)? as usize;
    if head.get(pe..pe + 4)? != b"PE\0\0" {
        return None;
    }
    let sections = usize::from(u16_at(pe + 6)?);
    let table = pe + 24 + usize::from(u16_at(pe + 20)?);
    (0..sections)
        .map(|index| {
            let entry = table + index * 40;
            Some(u64::from(u32_at(entry + 20)?) + u64::from(u32_at(entry + 16)?))
        })
        .try_fold(0u64, |end, section| Some(end.max(section?)))
}

mod registry {
    use windows::{
        Win32::System::Registry::{
            HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW,
        },
        core::HSTRING,
    };

    pub enum Root {
        CurrentUser,
        LocalMachine,
    }

    /// A string value, with `%VARIABLES%` expanded as Windows expands PATH.
    pub fn text(root: Root, key: &str, value: &str) -> Option<String> {
        let root: HKEY = match root {
            Root::CurrentUser => HKEY_CURRENT_USER,
            Root::LocalMachine => HKEY_LOCAL_MACHINE,
        };
        let (key, value) = (HSTRING::from(key), HSTRING::from(value));
        let mut size = 0u32;
        // SAFETY: A size query first, then a buffer of exactly that size;
        // RRF_RT_REG_SZ also expands REG_EXPAND_SZ values.
        unsafe {
            RegGetValueW(
                root,
                &key,
                &value,
                RRF_RT_REG_SZ,
                None,
                None,
                Some(&mut size),
            )
            .ok()
            .ok()?;
            let mut buffer = vec![0u16; (size as usize).div_ceil(2)];
            RegGetValueW(
                root,
                &key,
                &value,
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut size),
            )
            .ok()
            .ok()?;
            let len = buffer
                .iter()
                .position(|&unit| unit == 0)
                .unwrap_or(buffer.len());
            Some(String::from_utf16_lossy(&buffer[..len]))
        }
    }
}
fn read_json(path: &Path) -> Option<Value> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() > 1024 * 1024 {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

/// Whether a JSON value at `pointer` is a non-empty string.
fn has_text(value: &Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
}

/// OpenCode's credential files, where the dock looks for them too.
fn opencode_auth() -> Vec<Value> {
    let mut paths = Vec::new();
    match env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        Some(data) if data.is_absolute() => paths.push(data.join("opencode/auth.json")),
        _ => paths.extend(home().map(|home| home.join(".local/share/opencode/auth.json"))),
    }
    for variable in ["LOCALAPPDATA", "APPDATA"] {
        if let Some(folder) = env::var_os(variable).map(PathBuf::from) {
            paths.push(folder.join("opencode/auth.json"));
        }
    }
    paths.iter().filter_map(|path| read_json(path)).collect()
}

fn opencode_key(auth: &Value, id: &str) -> bool {
    auth.get(id).is_some_and(|entry| {
        entry
            .get("key")
            .and_then(Value::as_str)
            .is_some_and(|key| !key.trim().is_empty() && key != "opencode-oauth-dummy-key")
    })
}

fn env_set(variable: &str) -> bool {
    env::var(variable).is_ok_and(|value| !value.trim().is_empty())
}

fn signed_in(id: &str) -> bool {
    match id {
        "claude" => env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|| home().map(|home| home.join(".claude")))
            .and_then(|root| read_json(&root.join(".credentials.json")))
            .is_some_and(|value| has_text(&value, "/claudeAiOauth/accessToken")),
        "codex" => env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .or_else(|| home().map(|home| home.join(".codex")))
            .and_then(|root| read_json(&root.join("auth.json")))
            .is_some_and(|value| {
                has_text(&value, "/tokens/access_token") || has_text(&value, "/OPENAI_API_KEY")
            }),
        "opencode" => {
            env_set("OPENCODE_API_KEY")
                || opencode_auth()
                    .iter()
                    .any(|auth| opencode_key(auth, "opencode-go"))
        }
        "zai" => {
            env_set("ZAI_API_KEY")
                || env_set("ZHIPU_API_KEY")
                || opencode_auth()
                    .iter()
                    .any(|auth| opencode_key(auth, "zai-coding-plan") || opencode_key(auth, "zai"))
        }
        "gemini" => home().is_some_and(|home| {
            home.join(".gemini/oauth_creds.json").is_file()
                || home.join(".gemini/antigravity/daemon").is_dir()
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_text_should_need_a_non_empty_string() {
        let value = serde_json::json!({"a": {"b": "x"}, "c": " ", "d": 3});
        assert!(has_text(&value, "/a/b"));
        assert!(!has_text(&value, "/c"));
        assert!(!has_text(&value, "/d"));
        assert!(!has_text(&value, "/missing"));
    }

    #[test]
    fn opencode_key_should_ignore_the_dummy_key() {
        let auth = serde_json::json!({
            "opencode-go": {"type": "api", "key": "opencode-oauth-dummy-key"},
            "zai": {"type": "api", "key": "abc"},
        });
        assert!(!opencode_key(&auth, "opencode-go"));
        assert!(opencode_key(&auth, "zai"));
    }

    #[test]
    fn every_provider_should_have_an_https_install_page() {
        for provider in PROVIDERS {
            assert!(provider.install.starts_with("https://"), "{}", provider.id);
        }
    }

    /// A minimal PE image whose one section ends at `section_end`, padded to `length`.
    fn fake_exe(section_end: u32, length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length.max(512)];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        bytes[0x86..0x88].copy_from_slice(&1u16.to_le_bytes()); // one section
        bytes[0x94..0x96].copy_from_slice(&0u16.to_le_bytes()); // no optional header
        let entry = 0x80 + 24;
        bytes[entry + 16..entry + 20].copy_from_slice(&(section_end - 0x200).to_le_bytes());
        bytes[entry + 20..entry + 24].copy_from_slice(&0x200u32.to_le_bytes());
        bytes.truncate(length.max(0x100));
        bytes
    }

    #[test]
    fn pe_sections_end_should_read_the_last_section() {
        assert_eq!(pe_sections_end(&fake_exe(0x1000, 0x1000)), Some(0x1000));
        assert_eq!(pe_sections_end(b"not a program"), None);
    }

    #[test]
    fn newest_download_should_skip_a_download_cut_short() -> std::io::Result<()> {
        let folder = env::temp_dir().join(format!("bahia-ai-cut-{}", std::process::id()));
        std::fs::create_dir_all(&folder)?;
        std::fs::write(
            folder.join("claude-1.0.0-win32-x64.exe"),
            fake_exe(0x1000, 0x1000),
        )?;
        std::thread::sleep(std::time::Duration::from_millis(20));
        // Newer, but its section runs past the end of the file.
        std::fs::write(
            folder.join("claude-2.0.0-win32-x64.exe"),
            fake_exe(0x4000, 0x1000),
        )?;
        let found = newest_download(&folder);
        std::fs::remove_dir_all(&folder)?;
        assert_eq!(
            found.and_then(|path| path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())),
            Some("claude-1.0.0-win32-x64.exe".to_owned())
        );
        Ok(())
    }

    #[test]
    fn newest_download_should_pick_the_latest_claude_exe() -> std::io::Result<()> {
        let folder = env::temp_dir().join(format!("bahia-ai-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder)?;
        std::fs::write(
            folder.join("claude-1.0.0-win32-x64.exe"),
            fake_exe(0x1000, 0x1000),
        )?;
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(
            folder.join("claude-2.0.0-win32-x64.exe"),
            fake_exe(0x1000, 0x1000),
        )?;
        std::fs::write(folder.join("other.exe"), fake_exe(0x1000, 0x1000))?;
        let found = newest_download(&folder);
        std::fs::remove_dir_all(&folder)?;
        assert_eq!(
            found.and_then(|path| path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())),
            Some("claude-2.0.0-win32-x64.exe".to_owned())
        );
        Ok(())
    }

    #[test]
    fn fresh_path_should_keep_this_process_path_first() {
        let fresh = fresh_path();
        let first = env::var_os("PATH").and_then(|path| env::split_paths(&path).next());
        assert_eq!(env::split_paths(&fresh).next(), first);
    }
}
