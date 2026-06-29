use sysinfo::{Components, Cpu, CpuRefreshKind, Networks, RefreshKind, System};

#[derive(Debug)]
pub struct SystemMetrics {
    sys: System,
    networks: Networks,
    components: Components,
}

impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            sys: System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                    .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
            ),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.networks.refresh(true);
        self.components.refresh(true);
    }

    /// CPU usage as 0.0–1.0
    pub fn cpu_usage(&self) -> f32 {
        let cpus = self.sys.cpus();
        if cpus.is_empty() {
            return 0.0;
        }
        let total: f32 = cpus.iter().map(|c: &Cpu| c.cpu_usage()).sum();
        (total / cpus.len() as f32 / 100.0).clamp(0.0, 1.0)
    }

    /// Memory usage as 0.0–1.0
    pub fn memory_usage(&self) -> f32 {
        let total = self.sys.total_memory();
        if total == 0 {
            return 0.0;
        }
        (self.sys.used_memory() as f32 / total as f32).clamp(0.0, 1.0)
    }

    /// Network bytes/sec (delta since last refresh)
    pub fn network_bytes_per_sec(&self) -> u64 {
        self.networks
            .iter()
            .map(|(_, data)| data.total_transmitted() + data.total_received())
            .sum()
    }

    /// Map CPU usage to ember intensity (0.0–1.0)
    pub fn ember_intensity(&self) -> f32 {
        self.cpu_usage()
    }

    /// Map memory usage to wave amplitude (0.0–1.0)
    pub fn wave_amplitude(&self) -> f32 {
        self.memory_usage()
    }

    /// Map network to fall speed multiplier (1.0 at idle, 5.0 at high traffic)
    pub fn network_speed_multiplier(&self) -> f32 {
        let bps = self.network_bytes_per_sec() as f32;
        (1.0 + (bps / 1_000_000.0) * 4.0).clamp(1.0, 5.0)
    }

    /// Temperature readings (empty on most systems)
    pub fn temperatures(&self) -> Vec<(String, f32)> {
        self.components
            .iter()
            .filter_map(|c| c.temperature().map(|t| (c.label().to_string(), t)))
            .collect()
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_initializes() {
        let m = SystemMetrics::new();
        assert!(m.cpu_usage() >= 0.0);
        assert!(m.cpu_usage() <= 1.0);
    }

    #[test]
    fn refresh_does_not_panic() {
        let mut m = SystemMetrics::new();
        m.refresh();
        assert!(m.memory_usage() >= 0.0);
    }

    #[test]
    fn ember_intensity_range() {
        let m = SystemMetrics::new();
        let i = m.ember_intensity();
        assert!(i >= 0.0 && i <= 1.0);
    }

    #[test]
    fn network_speed_multiplier_minimum() {
        let m = SystemMetrics::new();
        assert!(m.network_speed_multiplier() >= 1.0);
    }
}
