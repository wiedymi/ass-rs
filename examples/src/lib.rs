//! Wave Plugin - Dynamic plugin example demonstrating wave effects
//! 
//! This plugin adds support for \wave() and \spiral() tags that create
//! animated wave and spiral effects on subtitle text.

use ass_core::plugin::{
    register_tag, Tag, TagArgument, TagArgumentType, TagParseError, 
    TagApplicationError, AnimationState, ActiveAnimation, AnimationMode,
};
use std::str;

/// Wave effect tag implementation
pub struct WaveTag;

impl Tag for WaveTag {
    fn name(&self) -> &'static str {
        "wave"
    }
    
    fn supports_animation(&self) -> bool {
        true
    }
    
    fn expected_args(&self) -> &[TagArgumentType] {
        static WAVE_ARGS: &[TagArgumentType] = &[
            TagArgumentType::Float, // amplitude
            TagArgumentType::Float, // frequency
        ];
        WAVE_ARGS
    }
    
    fn parse_args(&self, args: &[u8]) -> Result<Vec<TagArgument>, TagParseError> {
        let args_str = str::from_utf8(args).map_err(|_| TagParseError::InvalidArguments)?;
        let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
        
        if parts.len() < 2 {
            return Err(TagParseError::WrongArgumentCount);
        }
        
        let amplitude: f32 = parts[0].parse().map_err(|_| {
            TagParseError::TypeMismatch(TagArgumentType::Float)
        })?;
        
        let frequency: f32 = parts[1].parse().map_err(|_| {
            TagParseError::TypeMismatch(TagArgumentType::Float)
        })?;
        
        let mut result = vec![
            TagArgument::Float(amplitude),
            TagArgument::Float(frequency),
        ];
        
        // Optional phase
        if parts.len() > 2 {
            let phase: f32 = parts[2].parse().map_err(|_| {
                TagParseError::TypeMismatch(TagArgumentType::Float)
            })?;
            result.push(TagArgument::Float(phase));
        }
        
        // Optional speed
        if parts.len() > 3 {
            let speed: f32 = parts[3].parse().map_err(|_| {
                TagParseError::TypeMismatch(TagArgumentType::Float)
            })?;
            result.push(TagArgument::Float(speed));
        }
        
        Ok(result)
    }
    
    fn apply(
        &self,
        args: &[TagArgument],
        state: &mut AnimationState,
    ) -> Result<(), TagApplicationError> {
        if args.len() < 2 {
            return Err(TagApplicationError::InvalidState);
        }
        
        if let (TagArgument::Float(amplitude), TagArgument::Float(frequency)) = 
            (&args[0], &args[1]) {
            
            let phase = if args.len() > 2 {
                if let TagArgument::Float(p) = &args[2] { *p } else { 0.0 }
            } else { 0.0 };
            
            let speed = if args.len() > 3 {
                if let TagArgument::Float(s) = &args[3] { *s } else { 1.0 }
            } else { 1.0 };
            
            // Create wave effect parameters
            state.interpolated_values.insert(
                "wave_amplitude".to_string(),
                TagArgument::Float(*amplitude),
            );
            state.interpolated_values.insert(
                "wave_frequency".to_string(),
                TagArgument::Float(*frequency),
            );
            state.interpolated_values.insert(
                "wave_phase".to_string(),
                TagArgument::Float(phase),
            );
            state.interpolated_values.insert(
                "wave_speed".to_string(),
                TagArgument::Float(speed),
            );
        }
        
        Ok(())
    }
}

/// Spiral effect tag implementation
pub struct SpiralTag;

