//! Attention badges, bounce, mascot power moments and recycle-bin work.

use std::{sync::atomic::Ordering, thread, time::Instant};

use anyhow::Result;
use bahia_core::attention::Attention;
use bahia_shell::{
    power, recycle_bin, status,
    system::animations_enabled,
    tracker::{self, TrackedWindow},
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, WPARAM},
    System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize},
    UI::WindowsAndMessaging::{GetCursorPos, KillTimer, PostMessageW, SetTimer},
};

use crate::{mascot, menubar::MenuBar};

use super::{
    ATTENTION_HOPS, App, BIN_QUERY, BIN_STATE, Bounce, CHARGING_AGAIN, Entry, HIDE_DELAY_MS,
    HIDE_TIMER, MAIN_WINDOW, MASCOT_POLL_MS, MASCOT_TIMER, Role, window_of,
};

pub(super) fn app_has_attention(
    attention: &Attention,
    windows: &[TrackedWindow],
    id: &str,
) -> bool {
    windows
        .iter()
        .any(|window| window.app_id == id && attention.contains(window.hwnd))
}

pub(super) fn entry_has_attention(
    attention: &Attention,
    windows: &[TrackedWindow],
    entry: &Entry,
) -> bool {
    match entry.role {
        Role::App => entry
            .id
            .as_deref()
            .is_some_and(|id| app_has_attention(attention, windows, id)),
        Role::Group => entry
            .members
            .iter()
            .any(|id| app_has_attention(attention, windows, id)),
        Role::Window => entry
            .id
            .as_deref()
            .and_then(window_of)
            .is_some_and(|hwnd| attention.contains(hwnd)),
        _ => false,
    }
}

pub(super) fn active_attention_window(windows: &[TrackedWindow], foreground: isize) -> isize {
    // Windows may briefly report its last minimized window as foreground.
    if windows
        .iter()
        .any(|window| window.hwnd == foreground && !window.minimized)
    {
        foreground
    } else {
        0
    }
}

pub(super) fn power_act(
    before: Option<(Option<bool>, bool)>,
    now: (Option<bool>, bool),
    charging_allowed: bool,
) -> Option<mascot::Act> {
    let before = before?;
    // Unplugging can also enable energy saver. Keep the charger reaction
    // instead of replacing it with the less specific saver-only moment.
    if now.0 == Some(false) && before.0 == Some(true) {
        Some(mascot::Act::Unplugged)
    } else if now.1 && !before.1 {
        Some(mascot::Act::Sleepy)
    } else if now.0 == Some(true) && before.0 == Some(false) && charging_allowed {
        Some(mascot::Act::Charging)
    } else {
        None
    }
}

/// Checks the recycle bin on a background thread by scanning each drive
/// (measured at 50–75 ms), so the UI is not blocked. The result arrives via
/// `BIN_STATE`.
pub(super) fn query_bin_in_background() {
    if !BIN_QUERY.request() {
        return;
    }
    let window = MAIN_WINDOW.load(Ordering::Relaxed);
    let spawned = thread::Builder::new()
        .name("bahia-recycle-query".into())
        .spawn(move || {
            power::set_thread_efficiency(true);
            // SAFETY: COM is initialized and released on this thread; the message
            // is harmless if the window no longer exists.
            unsafe {
                let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                loop {
                    BIN_QUERY.begin_pass();
                    let full = recycle_bin::has_items();
                    let _ = PostMessageW(
                        Some(HWND(window as *mut _)),
                        BIN_STATE,
                        WPARAM(usize::from(full)),
                        LPARAM(0),
                    );
                    if !BIN_QUERY.needs_another_pass() {
                        break;
                    }
                }
                if com.is_ok() {
                    CoUninitialize();
                }
            }
        });
    if let Err(error) = spawned {
        BIN_QUERY.0.store(0, Ordering::Release);
        eprintln!("bahia: no se pudo consultar la Papelera: {error}");
    }
}

