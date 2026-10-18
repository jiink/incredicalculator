const ACTIVE_DIVIDER: u16 = 1;
const IDLE_DIVIDER: u16 = 12;

struct ClockState {
    idle_allowed: bool,
    active_users: u32,
}

impl ClockState {
    const fn new() -> Self {
        Self {
            idle_allowed: false,
            active_users: 0,
        }
    }

    fn divider(&self) -> u16 {
        if self.idle_allowed && self.active_users == 0 {
            IDLE_DIVIDER
        } else {
            ACTIVE_DIVIDER
        }
    }

    fn acquire(&mut self) {
        self.active_users = self
            .active_users
            .checked_add(1)
            .expect("clock guard overflow");
    }

    fn release(&mut self) {
        self.active_users = self
            .active_users
            .checked_sub(1)
            .expect("unbalanced clock guard");
    }
}

#[cfg(target_arch = "arm")]
pub use hardware::{keep_awake, set_idle_allowed};

#[cfg(target_arch = "arm")]
mod hardware {
    use super::ClockState;
    use core::cell::RefCell;
    use critical_section::Mutex;

    static CLOCK_STATE: Mutex<RefCell<ClockState>> = Mutex::new(RefCell::new(ClockState::new()));

    fn update_state(update: impl FnOnce(&mut ClockState)) {
        // The RP critical-section implementation locks both cores. Keep the
        // accounting and divider write atomic with respect to every client.
        critical_section::with(|cs| {
            let mut state = CLOCK_STATE.borrow(cs).borrow_mut();
            let previous_divider = state.divider();
            update(&mut state);
            let divider = state.divider();
            if divider != previous_divider {
                embassy_rp::pac::CLOCKS.clk_sys_div().write(|w| {
                    w.set_int(divider);
                    w.set_frac(0);
                });
            }
        });
    }

    pub fn set_idle_allowed(allowed: bool) {
        update_state(|state| state.idle_allowed = allowed);
    }

    // Embassy caches the boot clock rate. Hold this guard while using any
    // sys-clock-dependent hardware, including pending DMA and PIO samples.
    #[must_use]
    pub struct AwakeGuard;

    pub fn keep_awake() -> AwakeGuard {
        update_state(ClockState::acquire);
        AwakeGuard
    }

    impl Drop for AwakeGuard {
        fn drop(&mut self) {
            update_state(ClockState::release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ACTIVE_DIVIDER, ClockState, IDLE_DIVIDER};

    #[test]
    fn boots_at_full_speed() {
        assert_eq!(ClockState::new().divider(), ACTIVE_DIVIDER);
    }

    #[test]
    fn slows_only_when_idle_is_allowed_and_all_clients_are_idle() {
        let mut state = ClockState::new();
        state.acquire();
        state.idle_allowed = true;
        assert_eq!(state.divider(), ACTIVE_DIVIDER);
        state.release();
        assert_eq!(state.divider(), IDLE_DIVIDER);
    }

    #[test]
    fn activity_on_either_core_restores_full_speed_until_the_last_guard_drops() {
        let mut state = ClockState::new();
        state.idle_allowed = true;
        assert_eq!(state.divider(), IDLE_DIVIDER);
        state.acquire();
        assert_eq!(state.divider(), ACTIVE_DIVIDER);
        state.acquire();
        state.release();
        assert_eq!(state.divider(), ACTIVE_DIVIDER);
        state.release();
        assert_eq!(state.divider(), IDLE_DIVIDER);
    }

    #[test]
    fn wake_disables_idle_clock_even_after_all_guards_drop() {
        let mut state = ClockState::new();
        state.idle_allowed = true;
        state.acquire();
        state.idle_allowed = false;
        state.release();
        assert_eq!(state.divider(), ACTIVE_DIVIDER);
    }
}