impl Tag for SpiralTag {
    fn name(&self) -> &'static str {
        "spiral"
    }
    
    fn supports_animation(&self) -> bool {
        true
    }
    
    fn expected_args(&self) -> &[TagArgumentType] {
        static SPIRAL_ARGS: &[TagArgumentType] = &[
            TagArgumentType::Float, // radius
            TagArgumentType::Float, // speed
        ];
        SPIRAL_ARGS
    }
    
    fn parse_args(&self, args: &[u8]) -> Result<Vec<TagArgument>, TagParseError> {
        let args_str = str::from_utf8(args).map_err(|_| TagParseError::InvalidArguments)?;
        let parts: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
        
        if parts.len() < 2 {
            return Err(TagParseError::WrongArgumentCount);
        }
        
        let radius: f32 = parts[0].parse().map_err(|_| {
            TagParseError::TypeMismatch(TagArgumentType::Float)
        })?;
        
        let speed: f32 = parts[1].parse().map_err(|_| {
            TagParseError::TypeMismatch(TagArgumentType::Float)
        })?;
        
        let mut result = vec![
            TagArgument::Float(radius),
            TagArgument::Float(speed),
        ];
        
        // Optional center coordinates
        if parts.len() > 2 {
            let center_x: f32 = parts[2].parse().map_err(|_| {
                TagParseError::TypeMismatch(TagArgumentType::Float)
            })?;
            result.push(TagArgument::Float(center_x));
            
            if parts.len() > 3 {
                let center_y: f32 = parts[3].parse().map_err(|_| {
                    TagParseError::TypeMismatch(TagArgumentType::Float)
                })?;
                result.push(TagArgument::Float(center_y));
            }
        }
        
        Ok(result)
    }
    
    fn apply(
        &self,
        args: &[TagArgument],
        state: &mut AnimationState,
    ) -> Result<(), TagApplicationError> {
        if args.len() < 2 {
            return Err(TagApplicationError::InvalidState);
        }
        
        if let (TagArgument::Float(radius), TagArgument::Float(speed)) = 
            (&args[0], &args[1]) {
            
            let center_x = if args.len() > 2 {
                if let TagArgument::Float(x) = &args[2] { *x } else { 0.0 }
            } else { 0.0 };
            
            let center_y = if args.len() > 3 {
                if let TagArgument::Float(y) = &args[3] { *y } else { 0.0 }
            } else { 0.0 };
            
            // Create spiral animation
            let duration = 5.0; // 5 second duration
            let animation = ActiveAnimation {
                tag_name: "spiral_position".to_string(),
                start_time: 0.0,
                end_time: duration,
                start_value: TagArgument::Position(center_x, center_y),
                end_value: TagArgument::Position(
                    center_x + radius,
                    center_y,
                ),
                mode: AnimationMode::Linear,
            };
            
            state.add_animation(animation);
            
            // Store spiral parameters
            state.interpolated_values.insert(
                "spiral_radius".to_string(),
                TagArgument::Float(*radius),
            );
            state.interpolated_values.insert(
                "spiral_speed".to_string(),
                TagArgument::Float(*speed),
            );
            state.interpolated_values.insert(
                "spiral_center".to_string(),
                TagArgument::Position(center_x, center_y),
            );
        }
        
        Ok(())
    }
}

/// Plugin registration function - called when the plugin is loaded
#[no_mangle]
pub extern "C" fn register_plugin() {
    static WAVE_TAG: WaveTag = WaveTag;
    static SPIRAL_TAG: SpiralTag = SpiralTag;
    
    register_tag(&WAVE_TAG);
    register_tag(&SPIRAL_TAG);
}

/// Plugin information function
#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo {
    static INFO: PluginInfo = PluginInfo {
        name: "Wave Effects Plugin",
        version: "1.0.0",
        author: "ASS-RS Team",
        description: "Provides wave and spiral animation effects for subtitles",
    };
    &INFO
}

/// Plugin information structure
#[repr(C)]
pub struct PluginInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub description: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ass_core::plugin::AnimationState;
    
    #[test]
    fn test_wave_tag_parsing() {
        let wave_tag = WaveTag;
        
        // Test basic wave parameters
        let result = wave_tag.parse_args(b"10.0,2.0");
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.len(), 2);
        
        // Test with all parameters
        let result = wave_tag.parse_args(b"10.0,2.0,0.5,1.5");
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.len(), 4);
    }
    
    #[test]
    fn test_spiral_tag_parsing() {
        let spiral_tag = SpiralTag;
        
        // Test basic spiral parameters
        let result = spiral_tag.parse_args(b"50.0,1.0");
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.len(), 2);
        
        // Test with center coordinates
        let result = spiral_tag.parse_args(b"50.0,1.0,100.0,200.0");
        assert!(result.is_ok());
        let args = result.unwrap();
        assert_eq!(args.len(), 4);
    }
    
    #[test]
    fn test_wave_application() {
        let wave_tag = WaveTag;
        let mut state = AnimationState::new();
        
        let args = vec![
            TagArgument::Float(10.0),
            TagArgument::Float(2.0),
        ];
        
        let result = wave_tag.apply(&args, &mut state);
        assert!(result.is_ok());
        
        // Check that wave parameters were stored
        assert!(state.interpolated_values.contains_key("wave_amplitude"));
        assert!(state.interpolated_values.contains_key("wave_frequency"));
    }
    
    #[test]
    fn test_spiral_application() {
        let spiral_tag = SpiralTag;
        let mut state = AnimationState::new();
        
        let args = vec![
            TagArgument::Float(50.0),
            TagArgument::Float(1.0),
        ];
        
        let result = spiral_tag.apply(&args, &mut state);
        assert!(result.is_ok());
        
        // Check that spiral parameters were stored
        assert!(state.interpolated_values.contains_key("spiral_radius"));
        assert!(state.interpolated_values.contains_key("spiral_speed"));
        
        // Check that animation was created
        assert_eq!(state.active_animations.len(), 1);
    }
}