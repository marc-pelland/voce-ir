//! Compile-time spring ODE solver.
//!
//! Solves the damped harmonic oscillator equation:
//!   x'' + (damping/mass)*x' + (stiffness/mass)*x = 0
//!
//! Outputs a series of normalized progress values [0.0..1.0] that trace
//! the spring curve. These are emitted as CSS `linear()` easing points,
//! giving spring physics animation with zero JavaScript.
//!
//! # Example
//!
//! ```
//! use voce_compiler_dom::animation::spring::solve_spring;
//!
//! let points = solve_spring(300.0, 25.0, 1.0, 20);
//! assert!(points.len() == 20);
//! assert!((points[0] - 0.0).abs() < 0.01);
//! // Last point should be close to 1.0 (settled)
//! assert!((points[points.len() - 1] - 1.0).abs() < 0.05);
//! ```

/// Solve a spring curve and return normalized progress points.
///
/// # Arguments
/// * `stiffness` — Spring stiffness (k). Higher = faster oscillation.
/// * `damping` — Damping coefficient (c). Higher = less bounce.
/// * `mass` — Mass (m). Higher = slower, more inertia.
/// * `num_points` — Number of points to sample (15-25 for CSS linear()).
///
/// # Returns
/// Vector of progress values from 0.0 to ~1.0, tracing the spring curve.
pub fn solve_spring(stiffness: f64, damping: f64, mass: f64, num_points: usize) -> Vec<f64> {
    // Find the settling time (when displacement < 0.001)
    let duration = estimate_duration(stiffness, damping, mass);
    let dt = duration / num_points as f64;

    let mut points = Vec::with_capacity(num_points);

    // Initial conditions: displaced by -1.0 (moving from 0 to 1)
    let mut x: f64 = -1.0; // displacement from target
    let mut v: f64 = 0.0; // velocity

    for i in 0..num_points {
        // Normalized progress: 0.0 at start, 1.0 at target
        let progress = 1.0 + x;
        points.push(progress.clamp(-0.5, 2.0)); // allow overshoot but bound extremes

        // Semi-implicit Euler integration
        let spring_force = -stiffness * x;
        let damping_force = -damping * v;
        let acceleration = (spring_force + damping_force) / mass;

        v += acceleration * dt;
        x += v * dt;

        // If we're past the last point and settled, snap to 1.0
        if i == num_points - 1 {
            let last = points.last_mut().unwrap();
            if (*last - 1.0).abs() < 0.05 {
                *last = 1.0;
            }
        }
    }

    points
}

/// Estimate how long the spring takes to settle (displacement < threshold).
fn estimate_duration(stiffness: f64, damping: f64, mass: f64) -> f64 {
    let omega_n = (stiffness / mass).sqrt();
    let zeta = damping / (2.0 * (stiffness * mass).sqrt());

    if zeta >= 1.0 {
        // Overdamped or critically damped — settles without oscillation
        5.0 / (zeta * omega_n)
    } else {
        // Underdamped — oscillates before settling
        // Time to reach <0.1% of initial displacement
        -((0.001_f64).ln()) / (zeta * omega_n)
    }
}

/// Format spring points as a CSS `linear()` easing function.
///
/// # Example output
/// ```text
/// linear(0, 0.05, 0.18, 0.42, 0.71, 0.93, 1.05, 1.03, 1.01, 1)
/// ```
pub fn spring_to_css_linear(stiffness: f64, damping: f64, mass: f64) -> String {
    let points = solve_spring(stiffness, damping, mass, 20);
    let formatted: Vec<String> = points
        .iter()
        .map(|p| {
            if (*p - p.round()).abs() < 0.005 {
                format!("{}", p.round() as i32)
            } else {
                format!("{:.2}", p)
            }
        })
        .collect();

    format!("linear({})", formatted.join(", "))
}

/// Estimate the animation duration in milliseconds for a spring.
pub fn spring_duration_ms(stiffness: f64, damping: f64, mass: f64) -> f64 {
    estimate_duration(stiffness, damping, mass) * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_starts_at_zero_ends_near_one() {
        let points = solve_spring(300.0, 25.0, 1.0, 20);
        assert!((points[0] - 0.0).abs() < 0.01, "Should start near 0");
        assert!(
            (points[19] - 1.0).abs() < 0.05,
            "Should end near 1.0, got {}",
            points[19]
        );
    }

    #[test]
    fn underdamped_spring_overshoots() {
        // Low damping = overshoot
        let points = solve_spring(300.0, 10.0, 1.0, 20);
        let max = points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(max > 1.0, "Underdamped spring should overshoot, max={max}");
    }

    #[test]
    fn overdamped_spring_no_overshoot() {
        // High damping = no overshoot
        let points = solve_spring(100.0, 50.0, 1.0, 20);
        let max = points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max <= 1.05,
            "Overdamped spring should not significantly overshoot, max={max}"
        );
    }

    #[test]
    fn css_linear_output_format() {
        let css = spring_to_css_linear(300.0, 25.0, 1.0);
        assert!(css.starts_with("linear("));
        assert!(css.ends_with(')'));
        assert!(css.contains(", "));
        // Should have 20 values
        let count = css.matches(',').count() + 1;
        assert_eq!(count, 20, "Should have 20 points, got {count}");
    }

    #[test]
    fn spring_duration_reasonable() {
        let ms = spring_duration_ms(300.0, 25.0, 1.0);
        assert!(
            ms > 100.0 && ms < 2000.0,
            "Duration should be 100-2000ms, got {ms}"
        );
    }
}
