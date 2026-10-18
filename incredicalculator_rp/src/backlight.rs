// The deadline is measured from the last key press, just like idle_ms.
pub fn idle_backlight(brightness: u8, timeout_seconds: u32, idle_ms: u64) -> (u8, Option<u64>) {
    if timeout_seconds == 0 {
        return (brightness, None);
    }

    let timeout_ms = u64::from(timeout_seconds) * 1_000;
    let dim_ms = timeout_ms / 2;
    if idle_ms < dim_ms {
        (brightness, Some(dim_ms))
    } else if idle_ms < timeout_ms {
        (brightness / 2, Some(timeout_ms))
    } else {
        (0, None)
    }
}

pub fn idle_display_sleep(timeout_seconds: u32, idle_ms: u64) -> bool {
    timeout_seconds != 0 && idle_ms >= u64::from(timeout_seconds) * 1_000
}

#[cfg(test)]
mod tests {
    use super::{idle_backlight, idle_display_sleep};

    #[test]
    fn panel_sleeps_only_at_the_full_timeout() {
        assert!(!idle_display_sleep(30, 0));
        assert!(!idle_display_sleep(30, 15_000));
        assert!(!idle_display_sleep(30, 29_999));
        assert!(idle_display_sleep(30, 30_000));
        assert!(idle_display_sleep(30, 60_000));
    }

    #[test]
    fn panel_wakes_on_activity_or_timeout_changes() {
        assert!(idle_display_sleep(30, 30_000));
        assert!(!idle_display_sleep(30, 0));
        assert!(!idle_display_sleep(60, 30_000));
        assert!(!idle_display_sleep(0, u64::MAX));
    }

    #[test]
    fn panel_sleep_handles_maximum_timeout() {
        let timeout_ms = u64::from(u32::MAX) * 1_000;
        assert!(!idle_display_sleep(u32::MAX, timeout_ms - 1));
        assert!(idle_display_sleep(u32::MAX, timeout_ms));
    }

    #[test]
    fn dims_and_turns_off_at_the_deadlines() {
        assert_eq!(idle_backlight(128, 30, 0), (128, Some(15_000)));
        assert_eq!(idle_backlight(128, 30, 14_999), (128, Some(15_000)));
        assert_eq!(idle_backlight(128, 30, 15_000), (64, Some(30_000)));
        assert_eq!(idle_backlight(128, 30, 29_999), (64, Some(30_000)));
        assert_eq!(idle_backlight(128, 30, 30_000), (0, None));
        assert_eq!(idle_backlight(128, 30, 60_000), (0, None));
    }

    #[test]
    fn zero_timeout_disables_dimming_and_shutdown() {
        assert_eq!(idle_backlight(255, 0, u64::MAX), (255, None));
    }

    #[test]
    fn odd_timeouts_dim_at_half_a_second() {
        assert_eq!(idle_backlight(255, 1, 499), (255, Some(500)));
        assert_eq!(idle_backlight(255, 1, 500), (127, Some(1_000)));
        assert_eq!(idle_backlight(255, 1, 1_000), (0, None));
    }

    #[test]
    fn activity_restores_the_configured_brightness() {
        assert_eq!(idle_backlight(200, 30, 30_000), (0, None));
        assert_eq!(idle_backlight(200, 30, 0), (200, Some(15_000)));
    }

    #[test]
    fn brightness_and_timeout_changes_are_applied_to_current_idle_time() {
        assert_eq!(idle_backlight(200, 30, 20_000), (100, Some(30_000)));
        assert_eq!(idle_backlight(80, 30, 20_000), (40, Some(30_000)));
        assert_eq!(idle_backlight(80, 60, 20_000), (80, Some(30_000)));
        assert_eq!(idle_backlight(80, 10, 20_000), (0, None));
        assert_eq!(idle_backlight(80, 0, 20_000), (80, None));
    }

    #[test]
    fn handles_zero_brightness_and_maximum_timeout() {
        assert_eq!(idle_backlight(0, 30, 15_000), (0, Some(30_000)));
        let timeout_ms = u64::from(u32::MAX) * 1_000;
        assert_eq!(
            idle_backlight(128, u32::MAX, 0),
            (128, Some(timeout_ms / 2))
        );
        assert_eq!(idle_backlight(128, u32::MAX, timeout_ms), (0, None));
    }
}