impl App {
    /// The crab plays `act` (see `mascot`), unless it is out already, it is
    /// turned off, or something runs full screen.
    pub(super) fn perform(&mut self, act: mascot::Act, place: mascot::Place) {
        if !self.can_perform() {
            return;
        }
        let scale = self.stage.as_ref().map_or(1.0, |stage| stage.scale);
        let Some(mascot) = mascot::Mascot::play(&self.gfx, act, place, scale) else {
            return;
        };
        self.mascot = Some(mascot);
        // SAFETY: A timer on our own window, killed when the act ends.
        unsafe {
            if SetTimer(Some(self.input), MASCOT_TIMER, MASCOT_POLL_MS, None) == 0 {
                self.stop_mascot();
                return;
            }
        }
        if matches!(place, mascot::Place::Along { .. }) && self.config.hides_automatically() {
            // Reveal only after the mascot and its cleanup timer exist.
            unsafe {
                let _ = KillTimer(Some(self.input), HIDE_TIMER);
            }
            self.slide.target = 0.0;
            self.last_frame = Instant::now();
            if let Err(error) = self.place_windows().and_then(|()| self.render()) {
                bahia_shell::log::line(&format!("mostrar el dock para la mascota: {error:#}"));
                self.stop_mascot();
            }
        }
        if self.mascot.is_some() && act == mascot::Act::Charging {
            self.charging_shown = Some(Instant::now());
        }
    }

    pub(super) fn can_perform(&self) -> bool {
        self.config.mascot && self.mascot.is_none() && !self.fullscreen && animations_enabled()
    }

    pub(super) fn stop_mascot(&mut self) {
        let active = self.mascot.take().is_some();
        // SAFETY: These timers belong to this UI window. Always clear a stale
        // mascot timer, and resume the usual dock hiding delay after an act.
        unsafe {
            let _ = KillTimer(Some(self.input), MASCOT_TIMER);
            if active && self.config.hides_automatically() {
                SetTimer(Some(self.input), HIDE_TIMER, HIDE_DELAY_MS, None);
            }
        }
    }

    /// The dock's inner rim, for the crab to walk on. An auto-hidden
    /// dock comes up for the act and goes down after it, as the macOS Dock
    /// does when something asks for attention.
    pub(super) fn dock_edge(&self) -> Option<mascot::Place> {
        let stage = self.stage.as_ref()?;
        let margin = (stage.icon / 2.0) as i32;
        let frame = stage.frame;
        // The rim is the dock's side away from the screen edge (dock
        // coordinate across = 0), and the walk runs the dock's length.
        let start = frame.point(0.0, 0.0);
        let end = frame.point(stage.layout.width, 0.0);
        let (from, to, rim) = if frame.edge.is_vertical() {
            (start.1, end.1, start.0)
        } else {
            (start.0, end.0, start.1)
        };
        Some(mascot::Place::Along {
            left: from as i32 - margin,
            right: to as i32 + margin,
            top: rim as i32,
            edge: frame.edge,
        })
    }

    pub(super) fn mascot_tick(&mut self) -> Result<()> {
        if !self.config.mascot
            || self.fullscreen
            || !animations_enabled()
            || self.mascot.as_ref().is_none_or(mascot::Mascot::finished)
        {
            self.stop_mascot();
            return Ok(());
        }
        let mut point = POINT::default();
        // SAFETY: A valid output for the pointer position.
        if unsafe { GetCursorPos(&mut point) }.is_ok()
            && let Some(mascot) = &mut self.mascot
        {
            mascot.poke(&self.gfx, point);
        }
        Ok(())
    }

    /// A moment found by the menu bar (an AI plan's figure): the crab comes
    /// out under that provider.
    pub(super) fn take_bar_cue(&mut self) {
        let cue = self.menubar.as_mut().and_then(MenuBar::take_cue);
        if let Some((act, point)) = cue {
            self.perform(
                act,
                mascot::Place::Under {
                    x: point.x,
                    top: point.y,
                },
            );
        }
    }

