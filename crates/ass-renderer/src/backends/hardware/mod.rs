//! Hardware accelerated backends (Vulkan, Metal)

#[cfg(feature = "vulkan")]
pub mod vulkan;

#[cfg(all(feature = "metal", target_os = "macos"))]
pub mod metal;

#[cfg(all(feature = "vulkan", not(feature = "nostd")))]
pub use vulkan::VulkanBackend;

#[cfg(all(feature = "metal", target_os = "macos"))]
pub use metal::MetalBackend;
