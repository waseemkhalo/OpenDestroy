//! User-owned settings bounds, in macOS desktop points (not display pixels).
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl Bounds {
    pub fn valid(self) -> bool {
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|v| v.is_finite())
            && self.width >= 1.0
            && self.height >= 1.0
            && self.width <= 20000.0
            && self.height <= 20000.0
    }
    fn overlap(self, other: Self) -> f64 {
        ((self.x + self.width).min(other.x + other.width) - self.x.max(other.x)).max(0.0)
            * ((self.y + self.height).min(other.y + other.height) - self.y.max(other.y)).max(0.0)
    }
}

/// Preserve valid placements. Recover only against actual available work areas.
pub fn restore(saved: Option<Bounds>, screens: &[Bounds], preferred: usize) -> Option<Bounds> {
    let fallback = *screens.get(preferred).or_else(|| screens.first())?;
    let saved = saved.filter(|b| b.valid());
    let area = saved
        .and_then(|b| {
            screens
                .iter()
                .copied()
                .filter(|s| b.overlap(*s) > 0.0)
                .max_by(|a, c| b.overlap(*a).total_cmp(&b.overlap(*c)))
        })
        .unwrap_or(fallback);
    let mut b = saved.unwrap_or(Bounds {
        x: area.x,
        y: area.y,
        width: 680.0,
        height: 700.0,
    });
    b.width = b.width.min(area.width);
    b.height = b.height.min(area.height);
    if saved.is_none() || saved.is_some_and(|s| s.overlap(area) == 0.0) {
        b.x = area.x + (area.width - b.width) / 2.0;
        b.y = area.y + (area.height - b.height) / 2.0;
    }
    b.x = b.x.clamp(area.x, area.x + area.width - b.width);
    b.y = b.y.clamp(area.y, area.y + area.height - b.height);
    Some(b)
}

#[derive(Default)]
pub struct Placement {
    pub expanded: bool,
    pub bounds: Option<Bounds>,
    dirty: bool,
}
pub struct PanelBounds(pub Mutex<Placement>);
fn path() -> std::path::PathBuf {
    crate::app_identity::config_path().with_file_name("settings-window.json")
}
impl Default for PanelBounds {
    fn default() -> Self {
        let bounds = std::fs::read(path())
            .ok()
            .and_then(|v| serde_json::from_slice::<Bounds>(&v).ok())
            .filter(|b| b.valid());
        Self(Mutex::new(Placement {
            bounds,
            ..Default::default()
        }))
    }
}
impl PanelBounds {
    pub fn remember(&self, bounds: Bounds) {
        if let Ok(mut state) = self.0.lock() {
            if state.expanded && bounds.valid() && state.bounds != Some(bounds) {
                state.bounds = Some(bounds);
                state.dirty = true;
            }
        }
    }
    // One writer, at most once per second; no file write for every drag event.
    pub fn flush(&self) {
        let Ok(mut state) = self.0.lock() else { return };
        if !state.dirty {
            return;
        }
        let Some(bounds) = state.bounds else { return };
        let path = path();
        let Some(parent) = path.parent() else { return };
        let Ok(data) = serde_json::to_vec(&bounds) else {
            return;
        };
        let temp = path.with_extension("tmp");
        if std::fs::create_dir_all(parent).is_ok()
            && std::fs::write(&temp, data).is_ok()
            && std::fs::rename(temp, path).is_ok()
        {
            state.dirty = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn display() -> Bounds {
        Bounds {
            x: 0.0,
            y: 30.0,
            width: 1440.0,
            height: 850.0,
        }
    }
    #[test]
    fn restores_user_placement_and_size_exactly() {
        let b = Bounds {
            x: 117.0,
            y: 83.0,
            width: 840.0,
            height: 580.0,
        };
        assert_eq!(restore(Some(b), &[display()], 0), Some(b));
    }
    #[test]
    fn first_launch_uses_preferred_display() {
        let right = Bounds {
            x: 1440.0,
            ..display()
        };
        assert_eq!(restore(None, &[display(), right], 1).unwrap().x, 1820.0);
    }
    #[test]
    fn connected_negative_coordinate_display_is_preserved() {
        let left = Bounds {
            x: -1440.0,
            ..display()
        };
        let b = Bounds {
            x: -1200.0,
            y: 100.0,
            width: 960.0,
            height: 650.0,
        };
        assert_eq!(restore(Some(b), &[display(), left], 0), Some(b));
    }
    #[test]
    fn disconnected_display_recovers_without_changing_fitting_size() {
        let b = Bounds {
            x: -1200.0,
            y: 100.0,
            width: 960.0,
            height: 650.0,
        };
        let recovered = restore(Some(b), &[display()], 0).unwrap();
        assert_eq!(recovered.width, b.width);
        assert_eq!(recovered.x, 240.0);
    }
    #[test]
    fn small_display_clamps_and_corrupt_state_falls_back() {
        let small = Bounds {
            width: 800.0,
            height: 550.0,
            ..display()
        };
        let b = restore(None, &[small], 0).unwrap();
        assert_eq!((b.width, b.height), (680.0, 550.0));
        assert_eq!(
            restore(
                Some(Bounds {
                    width: f64::NAN,
                    ..b
                }),
                &[small],
                0
            ),
            Some(b)
        );
        assert_eq!(restore(None, &[], 0), None);
    }
    #[test]
    fn compact_geometry_cannot_overwrite_user_bounds() {
        let original = display();
        let state = PanelBounds(Mutex::new(Placement {
            bounds: Some(original),
            ..Default::default()
        }));
        state.remember(Bounds {
            width: 540.0,
            height: 440.0,
            ..original
        });
        assert_eq!(state.0.lock().unwrap().bounds, Some(original));
    }
}
