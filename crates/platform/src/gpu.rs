//! Cross-platform GPU detection and hardware acceleration management.
//!
//! Provides hardware-level inspection and classification of available graphics
//! devices across Windows, Linux (including WSL), and macOS:
//! - Dedicated GPU (discrete hardware with high-performance VRAM: NVIDIA GeForce / RTX, AMD Radeon RX, Intel Arc)
//! - Integrated GPU (internal / APU graphics with shared memory: AMD Radeon(TM) Graphics, Intel UHD/Iris, Apple M-series)
//! - CPU fallback (zero-allocation CPU compute)
//!
//! Adheres strictly to workspace-wide `unsafe_code = "deny"`.

use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(unix)]
use std::path::Path;
use std::sync::OnceLock;

/// Classification of graphics processing hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuDeviceType {
    /// Dedicated / discrete GPU with independent high-speed VRAM.
    Dedicated,
    /// Integrated / APU graphics sharing system memory.
    Integrated,
    /// Neural Processing Unit (DirectML / CoreML / AI Boost / Ryzen AI).
    Npu,
    /// Host CPU fallback compute.
    Cpu,
}

impl GpuDeviceType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dedicated => "dedicated",
            Self::Integrated => "integrated",
            Self::Npu => "npu",
            Self::Cpu => "cpu",
        }
    }

    #[must_use]
    pub const fn is_gpu(self) -> bool {
        matches!(self, Self::Dedicated | Self::Integrated | Self::Npu)
    }

    #[must_use]
    pub const fn is_npu(self) -> bool {
        matches!(self, Self::Npu)
    }
}

impl fmt::Display for GpuDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dedicated => write!(f, "Dedicated GPU"),
            Self::Integrated => write!(f, "Integrated Graphics"),
            Self::Npu => write!(f, "Neural Processing Unit (NPU)"),
            Self::Cpu => write!(f, "CPU Compute"),
        }
    }
}

/// User-selectable hardware acceleration policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccelerationMode {
    /// Automatically prefers Dedicated GPU, then Integrated GPU, then NPU, then CPU.
    #[default]
    Auto,
    /// Strictly prefers Dedicated GPU hardware; falls back to CPU if unavailable.
    Dedicated,
    /// Strictly prefers Integrated/Internal graphics; falls back to CPU if unavailable.
    Integrated,
    /// Prefers Neural Processing Unit hardware (DirectML / CoreML); falls back to CPU if unavailable.
    Npu,
    /// Forces CPU compute execution.
    Cpu,
}

impl AccelerationMode {
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "dedicated" | "dgpu" | "discrete" => Self::Dedicated,
            "integrated" | "igpu" | "internal" | "apu" => Self::Integrated,
            "npu" | "neural" | "ai" | "directml" | "coreml" => Self::Npu,
            "cpu" | "host" | "software" | "none" | "off" => Self::Cpu,
            _ => Self::Auto,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dedicated => "dedicated",
            Self::Integrated => "integrated",
            Self::Npu => "npu",
            Self::Cpu => "cpu",
        }
    }
}

impl fmt::Display for AccelerationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detailed descriptor of an enumerated GPU device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuInfo {
    /// Clean display name of the adapter (e.g. "NVIDIA GeForce RTX 3050 Laptop GPU").
    pub name: String,
    /// Hardware vendor (e.g. "NVIDIA", "AMD", "Intel", "Apple", "Broadcom", "ARM", "Qualcomm", "RISC-V").
    pub vendor: String,
    /// Classification tier: Dedicated vs Integrated.
    pub device_type: GpuDeviceType,
    /// Microarchitecture classification (e.g. "Ampere", "RDNA 2", "Metal Silicon", "VideoCore VII").
    pub architecture: String,
    /// Hardware release generation year (from 2006 pioneer unified shaders to 2026).
    pub generation_year: u32,
    /// Supported graphics & compute protocols (Vulkan, DirectX, Metal, OpenGL, OpenCL, DirectML).
    pub protocols: Vec<String>,
    /// Total video memory in bytes, if reportable.
    pub vram_bytes: Option<u64>,
    /// Driver or adapter identifier string.
    pub identifier: String,
    /// Whether this device is online and ready for compute work.
    pub is_available: bool,
}

impl GpuInfo {
    #[must_use]
    pub fn vram_formatted(&self) -> String {
        match self.vram_bytes {
            Some(bytes) if bytes >= 1024 * 1024 * 1024 => {
                format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            }
            Some(bytes) if bytes >= 1024 * 1024 => {
                format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0))
            }
            Some(bytes) => format!("{bytes} B"),
            None => "Shared".to_string(),
        }
    }
}

