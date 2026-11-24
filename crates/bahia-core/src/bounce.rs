use std::f32::consts::PI;

pub const HOP_SECONDS: f32 = 0.7;

/// Height above the resting position after `elapsed` seconds of bouncing.
/// `height` is the maximum bounce height.
#[must_use]
pub fn lift(elapsed: f32, height: f32) -> f32 {
    lift_with(elapsed, height, HOP_SECONDS)
}

/// [`lift`] for hops lasting `hop` seconds.
#[must_use]
pub fn lift_with(elapsed: f32, height: f32, hop: f32) -> f32 {
    height * (PI * elapsed / hop.max(f32::EPSILON)).sin().abs()
}

/// End time of the current bounce, allowing it to stop without cutting off
/// the icon mid-air.
#[must_use]
pub fn end_of_hop(elapsed: f32) -> f32 {
    end_of_hop_with(elapsed, HOP_SECONDS)
}

/// [`end_of_hop`] for hops lasting `hop` seconds.
#[must_use]
pub fn end_of_hop_with(elapsed: f32, hop: f32) -> f32 {
    let hop = hop.max(f32::EPSILON);
    (elapsed / hop).ceil().max(1.0) * hop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lift_should_start_on_the_ground() {
        assert_eq!(lift(0.0, 30.0), 0.0);
    }

    #[test]
    fn lift_should_peak_at_half_a_hop() {
        assert!((lift(HOP_SECONDS / 2.0, 30.0) - 30.0).abs() < 1e-3);
    }

    #[test]
    fn lift_should_touch_the_ground_at_the_end_of_each_hop() {
        assert!(lift(HOP_SECONDS * 3.0, 30.0).abs() < 1e-2);
    }

    #[test]
    fn lift_should_never_go_below_the_ground() {
        assert!((0..200).all(|i| lift(i as f32 * 0.013, 30.0) >= 0.0));
    }

    #[test]
    fn end_of_hop_should_round_up_to_the_next_landing() {
        assert!((end_of_hop(0.9) - 2.0 * HOP_SECONDS).abs() < 1e-6);
    }

    #[test]
    fn lift_with_should_follow_its_own_hop_length() {
        assert!((lift_with(0.25, 20.0, 0.5) - 20.0).abs() < 1e-3);
        assert!((end_of_hop_with(0.6, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn end_of_hop_should_complete_at_least_one_hop() {
        assert!((end_of_hop(0.0) - HOP_SECONDS).abs() < 1e-6);
    }
}
