//! The theme Settings chose for the dock, the menu bar and their menus and
//! panels. `Auto`, the default, follows Windows' light or dark mode; the
//! others fix both the material and whether content is drawn dark or light.
//!
//! It is process state, like the Windows theme it replaces: everything that
//! draws reads `current()` when it builds, and a change rebuilds through the
//! same keys a Windows theme change does.

use std::sync::atomic::{AtomicU8, Ordering};

use bahia_config::Appearance;
use bahia_shell::system::{transparency_enabled, uses_light_theme};

/// What the glass is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    /// The dock as it always was: the light or dark macOS glass.
    Standard,
    Frosted,
    Clear,
    Aurora,
}

#[cfg(test)]
impl Material {
    pub const ALL: [Self; 4] = [Self::Standard, Self::Frosted, Self::Clear, Self::Aurora];
}

/// What a surface is drawn for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub material: Material,
    /// Light surfaces with dark content; otherwise the reverse.
    pub light: bool,
    /// See `system::transparency_enabled`: without it every material is solid.
    pub glass: bool,
}

static CHOICE: AtomicU8 = AtomicU8::new(Appearance::Auto as u8);

pub fn set(choice: Appearance) {
    CHOICE.store(choice as u8, Ordering::Relaxed);
}

const CHOICES: [Appearance; 6] = [
    Appearance::Light,
    Appearance::Dark,
    Appearance::Frosted,
    Appearance::Clear,
    Appearance::Aurora,
    Appearance::Auto,
];

fn decode(stored: u8) -> Appearance {
    CHOICES
        .into_iter()
        .find(|choice| *choice as u8 == stored)
        .unwrap_or_default()
}

#[must_use]
pub fn resolve(choice: Appearance, system_light: bool, glass: bool) -> Look {
    let (material, light) = match choice {
        Appearance::Auto => (Material::Standard, system_light),
        Appearance::Light => (Material::Standard, true),
        Appearance::Dark => (Material::Standard, false),
        Appearance::Frosted => (Material::Frosted, true),
        Appearance::Clear => (Material::Clear, false),
        Appearance::Aurora => (Material::Aurora, false),
    };
    Look {
        material,
        light,
        glass,
    }
}

#[must_use]
pub fn current() -> Look {
    let choice = decode(CHOICE.load(Ordering::Relaxed));
    // Only `Auto` needs the Windows theme.
    let system_light = choice == Appearance::Auto && uses_light_theme();
    resolve(choice, system_light, transparency_enabled())
}

/// Whether menus and labels are drawn light, with dark content.
#[must_use]
pub fn light() -> bool {
    current().light
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_follows_windows_and_the_others_do_not() {
        for system_light in [true, false] {
            assert_eq!(
                resolve(Appearance::Auto, system_light, true).light,
                system_light
            );
            assert!(resolve(Appearance::Light, system_light, true).light);
            assert!(resolve(Appearance::Frosted, system_light, true).light);
            for dark in [Appearance::Dark, Appearance::Clear, Appearance::Aurora] {
                assert!(!resolve(dark, system_light, true).light);
            }
        }
    }

    #[test]
    fn every_choice_decodes_as_itself() {
        for choice in CHOICES {
            assert_eq!(decode(choice as u8), choice);
        }
    }
}