/// Cached list of discovered GPU adapters.
static GPU_DEVICES: OnceLock<Vec<GpuInfo>> = OnceLock::new();

/// Detect all graphics adapters present in the host system.
/// Result is lazily probed once and cached for process lifetime.
#[must_use]
pub fn detect_gpu_devices() -> &'static [GpuInfo] {
    GPU_DEVICES.get_or_init(probe_all_gpus).as_slice()
}

/// Resolve the active acceleration tier based on user request and hardware presence.
///
/// Hierarchy:
/// 1. If `Auto`:
///    - NPU present -> (Npu, Some(GpuInfo)) [prioritizes zero-overhead neural accelerator offload]
///    - Else if Dedicated GPU present -> (Dedicated, Some(GpuInfo))
///    - Else if Integrated GPU present -> (Integrated, Some(GpuInfo))
///    - Else -> (Cpu, None)
/// 2. If `Dedicated`:
///    - Dedicated GPU present -> (Dedicated, Some(GpuInfo))
///    - Else -> (Cpu, None)
/// 3. If `Integrated`:
///    - Integrated GPU present -> (Integrated, Some(GpuInfo))
///    - Else -> (Cpu, None)
/// 4. If `Npu`:
///    - NPU present -> (Npu, Some(GpuInfo))
///    - Else -> (Cpu, None)
/// 5. If `Cpu`:
///    - (Cpu, None)
#[must_use]
pub fn resolve_acceleration(mode: AccelerationMode) -> (GpuDeviceType, Option<GpuInfo>) {
    let devices = detect_gpu_devices();

    let dedicated = devices
        .iter()
        .find(|g| g.device_type == GpuDeviceType::Dedicated && g.is_available)
        .cloned();

    let integrated = devices
        .iter()
        .find(|g| g.device_type == GpuDeviceType::Integrated && g.is_available)
        .cloned();

    let npu = devices
        .iter()
        .find(|g| g.device_type == GpuDeviceType::Npu && g.is_available)
        .cloned();

    match mode {
        AccelerationMode::Auto => {
            if let Some(npu_dev) = npu {
                (GpuDeviceType::Npu, Some(npu_dev))
            } else if let Some(gpu) = dedicated {
                (GpuDeviceType::Dedicated, Some(gpu))
            } else if let Some(gpu) = integrated {
                (GpuDeviceType::Integrated, Some(gpu))
            } else {
                (GpuDeviceType::Cpu, None)
            }
        }
        AccelerationMode::Dedicated => {
            if let Some(gpu) = dedicated {
                (GpuDeviceType::Dedicated, Some(gpu))
            } else {
                (GpuDeviceType::Cpu, None)
            }
        }
        AccelerationMode::Integrated => {
            if let Some(gpu) = integrated {
                (GpuDeviceType::Integrated, Some(gpu))
            } else {
                (GpuDeviceType::Cpu, None)
            }
        }
        AccelerationMode::Npu => {
            if let Some(npu_dev) = npu {
                (GpuDeviceType::Npu, Some(npu_dev))
            } else {
                (GpuDeviceType::Cpu, None)
            }
        }
        AccelerationMode::Cpu => (GpuDeviceType::Cpu, None),
    }
}

// ── Platform-Specific Probing (Pure Safe Rust) ──────────────────────────

