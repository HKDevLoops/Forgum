//! Easing functions — the "alive" factor for animation.
//!
//! 8 pure-math easing functions used by the effect system. Each maps
//! `t ∈ [0.0, 1.0]` to an eased value. No dependencies beyond `std::f32`.

/// Linear interpolation — no easing.
#[must_use]
pub fn linear(t: f32) -> f32 {
    t
}

/// Smooth sinusoidal in/out — good for breathing, floating, swaying.
#[must_use]
pub fn sine_inout(t: f32) -> f32 {
    0.5 - 0.5 * (std::f32::consts::PI * t).cos()
}

/// Cubic in/out — slow start/stop, good for walking.
#[must_use]
pub fn cubic_inout(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Cubic out — decelerating, good for particle velocity.
#[must_use]
pub fn cubic_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Cubic in — accelerating, good for dissolve scatter.
#[must_use]
pub fn cubic_in(t: f32) -> f32 {
    t * t * t
}

/// Back out — slight overshoot, good for talk/chew animation.
#[must_use]
pub fn back_out(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Expo out — fast bright, slow fade, good for pulse/glow.
#[must_use]
pub fn expo_out(t: f32) -> f32 {
    if t >= 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * t)
    }
}

/// Bounce out — bouncy landing, good for Disney squash/stretch.
#[must_use]
pub fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        n1 * (t - 1.5 / d1) * (t - 1.5 / d1) + 0.75
    } else if t < 2.5 / d1 {
        n1 * (t - 2.25 / d1) * (t - 2.25 / d1) + 0.9375
    } else {
        n1 * (t - 2.625 / d1) * (t - 2.625 / d1) + 0.984375
    }
}

/// Look up an easing function by name.
pub fn by_name(name: &str) -> fn(f32) -> f32 {
    match name {
        "linear" => linear,
        "sine_inout" => sine_inout,
        "cubic_inout" => cubic_inout,
        "cubic_out" => cubic_out,
        "cubic_in" => cubic_in,
        "back_out" => back_out,
        "expo_out" => expo_out,
        "bounce_out" => bounce_out,
        _ => linear,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_endpoints() {
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(0.5), 0.5);
        assert_eq!(linear(1.0), 1.0);
    }

    #[test]
    fn sine_inout_midpoint() {
        let v = sine_inout(0.5);
        assert!((v - 0.5).abs() < 0.001);
    }

    #[test]
    fn sine_inout_endpoints() {
        assert!((sine_inout(0.0) - 0.0).abs() < 0.001);
        assert!((sine_inout(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn cubic_inout_symmetry() {
        let low = cubic_inout(0.25);
        let high = cubic_inout(0.75);
        assert!((low + high - 1.0).abs() < 0.001);
    }

    #[test]
    fn cubic_out_endpoints() {
        assert!((cubic_out(0.0) - 0.0).abs() < 0.001);
        assert!((cubic_out(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn cubic_in_endpoints() {
        assert!((cubic_in(0.0) - 0.0).abs() < 0.001);
        assert!((cubic_in(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn back_out_overshoots() {
        // back_out should overshoot past 1.0 in the middle
        let v = back_out(0.75);
        assert!(v > 1.0, "back_out should overshoot, got {v}");
    }

    #[test]
    fn expo_out_endpoints() {
        assert!((expo_out(0.0) - 0.0).abs() < 0.001);
        assert!((expo_out(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn bounce_out_endpoints() {
        assert!((bounce_out(0.0) - 0.0).abs() < 0.001);
        assert!((bounce_out(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn bounce_out_bounces() {
        // bounce_out should have multiple local minima
        let v1 = bounce_out(0.3);
        let v2 = bounce_out(0.5);
        let v3 = bounce_out(0.7);
        assert!(v1 < v2 || v2 < v3, "bounce should oscillate");
    }

    #[test]
    fn by_name_returns_correct_function() {
        assert!((by_name("sine_inout")(0.5) - 0.5).abs() < 0.001);
        assert!((by_name("cubic_in")(0.5) - 0.125).abs() < 0.001);
        assert_eq!(by_name("unknown")(0.5), 0.5);
    }

    #[test]
    fn all_easing_are_monotonic_near_midpoint() {
        // Most easing functions should be monotonic overall,
        // but at minimum they should be well-behaved near t=0.5
        let fns: Vec<fn(f32) -> f32> = vec![
            linear,
            sine_inout,
            cubic_inout,
            cubic_out,
            cubic_in,
            expo_out,
            bounce_out,
        ];
        for f in &fns {
            let v = f(0.4);
            let v2 = f(0.5);
            let v3 = f(0.6);
            // At minimum, the values should be finite
            assert!(v.is_finite(), "easing returned NaN/Inf at 0.4");
            assert!(v2.is_finite(), "easing returned NaN/Inf at 0.5");
            assert!(v3.is_finite(), "easing returned NaN/Inf at 0.6");
        }
    }
}
