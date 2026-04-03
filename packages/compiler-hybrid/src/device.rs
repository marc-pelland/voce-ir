//! Device profile — describes the target execution environment.
//!
//! The device profile determines which compile targets are available
//! and influences target selection decisions.

/// Description of the target device's capabilities.
#[derive(Debug, Clone)]
pub struct DeviceProfile {
    pub name: String,
    /// Whether WebGPU is available.
    pub has_webgpu: bool,
    /// Whether WASM is available (always true in modern browsers).
    pub has_wasm: bool,
    /// GPU VRAM in MB (0 = no GPU).
    pub gpu_vram_mb: u32,
    /// Screen width in logical pixels.
    pub screen_width: u32,
    /// Number of CPU cores.
    pub cpu_cores: u32,
    /// Network type hint.
    pub network: NetworkType,
}

#[derive(Debug, Clone, Default)]
pub enum NetworkType {
    #[default]
    Broadband,
    Mobile4G,
    Slow,
    Offline,
}

impl DeviceProfile {
    /// Desktop with dedicated GPU.
    pub fn desktop() -> Self {
        Self {
            name: "desktop".to_string(),
            has_webgpu: true,
            has_wasm: true,
            gpu_vram_mb: 4096,
            screen_width: 1920,
            cpu_cores: 8,
            network: NetworkType::Broadband,
        }
    }

    /// Mobile high-end (iPhone 15, Pixel 8).
    pub fn mobile_high() -> Self {
        Self {
            name: "mobile-high".to_string(),
            has_webgpu: true,
            has_wasm: true,
            gpu_vram_mb: 1024,
            screen_width: 390,
            cpu_cores: 6,
            network: NetworkType::Mobile4G,
        }
    }

    /// Mobile low-end (budget Android).
    pub fn mobile_low() -> Self {
        Self {
            name: "mobile-low".to_string(),
            has_webgpu: false,
            has_wasm: true,
            gpu_vram_mb: 0,
            screen_width: 360,
            cpu_cores: 4,
            network: NetworkType::Slow,
        }
    }
}