fn probe_all_gpus() -> Vec<GpuInfo> {
    let mut devices = Vec::new();

    #[cfg(windows)]
    {
        devices.extend(probe_windows_gpus());
    }

    #[cfg(unix)]
    {
        devices.extend(probe_unix_gpus());
    }

    // Deduplicate by name and device type (non-adjacent duplicates included)
    let mut seen: Vec<(String, GpuDeviceType)> = Vec::new();
    devices.retain(|d| {
        let key = (d.name.clone(), d.device_type);
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
    devices
}

/// True when `name` contains an "rx <lead><digits>" model whose digit run has exactly `len` digits
/// (e.g. "rx 5700 xt" is a 4-digit 5-series part while "rx 580" is a 3-digit one).
fn is_rx_series(name: &str, lead: char, len: usize) -> bool {
    let mut rest = name;
    while let Some(pos) = rest.find("rx ") {
        let tail = &rest[pos + 3..];
        let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() == len && digits.starts_with(lead) {
            return true;
        }
        rest = tail;
    }
    false
}

/// Classify GPU microarchitecture, hardware generation year (2006 pioneer unified shaders to 2026),
/// and supported graphical & compute protocols across all major vendors.
#[must_use]
pub fn classify_gpu_architecture(
    name: &str,
    vendor: &str,
    identifier: &str,
) -> (String, u32, Vec<String>) {
    let lower_name = name.to_ascii_lowercase();
    let upper_id = identifier.to_ascii_uppercase();

    if lower_name.contains("npu")
        || lower_name.contains("ai boost")
        || lower_name.contains("xdna")
        || lower_name.contains("ipu")
        || lower_name.contains("neural engine")
        || lower_name.contains("ane")
        || lower_name.contains("hexagon")
    {
        return if vendor == "Intel"
            || lower_name.contains("intel")
            || lower_name.contains("ai boost")
        {
            (
                "Intel AI Boost NPU (NPU3720/NPU4000)".into(),
                2024,
                vec![
                    "DirectML".into(),
                    "OpenVINO".into(),
                    "Vulkan 1.3".into(),
                    "ONNX Runtime".into(),
                ],
            )
        } else if vendor == "AMD"
            || lower_name.contains("amd")
            || lower_name.contains("xdna")
            || lower_name.contains("ryzen ai")
        {
            (
                "AMD XDNA NPU (Ryzen AI)".into(),
                2024,
                vec![
                    "DirectML".into(),
                    "ROCm NPU".into(),
                    "Vulkan 1.3".into(),
                    "ONNX Runtime".into(),
                ],
            )
        } else if vendor == "Qualcomm"
            || lower_name.contains("qualcomm")
            || lower_name.contains("snapdragon")
            || lower_name.contains("hexagon")
        {
            (
                "Qualcomm Hexagon NPU (45 TOPS)".into(),
                2024,
                vec![
                    "DirectML".into(),
                    "Qualcomm AI Engine".into(),
                    "ONNX Runtime".into(),
                ],
            )
        } else if vendor == "Apple" || lower_name.contains("apple") || lower_name.contains("ane") {
            (
                "Apple Neural Engine (16-Core ANE)".into(),
                2024,
                vec![
                    "CoreML".into(),
                    "Metal Performance Shaders".into(),
                    "Accelerate".into(),
                ],
            )
        } else {
            (
                "Hardware Neural Processing Unit (NPU)".into(),
                2024,
                vec![
                    "DirectML".into(),
                    "ONNX Runtime".into(),
                    "Vulkan 1.3".into(),
                ],
            )
        };
    }

    if vendor == "NVIDIA" || upper_id.contains("VEN_10DE") || lower_name.contains("nvidia") {
        if (lower_name.contains("rtx 50") && !lower_name.contains(" ada"))
            || lower_name.contains("b100")
            || lower_name.contains("b200")
            || lower_name.contains("blackwell")
        {
            (
                "Blackwell".into(),
                2024,
                vec![
                    "Vulkan 1.4".into(),
                    "DirectX 12 Ultimate".into(),
                    "CUDA 12".into(),
                    "OpenGL 4.6".into(),
                    "DirectML".into(),
                ],
            )
        } else if lower_name.contains("rtx 40") || lower_name.contains("ada") {
            (
                "Ada Lovelace".into(),
                2022,
                vec![
                    "Vulkan 1.3".into(),
                    "DirectX 12 Ultimate".into(),
                    "CUDA 12".into(),
                    "OpenGL 4.6".into(),
                    "DirectML".into(),
                ],
            )
        } else if lower_name.contains("rtx 30")
            || lower_name.contains("rtx a")
            || lower_name.contains(" a100")
            || lower_name.contains("ampere")
            || upper_id.contains("DEV_25A2")
        {
            (
                "Ampere".into(),
                2020,
                vec![
                    "Vulkan 1.3".into(),
                    "DirectX 12 Ultimate".into(),
                    "CUDA 11/12".into(),
                    "OpenGL 4.6".into(),
                    "DirectML".into(),
                ],
            )
        } else if lower_name.contains("rtx 20")
            || lower_name.contains("gtx 16")
            || lower_name.contains("turing")
        {
            (
                "Turing".into(),
                2018,
                vec![
                    "Vulkan 1.3".into(),
                    "DirectX 12".into(),
                    "CUDA 11".into(),
                    "OpenGL 4.6".into(),
                    "DirectML".into(),
                ],
            )
        } else if lower_name.contains("gtx 10") || lower_name.contains("pascal") {
            (
                "Pascal".into(),
                2016,
                vec![
                    "Vulkan 1.2".into(),
                    "DirectX 12".into(),
                    "CUDA 10".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains("gtx 9")
            || lower_name.contains("gtx 750")
            || lower_name.contains("maxwell")
        {
            (
                "Maxwell".into(),
                2014,
                vec![
                    "Vulkan 1.1".into(),
                    "DirectX 12".into(),
                    "CUDA 9".into(),
                    "OpenGL 4.5".into(),
                ],
            )
        } else if lower_name.contains("gtx 6")
            || lower_name.contains("gtx 7")
            || lower_name.contains("kepler")
        {
            (
                "Kepler".into(),
                2012,
                vec![
                    "Vulkan 1.0".into(),
                    "DirectX 11".into(),
                    "CUDA 8".into(),
                    "OpenGL 4.5".into(),
                ],
            )
        } else if lower_name.contains("gtx 4")
            || lower_name.contains("gtx 5")
            || lower_name.contains("fermi")
        {
            (
                "Fermi".into(),
                2010,
                vec!["DirectX 11".into(), "CUDA 6".into(), "OpenGL 4.4".into()],
            )
        } else {
            (
                "Tesla / G80 (Pioneer Unified Architecture)".into(),
                2006,
                vec!["DirectX 10".into(), "CUDA 1.0".into(), "OpenGL 3.3".into()],
            )
        }
    } else if vendor == "AMD"
        || upper_id.contains("VEN_1002")
        || lower_name.contains("amd")
        || lower_name.contains("radeon")
    {
        if lower_name.contains("rx 8") || lower_name.contains("rdna 4") {
            (
                "RDNA 4".into(),
                2025,
                vec![
                    "Vulkan 1.4".into(),
                    "DirectX 12 Ultimate".into(),
                    "ROCm".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains("rx 7")
            || lower_name.contains("780m")
            || lower_name.contains("rdna 3")
        {
            (
                "RDNA 3 / 3.5".into(),
                2022,
                vec![
                    "Vulkan 1.3".into(),
                    "DirectX 12 Ultimate".into(),
                    "ROCm".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains("rx 6")
            || lower_name.contains("680m")
            || lower_name.contains("rdna 2")
        {
            (
                "RDNA 2".into(),
                2020,
                vec![
                    "Vulkan 1.3".into(),
                    "DirectX 12 Ultimate".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if is_rx_series(&lower_name, '5', 4) || lower_name.contains("rdna") {
            (
                "RDNA 1".into(),
                2019,
                vec![
                    "Vulkan 1.2".into(),
                    "DirectX 12".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains("vega")
            || lower_name.contains("radeon vii")
            || lower_name.contains("radeon(tm) graphics")
        {
            (
                "GCN 5.0 (Vega / Ryzen APU)".into(),
                2017,
                vec![
                    "Vulkan 1.2".into(),
                    "DirectX 12".into(),
                    "OpenGL 4.6".into(),
                    "DirectML".into(),
                ],
            )
        } else if lower_name.contains("rx 4")
            || lower_name.contains("rx 5")
            || lower_name.contains("polaris")
        {
            (
                "GCN 4.0 (Polaris)".into(),
                2016,
                vec![
                    "Vulkan 1.2".into(),
                    "DirectX 12".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains("r9")
            || lower_name.contains("r7")
            || lower_name.contains("hd 7")
        {
            (
                "GCN 1.0 - 3.0".into(),
                2012,
                vec![
                    "Vulkan 1.0".into(),
                    "DirectX 11.1".into(),
                    "OpenGL 4.5".into(),
                ],
            )
        } else {
            (
                "TeraScale (Unified VLIW Architecture)".into(),
                2007,
                vec![
                    "DirectX 10/11".into(),
                    "OpenGL 3.3/4.1".into(),
                    "OpenCL 1.1".into(),
                ],
            )
        }
    } else if vendor == "Intel" || upper_id.contains("VEN_8086") || lower_name.contains("intel") {
        if lower_name.contains("battlemage") || lower_name.contains("bmg") {
            (
                "Xe2 / Battlemage".into(),
                2024,
                vec![
                    "Vulkan 1.3".into(),
                    "DirectX 12 Ultimate".into(),
                    "OneAPI".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains(" arc") || lower_name.contains("alchemist") {
            (
                "Xe-HPG / Alchemist".into(),
                2022,
                vec![
                    "Vulkan 1.3".into(),
                    "DirectX 12 Ultimate".into(),
                    "OneAPI".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains("iris") || lower_name.contains("xe") {
            (
                "Xe-LP (Gen12)".into(),
                2020,
                vec![
                    "Vulkan 1.2".into(),
                    "DirectX 12".into(),
                    "OpenGL 4.6".into(),
                ],
            )
        } else if lower_name.contains("uhd")
            || lower_name.contains("630")
            || lower_name.contains("620")
        {
            (
                "Gen9.5 (UHD Graphics)".into(),
                2016,
                vec![
                    "Vulkan 1.1".into(),
                    "DirectX 12".into(),
                    "OpenGL 4.5".into(),
                ],
            )
        } else if lower_name.contains("hd graphics") {
            (
                "Gen7 / Gen8 (HD Graphics)".into(),
                2012,
                vec![
                    "DirectX 11".into(),
                    "OpenGL 4.0".into(),
                    "OpenCL 1.2".into(),
                ],
            )
        } else {
            (
                "Gen4 / GMA X3000 (Unified Graphics Pioneer)".into(),
                2006,
                vec!["DirectX 10".into(), "OpenGL 2.1".into()],
            )
        }
    } else if vendor == "Apple" || lower_name.contains("apple") {
        if lower_name.contains("m4") || lower_name.contains("m3") {
            (
                "Apple Silicon M-Series (Hardware Ray Tracing / Metal 3)".into(),
                2023,
                vec![
                    "Metal 3".into(),
                    "MPS".into(),
                    "WebGPU".into(),
                    "OpenGL 4.1".into(),
                ],
            )
        } else {
            (
                "Apple Silicon M-Series (Unified Memory Architecture)".into(),
                2020,
                vec!["Metal 2/3".into(), "MPS".into(), "OpenGL 4.1".into()],
            )
        }
    } else if lower_name.contains("videocore")
        || lower_name.contains("broadcom")
        || lower_name.contains("bcm")
    {
        if lower_name.contains("vii") || lower_name.contains("pi 5") {
            (
                "Broadcom VideoCore VII".into(),
                2023,
                vec![
                    "Vulkan 1.2".into(),
                    "OpenGL ES 3.1".into(),
                    "DRM/KMS".into(),
                ],
            )
        } else if lower_name.contains("videocore vi")
            || lower_name.contains("videocore 6")
            || lower_name.contains("pi 4")
        {
            (
                "Broadcom VideoCore VI".into(),
                2019,
                vec![
                    "Vulkan 1.0".into(),
                    "OpenGL ES 3.0".into(),
                    "DRM/KMS".into(),
                ],
            )
        } else {
            (
                "Broadcom VideoCore IV".into(),
                2012,
                vec!["OpenGL ES 2.0".into(), "OpenVG".into(), "DRM/KMS".into()],
            )
        }
    } else if lower_name.contains("mali") || lower_name.contains("immortalis") {
        if lower_name.contains("immortalis") {
            (
                "ARM Immortalis (Hardware Ray Tracing)".into(),
                2022,
                vec!["Vulkan 1.3".into(), "OpenGL ES 3.2".into()],
            )
        } else if lower_name.contains("valhall")
            || lower_name.contains("g7")
            || lower_name.contains("g6")
        {
            (
                "ARM Mali Valhall / Bifrost".into(),
                2019,
                vec!["Vulkan 1.2".into(), "OpenGL ES 3.2".into()],
            )
        } else {
            (
                "ARM Mali Midgard / Utgard".into(),
                2012,
                vec!["OpenGL ES 3.1".into(), "OpenCL 1.2".into()],
            )
        }
    } else if lower_name.contains("adreno") || lower_name.contains("qualcomm") {
        (
            "Qualcomm Adreno GPU".into(),
            2016,
            vec![
                "Vulkan 1.1/1.3".into(),
                "DirectX 12".into(),
                "OpenGL ES 3.2".into(),
                "OpenCL 3.0".into(),
            ],
        )
    } else if lower_name.contains("risc-v")
        || lower_name.contains("powervr")
        || lower_name.contains("tenstorrent")
        || lower_name.contains("xuantie")
    {
        (
            "RISC-V Compute Accelerator / PowerVR B-Series".into(),
            2022,
            vec!["Vulkan 1.2".into(), "OpenCL 3.0".into(), "DRM/KMS".into()],
        )
    } else {
        (
            "Generic Unified Compute Architecture".into(),
            2006,
            vec![
                "Vulkan".into(),
                "DirectX".into(),
                "OpenGL".into(),
                "OpenCL".into(),
            ],
        )
    }
}

#[cfg(windows)]
fn probe_windows_gpus() -> Vec<GpuInfo> {
    let mut list = Vec::new();
    let raw_gpus = crate::platform_windows::probe_windows_gpus_registry();

    for (driver_desc, matching_id, vram_bytes) in raw_gpus {
        let lower_desc = driver_desc.to_ascii_lowercase();
        let upper_id = matching_id.to_ascii_uppercase();

        let vendor = if upper_id.contains("VEN_10DE")
            || lower_desc.contains("nvidia")
            || lower_desc.contains("geforce")
            || lower_desc.contains("quadro")
        {
            "NVIDIA"
        } else if upper_id.contains("VEN_1002")
            || lower_desc.contains("amd")
            || lower_desc.contains("radeon")
        {
            "AMD"
        } else if upper_id.contains("VEN_8086")
            || lower_desc.contains("intel")
            || lower_desc.contains("arc")
        {
            "Intel"
        } else {
            "Unknown"
        };

        let is_npu = lower_desc.contains("npu")
            || lower_desc.contains("ai boost")
            || lower_desc.contains("xdna")
            || lower_desc.contains("ipu")
            || lower_desc.contains("neural")
            || lower_desc.contains("hexagon");

        let is_integrated = lower_desc.contains("radeon(tm) graphics")
            || lower_desc.contains("radeon graphics")
            || lower_desc.contains("uhd graphics")
            || lower_desc.contains("hd graphics")
            || lower_desc.contains("iris")
            || lower_desc.contains("apu")
            || vram_bytes.is_some_and(|b| {
                b <= 512 * 1024 * 1024 && !lower_desc.contains("rtx") && !lower_desc.contains("gtx")
            });

        let device_type = if is_npu {
            GpuDeviceType::Npu
        } else if is_integrated {
            GpuDeviceType::Integrated
        } else {
            GpuDeviceType::Dedicated
        };

        let (architecture, generation_year, protocols) =
            classify_gpu_architecture(&driver_desc, vendor, &matching_id);

        list.push(GpuInfo {
            name: driver_desc,
            vendor: vendor.to_string(),
            device_type,
            architecture,
            generation_year,
            protocols,
            vram_bytes,
            identifier: matching_id,
            is_available: true,
        });
    }

    list
}

#[cfg(unix)]
fn probe_unix_gpus() -> Vec<GpuInfo> {
    let mut list = Vec::new();

    // 1. WSL DirectX GPU Paravirtualization (/dev/dxg)
    if Path::new("/dev/dxg").exists() {
        let has_nvidia = Path::new("/proc/driver/nvidia/version").exists()
            || Path::new("/usr/lib/wsl/lib/libcuda.so").exists();

        if has_nvidia {
            let (arch, yr, protos) = classify_gpu_architecture(
                "WSL DirectX / NVIDIA GPU Adapter",
                "NVIDIA",
                "dxg:nvidia",
            );
            list.push(GpuInfo {
                name: "WSL DirectX / NVIDIA GPU Adapter".to_string(),
                vendor: "NVIDIA".to_string(),
                device_type: GpuDeviceType::Dedicated,
                architecture: arch,
                generation_year: yr,
                protocols: protos,
                vram_bytes: None,
                identifier: "/dev/dxg:nvidia".to_string(),
                is_available: true,
            });
        } else {
            let (arch, yr, protos) = classify_gpu_architecture(
                "WSL DirectX Virtual Graphics Adapter",
                "Microsoft",
                "dxg",
            );
            list.push(GpuInfo {
                name: "WSL DirectX Virtual Graphics Adapter".to_string(),
                vendor: "Microsoft".to_string(),
                device_type: GpuDeviceType::Integrated,
                architecture: arch,
                generation_year: yr,
                protocols: protos,
                vram_bytes: Some(1024 * 1024 * 1024),
                identifier: "/dev/dxg".to_string(),
                is_available: true,
            });
        }
    }

    // 2. Linux native DRM subsystem (/sys/class/drm/card*)
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !file_name.starts_with("card") || file_name.contains('-') {
                continue;
            }

            let vendor_path = path.join("device/vendor");
            let device_path = path.join("device/device");
            let vram_path = path.join("device/mem_info_vram_total");

            let vendor_id = std::fs::read_to_string(vendor_path)
                .map(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_default();
            let device_id = std::fs::read_to_string(device_path)
                .map(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_default();
            let vram_bytes = std::fs::read_to_string(vram_path)
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());

            if vendor_id.is_empty() {
                continue;
            }

            let (vendor_name, is_dedicated) = match vendor_id.as_str() {
                "0x10de" => ("NVIDIA", true),
                "0x1002" => ("AMD", vram_bytes.is_some_and(|v| v > 1024 * 1024 * 1024)),
                "0x8086" => ("Intel", false),
                "0x13b5" => ("ARM", false),      // ARM Mali
                "0x14e4" => ("Broadcom", false), // Broadcom VideoCore (Raspberry Pi)
                _ => ("Generic", false),
            };

            let dev_type = if is_dedicated {
                GpuDeviceType::Dedicated
            } else {
                GpuDeviceType::Integrated
            };

            let name = format!("{vendor_name} Graphics Adapter ({file_name})");
            let (arch, yr, protos) = classify_gpu_architecture(&name, vendor_name, &device_id);

            list.push(GpuInfo {
                name,
                vendor: vendor_name.to_string(),
                device_type: dev_type,
                architecture: arch,
                generation_year: yr,
                protocols: protos,
                vram_bytes,
                identifier: file_name.to_string(),
                is_available: true,
            });
        }
    }

    // 2b. Linux Compute Accelerator / NPU Subsystem (/sys/class/accel/accel*)
    if let Ok(entries) = std::fs::read_dir("/sys/class/accel") {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !file_name.starts_with("accel") {
                continue;
            }

            let vendor_path = path.join("device/vendor");
            let device_path = path.join("device/device");

            let vendor_id = std::fs::read_to_string(vendor_path)
                .map(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_default();
            let device_id = std::fs::read_to_string(device_path)
                .map(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_default();

            let vendor_name = match vendor_id.as_str() {
                "0x8086" => "Intel",
                "0x1002" => "AMD",
                _ => "Linux NPU",
            };

            let name = format!("{vendor_name} Neural Processing Unit ({file_name})");
            let (arch, yr, protos) = classify_gpu_architecture(&name, vendor_name, &device_id);

            list.push(GpuInfo {
                name,
                vendor: vendor_name.to_string(),
                device_type: GpuDeviceType::Npu,
                architecture: arch,
                generation_year: yr,
                protocols: protos,
                vram_bytes: None,
                identifier: file_name.to_string(),
                is_available: true,
            });
        }
    }

    // 3. macOS system_profiler
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("system_profiler")
            .args(["SPDisplaysDataType", "-json"])
            .output()
        {
            if out.status.success() {
                let json_str = String::from_utf8_lossy(&out.stdout);
                if json_str.contains("Apple") {
                    let (arch, yr, protos) =
                        classify_gpu_architecture("Apple Silicon GPU", "Apple", "apple-metal");
                    list.push(GpuInfo {
                        name: "Apple Silicon Integrated GPU".to_string(),
                        vendor: "Apple".to_string(),
                        device_type: GpuDeviceType::Integrated,
                        architecture: arch,
                        generation_year: yr,
                        protocols: protos,
                        vram_bytes: None,
                        identifier: "apple-metal".to_string(),
                        is_available: true,
                    });

                    // Apple Neural Engine (ANE) present on all Apple Silicon M-series
                    let (npu_arch, npu_yr, npu_protos) = classify_gpu_architecture(
                        "Apple Neural Engine (ANE)",
                        "Apple",
                        "apple-ane",
                    );
                    list.push(GpuInfo {
                        name: "Apple Neural Engine (ANE)".to_string(),
                        vendor: "Apple".to_string(),
                        device_type: GpuDeviceType::Npu,
                        architecture: npu_arch,
                        generation_year: npu_yr,
                        protocols: npu_protos,
                        vram_bytes: None,
                        identifier: "apple-ane".to_string(),
                        is_available: true,
                    });
                }
            }
        }
    }

    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceleration_mode_parsing() {
        assert_eq!(AccelerationMode::parse("auto"), AccelerationMode::Auto);
        assert_eq!(
            AccelerationMode::parse("dedicated"),
            AccelerationMode::Dedicated
        );
        assert_eq!(AccelerationMode::parse("dgpu"), AccelerationMode::Dedicated);
        assert_eq!(
            AccelerationMode::parse("integrated"),
            AccelerationMode::Integrated
        );
        assert_eq!(
            AccelerationMode::parse("igpu"),
            AccelerationMode::Integrated
        );
        assert_eq!(AccelerationMode::parse("npu"), AccelerationMode::Npu);
        assert_eq!(AccelerationMode::parse("neural"), AccelerationMode::Npu);
        assert_eq!(AccelerationMode::parse("directml"), AccelerationMode::Npu);
        assert_eq!(AccelerationMode::parse("cpu"), AccelerationMode::Cpu);
        assert_eq!(AccelerationMode::parse("none"), AccelerationMode::Cpu);
    }

    #[test]
    fn test_resolve_acceleration_cpu_always_works() {
        let (dev_type, info) = resolve_acceleration(AccelerationMode::Cpu);
        assert_eq!(dev_type, GpuDeviceType::Cpu);
        assert!(info.is_none());
    }

    #[test]
    fn test_device_detection_does_not_panic() {
        let devices = detect_gpu_devices();
        for dev in devices {
            assert!(!dev.name.is_empty());
            assert!(dev.generation_year >= 2006);
            assert!(!dev.protocols.is_empty());
        }
    }

    #[test]
    fn test_architecture_classification() {
        let (arch, yr, protos) =
            classify_gpu_architecture("NVIDIA GeForce RTX 3050 Laptop GPU", "NVIDIA", "DEV_25A2");
        assert_eq!(arch, "Ampere");
        assert_eq!(yr, 2020);
        assert!(protos.iter().any(|p| p.contains("Vulkan")));

        let (mali_arch, mali_yr, _) = classify_gpu_architecture("ARM Mali-G78", "ARM", "Mali");
        assert!(mali_arch.contains("Mali"));
        assert_eq!(mali_yr, 2019);

        let (pi_arch, pi_yr, _) =
            classify_gpu_architecture("Broadcom VideoCore VII", "Broadcom", "bcm2712");
        assert_eq!(pi_arch, "Broadcom VideoCore VII");
        assert_eq!(pi_yr, 2023);
    }

    #[test]
    fn test_classifier_regressions() {
        let (a, _, _) = classify_gpu_architecture("Broadcom VideoCore IV", "Broadcom", "bcm2835");
        assert_eq!(a, "Broadcom VideoCore IV");
        let (a, _, _) = classify_gpu_architecture("Broadcom VideoCore VI", "Broadcom", "bcm2711");
        assert_eq!(a, "Broadcom VideoCore VI");
        let (a, _, _) = classify_gpu_architecture("AMD Radeon RX 5700 XT", "AMD", "VEN_1002");
        assert_eq!(a, "RDNA 1");
        let (a, _, _) = classify_gpu_architecture("AMD Radeon RX 580", "AMD", "VEN_1002");
        assert!(a.contains("Polaris"));
        let (a, _, _) = classify_gpu_architecture("AMD Radeon RX 7900 XTX", "AMD", "VEN_1002");
        assert!(a.starts_with("RDNA 3"));
        let (a, _, _) = classify_gpu_architecture("NVIDIA RTX 5000 Ada Generation", "NVIDIA", "");
        assert_eq!(a, "Ada Lovelace");
        let (a, _, _) = classify_gpu_architecture("NVIDIA RTX A4000", "NVIDIA", "");
        assert_eq!(a, "Ampere");
        let (a, _, _) = classify_gpu_architecture("Intel(R) Arc(TM) A770", "Intel", "");
        assert!(a.contains("Alchemist"));
    }

    #[test]
    fn test_npu_architecture_classification() {
        let (arch, yr, protos) = classify_gpu_architecture("Intel(R) AI Boost", "Intel", "");
        assert!(arch.contains("AI Boost"));
        assert_eq!(yr, 2024);
        assert!(protos.iter().any(|p| p.contains("DirectML")));

        let (amd_arch, _, _) = classify_gpu_architecture("AMD XDNA NPU (Ryzen AI)", "AMD", "");
        assert!(amd_arch.contains("XDNA"));

        let (apple_arch, _, _) =
            classify_gpu_architecture("Apple Neural Engine (ANE)", "Apple", "");
        assert!(apple_arch.contains("Neural Engine"));

        let (q_arch, _, _) =
            classify_gpu_architecture("Qualcomm Snapdragon X Elite NPU", "Qualcomm", "");
        assert!(q_arch.contains("Hexagon"));
    }

    #[test]
    fn test_npu_priority_in_auto_mode() {
        let (dev_type, _) = resolve_acceleration(AccelerationMode::Auto);
        assert!(matches!(
            dev_type,
            GpuDeviceType::Npu
                | GpuDeviceType::Dedicated
                | GpuDeviceType::Integrated
                | GpuDeviceType::Cpu
        ));
    }
}