    /// The charger plugged in or out, or energy saver turned on: the crab walks
    /// along the dock. Only on the change itself, never at startup.
    pub(super) fn power_moment(&mut self) {
        let now = (
            bahia_shell::system::battery().map(|battery| battery.charging),
            bahia_shell::system::energy_saver_on(),
        );
        let before = self.power_seen.replace(now);
        let charging_allowed = self
            .charging_shown
            .is_none_or(|shown| shown.elapsed() >= CHARGING_AGAIN);
        let act = power_act(before, now, charging_allowed);
        if let Some(act) = act {
            if matches!(act, mascot::Act::Charging | mascot::Act::Unplugged)
                && self.config.mascot
                && !self.fullscreen
                && animations_enabled()
            {
                // A disconnect must still show if it happens during the
                // charging reaction; keep only one scene and cleanup timer.
                self.stop_mascot();
            }
            if self.can_perform()
                && let Some(place) = self.dock_edge()
            {
                self.perform(act, place);
            }
        }
    }

    pub(super) fn set_bin_full(&mut self, full: bool) -> Result<()> {
        if full == self.bin_full {
            return Ok(());
        }
        self.bin_full = full;
        self.assets.remove(recycle_bin::PARSING_NAME);
        self.refresh()
    }

    /// `until: None` is the bounce of an app being opened, which the user can
    /// turn off; a request for attention always bounces.
    /// Length of one bounce, from Settings.
    pub(super) fn hop(&self) -> f32 {
        self.config.bounce_ms as f32 / 1000.0
    }

    pub(super) fn start_bounce(&mut self, id: String, until: Option<f32>) {
        if until.is_none() && !self.config.bounce_on_open {
            return;
        }
        self.bounces.entry(id).or_insert(Bounce {
            start: Instant::now(),
            until,
        });
        self.last_frame = Instant::now();
    }

    pub(super) fn on_attention(&mut self, hwnd: isize) -> Result<()> {
        if !self.windows.iter().any(|window| window.hwnd == hwnd) && self.read_windows() {
            self.sync_badge_visuals()?;
            self.render()?;
        }
        // A minimize/flash can arrive before the debounced tracker refresh.
        // SAFETY: Read-only query; a stale or destroyed HWND returns false.
        let minimized =
            unsafe { windows::Win32::UI::WindowsAndMessaging::IsIconic(HWND(hwnd as *mut _)) }
                .as_bool();
        let foreground = if minimized {
            0
        } else {
            active_attention_window(&self.windows, tracker::foreground_window())
        };
        if hwnd == foreground {
            return Ok(());
        }
        let app_id = self
            .windows
            .iter()
            .find(|window| window.hwnd == hwnd)
            .map(|window| window.app_id.clone());
        if let Some(app_id) = app_id {
            let changed = self.attention_enabled && self.attention.request(hwnd, foreground);
            if (changed || !self.attention_enabled)
                && animations_enabled()
                && self.config.bounce_on_attention
            {
                self.start_bounce(app_id.clone(), Some(ATTENTION_HOPS * self.hop()));
            }
            if changed {
                self.sync_badge_visuals()?;
                self.render()?;
            }
            // HSHELL_FLASH is only an attention request: it carries neither a
            // notification body nor an unread count. Repeated flashes must not
            // manufacture banners or increment message counters.
        }
        Ok(())
    }

    pub(super) fn sync_badge_visuals(&mut self) -> Result<()> {
        if let Some(stage) = &mut self.stage {
            stage.scene.set_attention(
                &self.gfx,
                self.entries.iter().map(|entry| {
                    self.attention_enabled
                        && self.config.notification_badges
                        && entry_has_attention(&self.attention, &self.windows, entry)
                }),
            )?;
        }
        if let Some(view) = &mut self.group_view {
            view.set_attention(&self.gfx, |id| {
                self.attention_enabled
                    && self.config.notification_badges
                    && app_has_attention(&self.attention, &self.windows, id)
            })?;
        }
        Ok(())
    }

