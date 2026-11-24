//! Optional offline gallery. Only an explicit apply command changes Windows.

use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use windows::{
    Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        System::Com::{
            CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
            CoUninitialize,
        },
        UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
    },
    core::{HSTRING, PCWSTR, PWSTR},
};

struct Wallpaper {
    name: &'static str,
    bytes: &'static [u8],
}

macro_rules! wallpaper {
    ($name:literal) => {
        Wallpaper {
            name: $name,
            bytes: include_bytes!(concat!("../../wallpapers/", $name)),
        }
    };
}

// Originals are embedded once in Settings, not downloaded or decoded at startup.
static CATALOG: [Wallpaper; 12] = [
    wallpaper!("desktop-01.jpg"),
    wallpaper!("desktop-02.jpg"),
    wallpaper!("desktop-03.jpg"),
    wallpaper!("desktop-04.jpg"),
    wallpaper!("desktop-05.jpg"),
    wallpaper!("desktop-06.jpg"),
    wallpaper!("desktop-07.jpg"),
    wallpaper!("desktop-08.jpg"),
    wallpaper!("desktop-09.jpg"),
    wallpaper!("desktop-10.jpg"),
    wallpaper!("desktop-11.jpg"),
    wallpaper!("desktop-12.jpg"),
];
static APPLYING: AtomicBool = AtomicBool::new(false);
static TEMPORARY: AtomicU64 = AtomicU64::new(0);

struct ApplyGuard;
impl ApplyGuard {
    fn acquire() -> Result<Self, String> {
        APPLYING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self)
            .map_err(|_| "A wallpaper change is already in progress".into())
    }
}
impl Drop for ApplyGuard {
    fn drop(&mut self) {
        APPLYING.store(false, Ordering::Release);
    }
}

struct Apartment(bool);
impl Apartment {
    fn new() -> Result<Self, String> {
        let status = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if status.is_err() && status != RPC_E_CHANGED_MODE {
            return Err(status.to_string());
        }
        Ok(Self(status.is_ok()))
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

struct Text(PWSTR);
impl Drop for Text {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.0.cast())) };
    }
}

fn with_desktop<T>(
    action: impl FnOnce(&IDesktopWallpaper) -> Result<T, String>,
) -> Result<T, String> {
    let _apartment = Apartment::new()?;
    // The interface is released before its apartment, on this same worker.
    let desktop: IDesktopWallpaper =
        unsafe { CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL) }
            .map_err(|error| error.to_string())?;
    action(&desktop)
}

fn cache_dir() -> Result<PathBuf, String> {
    env::var_os("LOCALAPPDATA")
        .map(|root| PathBuf::from(root).join("Bahia").join("Wallpapers"))
        .ok_or_else(|| "The local application data folder is unavailable".into())
}

fn entry(id: u8) -> Result<&'static Wallpaper, String> {
    id.checked_sub(1)
        .and_then(|index| CATALOG.get(usize::from(index)))
        .ok_or_else(|| "Unknown wallpaper".into())
}

fn selected_path(root: &Path, current: &Path) -> Option<u8> {
    CATALOG
        .iter()
        .position(|asset| {
            root.join(asset.name)
                .to_string_lossy()
                .eq_ignore_ascii_case(&current.to_string_lossy())
        })
        .map(|index| index as u8 + 1)
}

