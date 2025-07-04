//! Wave animation plugin for ASS subtitles
//! 
//! This plugin demonstrates dynamic loading and provides a \wave tag
//! that creates a sinusoidal wave animation effect.

use ass_core::plugin::{
    register_tag, Tag, TagArgument, TagArgumentType, TagParseError, 
    TagApplicationError, AnimationState, ActiveAnimation, AnimationMode
};

/// Wave animation tag (\wave)
/// Usage: \wave(amplitude,frequency,phase)
pub struct WaveTag;

impl Tag for WaveTag {
    fn name(&self) -> &'static str {
        "wave"
    }

    fn expected_args(&self) -> &[TagArgumentType] {
        &[
            TagArgumentType::Float, // amplitude
            TagArgumentType::Float, // frequency (Hz)
            TagArgumentType::Optional(Box::new(TagArgumentType::Float)), // phase offset
        ]
    }

    fn parse_args(&self, args: &[u8]) -> Result<Vec<TagArgument>, TagParseError> {
        let args_str = std::str::from_utf8(args).map_err(|_| TagParseError::InvalidArguments)?;
        let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

        if parts.is_empty() || parts.len() > 3 {
            return Err(TagParseError::WrongArgumentCount);
        }

        let amplitude: f32 = parts[0].parse()
            .map_err(|_| TagParseError::TypeMismatch(TagArgumentType::Float))?;
        
        let frequency: f32 = if parts.len() > 1 {
            parts[1].parse().map_err(|_| TagParseError::TypeMismatch(TagArgumentType::Float))?
        } else {
            1.0 // Default frequency
        };

        let phase: f32 = if parts.len() > 2 {
            parts[2].parse().map_err(|_| TagParseError::TypeMismatch(TagArgumentType::Float))?
        } else {
            0.0 // Default phase
        };

        Ok(vec![
            TagArgument::Float(amplitude),
            TagArgument::Float(frequency),
            TagArgument::Float(phase),
        ])
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(&self, args: &[TagArgument], state: &mut AnimationState) -> Result<(), TagApplicationError> {
        if args.len() < 2 {
            return Err(TagApplicationError::InvalidState);
        }

        if let (TagArgument::Float(amplitude), TagArgument::Float(frequency)) = (&args[0], &args[1]) {
            let phase = if args.len() > 2 {
                if let TagArgument::Float(p) = &args[2] {
                    *p
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Create a continuous wave animation
            let wave_duration = 10.0; // 10 second animation loop
            
            // Calculate wave positions for interpolation
            let start_y = amplitude * (phase).sin();
            let end_y = amplitude * (frequency * 2.0 * std::f32::consts::PI * wave_duration + phase).sin();

            let animation = ActiveAnimation {
                tag_name: "wave_y_offset".to_string(),
                start_time: state.current_time,
                end_time: state.current_time + wave_duration as f64,
                start_value: TagArgument::Float(start_y),
                end_value: TagArgument::Float(end_y),
                mode: AnimationMode::Linear, // Linear interpolation between sine values
            };

            state.add_animation(animation);

            // Also add horizontal wave if needed
            let wave_x_animation = ActiveAnimation {
                tag_name: "wave_x_offset".to_string(),
                start_time: state.current_time,
                end_time: state.current_time + wave_duration as f64,
                start_value: TagArgument::Float(0.0),
                end_value: TagArgument::Float(amplitude * 0.5), // Smaller horizontal movement
                mode: AnimationMode::Bezier(0.25, 0.1, 0.75, 0.9), // Smooth bezier curve
            };

            state.add_animation(wave_x_animation);
        }

        Ok(())
    }
}

/// Spiral animation tag (\spiral)
/// Usage: \spiral(radius,speed,duration)
pub struct SpiralTag;

impl Tag for SpiralTag {
    fn name(&self) -> &'static str {
        "spiral"
    }

    fn expected_args(&self) -> &[TagArgumentType] {
        &[
            TagArgumentType::Float, // radius
            TagArgumentType::Float, // speed (rotations per second)
            TagArgumentType::Float, // duration
        ]
    }

    fn parse_args(&self, args: &[u8]) -> Result<Vec<TagArgument>, TagParseError> {
        let args_str = std::str::from_utf8(args).map_err(|_| TagParseError::InvalidArguments)?;
        let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

        if parts.len() != 3 {
            return Err(TagParseError::WrongArgumentCount);
        }

        let radius: f32 = parts[0].parse()
            .map_err(|_| TagParseError::TypeMismatch(TagArgumentType::Float))?;
        let speed: f32 = parts[1].parse()
            .map_err(|_| TagParseError::TypeMismatch(TagArgumentType::Float))?;
        let duration: f32 = parts[2].parse()
            .map_err(|_| TagParseError::TypeMismatch(TagArgumentType::Float))?;

        Ok(vec![
            TagArgument::Float(radius),
            TagArgument::Float(speed),
            TagArgument::Float(duration),
        ])
    }

    fn supports_animation(&self) -> bool {
        true
    }

    fn apply(&self, args: &[TagArgument], state: &mut AnimationState) -> Result<(), TagApplicationError> {
        if args.len() != 3 {
            return Err(TagApplicationError::InvalidState);
        }

        if let (TagArgument::Float(radius), TagArgument::Float(speed), TagArgument::Float(duration)) = 
           (&args[0], &args[1], &args[2]) {
            
            // Create spiral animations for X and Y positions
            let total_rotations = speed * duration;
            let end_angle = total_rotations * 2.0 * std::f32::consts::PI;

            // X position animation (cosine)
            let x_start = *radius;
            let x_end = radius * end_angle.cos();
            
            let x_animation = ActiveAnimation {
                tag_name: "spiral_x".to_string(),
                start_time: state.current_time,
                end_time: state.current_time + *duration as f64,
                start_value: TagArgument::Float(x_start),
                end_value: TagArgument::Float(x_end),
                mode: AnimationMode::Linear,
            };

            // Y position animation (sine)
            let y_start = 0.0;
            let y_end = radius * end_angle.sin();
            
            let y_animation = ActiveAnimation {
                tag_name: "spiral_y".to_string(),
                start_time: state.current_time,
                end_time: state.current_time + *duration as f64,
                start_value: TagArgument::Float(y_start),
                end_value: TagArgument::Float(y_end),
                mode: AnimationMode::Linear,
            };

            state.add_animation(x_animation);
            state.add_animation(y_animation);
        }

        Ok(())
    }
}

// Plugin registration function called by the dynamic loader
#[no_mangle]
pub extern "C" fn register_plugin() {
    static WAVE_TAG: WaveTag = WaveTag;
    static SPIRAL_TAG: SpiralTag = SpiralTag;
    
    register_tag(&WAVE_TAG);
    register_tag(&SPIRAL_TAG);
}

// Plugin information function
#[no_mangle]
pub extern "C" fn plugin_info() -> *const i8 {
    b"Wave Animation Plugin v0.1.0 - Provides \\wave and \\spiral tags\0".as_ptr() as *const i8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_tag_parsing() {
        let tag = WaveTag;
        
        // Test with all parameters
        let args = b"10.0,2.0,0.5";
        let parsed = tag.parse_args(args).unwrap();
        assert_eq!(parsed.len(), 3);
        
        // Test with minimal parameters  
        let args = b"5.0";
        let parsed = tag.parse_args(args).unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[test]
    fn test_spiral_tag_parsing() {
        let tag = SpiralTag;
        
        let args = b"50.0,1.5,3.0";
        let parsed = tag.parse_args(args).unwrap();
        assert_eq!(parsed.len(), 3);
    }

    #[test] 
    fn test_wave_animation() {
        let tag = WaveTag;
        let mut state = AnimationState::new();
        
        let args = vec![
            TagArgument::Float(10.0),
            TagArgument::Float(2.0),
            TagArgument::Float(0.0),
        ];
        
        let result = tag.apply(&args, &mut state);
        assert!(result.is_ok());
        assert!(!state.active_animations.is_empty());
    }
}