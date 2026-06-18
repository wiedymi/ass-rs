//! Animation system for ASS subtitle effects

mod controller;
mod state;
mod timing;
mod track;
mod value;

pub use controller::AnimationController;
pub use state::AnimationState;
pub use timing::{AnimationInterpolation, AnimationTag, AnimationTiming, InterpolationFn};
pub use track::AnimationTrack;
pub use value::{AnimatedResult, AnimatedValue};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_timing() {
        let timing = AnimationTiming::new(100, 200, 1.0);

        assert_eq!(timing.progress(50), 0.0);
        assert_eq!(timing.progress(100), 0.0);
        assert_eq!(timing.progress(150), 0.5);
        assert_eq!(timing.progress(200), 1.0);
        assert_eq!(timing.progress(250), 1.0);
    }

    #[test]
    fn test_animated_value_interpolation() {
        let int_anim = AnimatedValue::Integer { from: 0, to: 100 };
        if let AnimatedResult::Integer(val) = int_anim.interpolate(0.5) {
            assert_eq!(val, 50);
        } else {
            panic!("Wrong result type");
        }

        let color_anim = AnimatedValue::Color {
            from: [0, 0, 0, 255],
            to: [255, 255, 255, 255],
        };
        if let AnimatedResult::Color(val) = color_anim.interpolate(0.5) {
            assert_eq!(val, [127, 127, 127, 255]);
        } else {
            panic!("Wrong result type");
        }
    }

    #[test]
    fn test_animation_controller() {
        let mut controller = AnimationController::new();

        let timing = AnimationTiming::new(0, 100, 1.0);
        let track = AnimationTrack::new(
            "test".to_string(),
            timing,
            AnimatedValue::Float {
                from: 0.0,
                to: 100.0,
            },
            AnimationInterpolation::Linear,
        );

        controller.add_track(track);

        let state = controller.evaluate(50);
        if let Some(AnimatedResult::Float(val)) = state.get_property("test") {
            assert!((val - 50.0).abs() < 0.001);
        } else {
            panic!("Property not found or wrong type");
        }
    }
}