/// Read-only snapshot; an empty/multi-monitor/slideshow result is not a gallery selection.
#[tauri::command]
pub async fn wallpaper_selection() -> Result<Option<u8>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let root = cache_dir()?;
        with_desktop(|desktop| {
            let text = Text(
                unsafe { desktop.GetWallpaper(PCWSTR::null()) }
                    .map_err(|error| error.to_string())?,
            );
            if text.0.is_null() {
                return Ok(None);
            }
            let path = unsafe { text.0.to_string() }.map_err(|error| error.to_string())?;
            Ok(selected_path(&root, Path::new(&path)))
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

fn materialize(asset: &Wallpaper, root: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let destination = root.join(asset.name);
    // Bound reads even if someone replaced our cached file with a larger one.
    if let Ok(file) = fs::File::open(&destination)
        && file
            .metadata()
            .is_ok_and(|metadata| metadata.len() == asset.bytes.len() as u64)
    {
        let mut bytes = Vec::with_capacity(asset.bytes.len());
        if file
            .take(asset.bytes.len() as u64 + 1)
            .read_to_end(&mut bytes)
            .is_ok()
            && bytes == asset.bytes
        {
            return Ok(destination);
        }
    }
    let temporary = root.join(format!(
        ".{}-{}-{}.tmp",
        asset.name,
        std::process::id(),
        TEMPORARY.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(asset.bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &destination)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| error.to_string())?;
    Ok(destination)
}

fn apply_at(
    id: u8,
    root: &Path,
    set: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let asset = entry(id)?;
    let path = materialize(asset, root)?;
    set(&path)
}

/// Only accepts a catalog ID, never a URL, caller-supplied path or executable.
#[tauri::command]
pub async fn apply_wallpaper(id: u8) -> Result<(), String> {
    entry(id)?;
    let guard = ApplyGuard::acquire()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        apply_at(id, &cache_dir()?, |path| {
            with_desktop(|desktop| {
                let path = HSTRING::from(path.as_os_str());
                // NULL selects all monitors. Windows persists and broadcasts the change.
                unsafe { desktop.SetWallpaper(PCWSTR::null(), &path) }
                    .map_err(|error| error.to_string())
            })
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_complete_and_every_original_decodes_with_bounded_memory() -> Result<(), String> {
        let _apartment = Apartment::new()?;
        for id in 1..=12 {
            let asset = entry(id)?;
            assert_eq!(asset.name, format!("desktop-{id:02}.jpg"));
            let image = bahia_shell::imagery::decode(asset.bytes, 128, 40_000_000)
                .ok_or("invalid packaged wallpaper")?;
            assert!(image.width <= 128 && image.height <= 128);
        }
        Ok(())
    }

    #[test]
    fn arbitrary_ids_are_rejected_before_writes_or_system_changes() {
        let root = env::temp_dir().join(format!("bahia-wallpaper-rejected-{}", std::process::id()));
        for id in [0, 13, 255] {
            let result = apply_at(id, &root, |_| panic!("invalid ID must never reach Windows"));
            assert!(result.is_err());
        }
        assert!(!root.exists());
    }

    #[test]
    fn cache_is_durable_reused_and_repaired_and_failed_apply_is_reported() -> Result<(), String> {
        let root = env::temp_dir().join(format!(
            "bahia-wallpaper-cache-{}-{}",
            std::process::id(),
            TEMPORARY.fetch_add(1, Ordering::Relaxed)
        ));
        let asset = entry(1)?;
        let path = materialize(asset, &root)?;
        let modified = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .map_err(|error| error.to_string())?;
        materialize(asset, &root)?;
        assert_eq!(
            fs::metadata(&path)
                .and_then(|metadata| metadata.modified())
                .map_err(|error| error.to_string())?,
            modified
        );
        fs::write(&path, b"damaged cache").map_err(|error| error.to_string())?;
        materialize(asset, &root)?;
        assert_eq!(
            fs::read(&path).map_err(|error| error.to_string())?,
            asset.bytes
        );
        let result = apply_at(1, &root, |actual| {
            assert_eq!(actual, path);
            Err("Windows refused the change".into())
        });
        assert_eq!(result, Err("Windows refused the change".into()));
        assert_eq!(selected_path(&root, &path), Some(1));
        assert_eq!(
            selected_path(&root, &root.join("elsewhere/desktop-01.jpg")),
            None
        );
        fs::remove_file(path)
            .and_then(|()| fs::remove_dir(root))
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn apply_guard_rejects_overlap_and_is_released_after_failure() -> Result<(), String> {
        let guard = ApplyGuard::acquire()?;
        assert!(ApplyGuard::acquire().is_err());
        drop(guard);
        drop(ApplyGuard::acquire()?);
        Ok(())
    }

    #[test]
    #[ignore = "Read-only Windows wallpaper service probe; never changes the user's background"]
    fn native_wallpaper_service_can_read_the_current_background() -> Result<(), String> {
        with_desktop(|desktop| {
            let value = Text(
                unsafe { desktop.GetWallpaper(PCWSTR::null()) }
                    .map_err(|error| error.to_string())?,
            );
            if !value.0.is_null() {
                let _ = unsafe { value.0.to_string() }.map_err(|error| error.to_string())?;
            }
            Ok(())
        })
    }
}