    pub(super) fn taskbar_preferences_changed(&mut self) -> Result<()> {
        if let Some(watch) = &mut self.badge_watch {
            watch.arm();
        }
        let enabled = status::taskbar_badges_enabled();
        if self.attention_enabled != enabled {
            self.attention_enabled = enabled;
            self.attention.clear();
            self.sync_badge_visuals()?;
            self.render()?;
        }
        Ok(())
    }

    pub(super) fn forget_attention(&mut self, hwnd: isize) -> Result<()> {
        if self.attention.acknowledge(hwnd) {
            self.sync_badge_visuals()?;
            self.render()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod attention_tests {
    use super::super::{HSHELL_FLASH, window_id};
    use super::*;
    use bahia_core::layout::ItemKind;
    use windows::{
        Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DispatchMessageW, MSG, PM_REMOVE, PeekMessageW,
            RegisterShellHookWindow, RegisterWindowMessageW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_POPUP,
        },
        core::w,
    };

    #[test]
    fn attention_reaches_apps_groups_and_minimized_thumbnails_without_clearing_other_windows() {
        let windows = [
            TrackedWindow {
                hwnd: 7,
                app_id: "chat".into(),
                minimized: true,
            },
            TrackedWindow {
                hwnd: 8,
                app_id: "chat".into(),
                minimized: false,
            },
        ];
        let mut attention = Attention::default();
        attention.request(7, 2);
        let app = Entry {
            kind: ItemKind::Icon,
            id: Some("chat".into()),
            role: Role::App,
            running: true,
            members: Vec::new(),
        };
        let group = Entry {
            id: Some("group:chat".into()),
            role: Role::Group,
            members: vec!["chat".into()],
            ..app.clone()
        };
        let thumbnail = Entry {
            id: Some(window_id(7)),
            role: Role::Window,
            ..app.clone()
        };
        assert!(entry_has_attention(&attention, &windows, &app));
        assert!(entry_has_attention(&attention, &windows, &group));
        assert!(entry_has_attention(&attention, &windows, &thumbnail));
        assert_eq!(active_attention_window(&windows, 7), 0);
        attention.acknowledge(active_attention_window(&windows, 8));
        assert!(entry_has_attention(&attention, &windows, &app));
        attention.acknowledge(7);
        assert!(!entry_has_attention(&attention, &windows, &group));
    }

    #[test]
    #[ignore = "Requires Explorer's shell hooks; uses only off-screen test-owned windows"]
    fn native_flash_request_produces_the_shell_event_used_by_dock_badges() -> Result<()> {
        use std::time::Duration;
        use windows::Win32::UI::WindowsAndMessaging::{
            DeregisterShellHookWindow, DestroyWindow, FLASHW_STOP, FLASHW_TRAY, FLASHWINFO,
            FlashWindowEx, SW_SHOWNOACTIVATE, ShowWindow, WS_EX_APPWINDOW, WS_OVERLAPPEDWINDOW,
        };
        struct Window(HWND);
        impl Drop for Window {
            fn drop(&mut self) {
                unsafe {
                    let _ = DeregisterShellHookWindow(self.0);
                    let _ = DestroyWindow(self.0);
                }
            }
        }
        let receiver = Window(unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                w!("STATIC"),
                w!("Bahia badge receiver test"),
                WS_POPUP,
                -32000,
                -32000,
                80,
                40,
                None,
                None,
                None,
                None,
            )?
        });
        let source = Window(unsafe {
            CreateWindowExW(
                WS_EX_APPWINDOW | WS_EX_NOACTIVATE,
                w!("STATIC"),
                w!("Bahia native attention test"),
                WS_OVERLAPPEDWINDOW,
                -32000,
                -32000,
                160,
                120,
                None,
                None,
                None,
                None,
            )?
        });
        let message = unsafe {
            RegisterShellHookWindow(receiver.0).ok()?;
            let _ = ShowWindow(source.0, SW_SHOWNOACTIVATE);
            RegisterWindowMessageW(w!("SHELLHOOK"))
        };
        let pump = |duration: Duration| {
            let until = Instant::now() + duration;
            let mut found = false;
            while Instant::now() < until {
                let mut event = MSG::default();
                while unsafe { PeekMessageW(&mut event, None, 0, 0, PM_REMOVE) }.as_bool() {
                    found |= event.hwnd == receiver.0
                        && event.message == message
                        && event.wParam.0 == HSHELL_FLASH
                        && event.lParam.0 == source.0.0 as isize;
                    unsafe {
                        DispatchMessageW(&event);
                    }
                }
                if found {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            found
        };
        pump(Duration::from_millis(350));
        let mut info = FLASHWINFO {
            cbSize: size_of::<FLASHWINFO>() as u32,
            hwnd: source.0,
            dwFlags: FLASHW_TRAY,
            uCount: 2,
            dwTimeout: 100,
        };
        unsafe {
            let _ = FlashWindowEx(&info);
        }
        let found = pump(Duration::from_secs(2));
        info.dwFlags = FLASHW_STOP;
        unsafe {
            let _ = FlashWindowEx(&info);
        }
        assert!(found, "Explorer did not publish the native attention event");
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use super::super::BinQueryGate;

    #[test]
    fn recycle_notifications_share_one_scan_and_preserve_the_last_change() {
        let gate = BinQueryGate::new();
        assert!(gate.request());
        gate.begin_pass();
        for _ in 0..10_000 {
            assert!(!gate.request());
        }
        assert!(gate.needs_another_pass());
        gate.begin_pass();
        assert!(!gate.needs_another_pass());
        assert!(gate.request());
    }

    #[test]
    fn quiet_recycle_scan_allows_the_next_independent_scan() {
        let gate = BinQueryGate::new();
        assert!(gate.request());
        gate.begin_pass();
        assert!(!gate.needs_another_pass());
        assert!(gate.request());
    }
}

#[cfg(test)]
mod mascot_power_tests {
    use super::{mascot::Act, power_act};

    #[test]
    fn initial_power_state_and_repeated_notifications_do_not_play_an_act() {
        assert_eq!(power_act(None, (Some(true), true), true), None);
        assert_eq!(
            power_act(Some((Some(false), true)), (Some(false), true), true),
            None
        );
        assert_eq!(
            power_act(Some((Some(true), false)), (Some(true), false), true),
            None
        );
    }

    #[test]
    fn saver_entry_and_charging_have_distinct_acts() {
        assert_eq!(
            power_act(Some((Some(false), false)), (Some(false), true), true),
            Some(Act::Sleepy)
        );
        assert_eq!(
            power_act(Some((Some(false), false)), (Some(true), false), true),
            Some(Act::Charging)
        );
        assert_eq!(
            power_act(Some((Some(false), false)), (Some(true), true), true),
            Some(Act::Sleepy)
        );
    }

    #[test]
    fn charging_cooldown_and_unknown_battery_state_suppress_false_plug_events() {
        assert_eq!(
            power_act(Some((Some(false), false)), (Some(true), false), false),
            None
        );
        assert_eq!(
            power_act(Some((None, false)), (Some(true), false), true),
            None
        );
        assert_eq!(
            power_act(Some((Some(true), true)), (Some(false), false), true),
            Some(Act::Unplugged)
        );
    }

    #[test]
    fn unplugging_is_not_suppressed_by_charging_cooldown_or_automatic_saver() {
        assert_eq!(
            power_act(Some((Some(true), false)), (Some(false), true), false),
            Some(Act::Unplugged)
        );
        assert_eq!(
            power_act(Some((Some(true), false)), (Some(false), false), false),
            Some(Act::Unplugged)
        );
    }

    #[test]
    fn missing_or_repeated_power_readings_do_not_announce_a_disconnect() {
        for (before, now) in [
            (None, Some(false)),
            (Some(true), None),
            (Some(false), Some(false)),
        ] {
            assert_eq!(power_act(Some((before, false)), (now, false), true), None);
        }
    }
}
