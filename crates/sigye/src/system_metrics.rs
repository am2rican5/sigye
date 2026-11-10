//! System resource monitoring for reactive backgrounds.

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use sigye_core::SystemMetrics;
use sysinfo::{Networks, System};

/// Shared state for tracking max observed values (for normalization).
#[derive(Debug, Default)]
struct MaxValues {
    network_rx: u64,
    network_tx: u64,
}

/// System monitor that polls resource usage in a background thread.
#[derive(Debug)]
pub struct SystemMonitor {
    /// Shared metrics updated by the background thread.
    metrics: Arc<RwLock<SystemMetrics>>,
    /// Flag to signal thread termination.
    running: Arc<RwLock<bool>>,
}

impl SystemMonitor {
    /// Create a new system monitor.
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the background monitoring thread.
    pub fn start(&self) {
        // Set running flag
        if let Ok(mut running) = self.running.write() {
            if *running {
                return; // Already running
            }
            *running = true;
        }

        let metrics = self.metrics.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            // sysinfo 0.40 constructors return Result; if refresh setup
            // fails, leave metrics at their defaults and exit the thread.
            let Ok(mut sys) = System::new_all() else {
                return;
            };
            let Ok(mut networks) = Networks::new_with_refreshed_list() else {
                return;
            };
            let mut max_values = MaxValues::default();

            // Initial refresh to get baseline
            sys.refresh_all();
            thread::sleep(Duration::from_millis(500));

            // Track previous network bytes for rate calculation
            let mut prev_rx: u64 = networks.values().map(|n| n.received()).sum();
            let mut prev_tx: u64 = networks.values().map(|n| n.transmitted()).sum();
            let mut prev_time = Instant::now();

            loop {
                // Check if we should stop
                if let Ok(is_running) = running.read()
                    && !*is_running
                {
                    break;
                }

                // Refresh system info
                sys.refresh_cpu_all();
                sys.refresh_memory();
                networks.refresh(true);

                let now = Instant::now();
                let elapsed_secs = now.duration_since(prev_time).as_secs_f64().max(0.001);

                // Calculate CPU usage (average across all cores)
                let cpu_usage = sys.cpus().iter().map(|c| c.usage()).sum::<f32>()
                    / sys.cpus().len().max(1) as f32
                    / 100.0;

                // Calculate memory usage
                let memory_usage = if sys.total_memory() > 0 {
                    sys.used_memory() as f32 / sys.total_memory() as f32
                } else {
                    0.0
                };

                // Calculate network rates
                let current_rx: u64 = networks.values().map(|n| n.received()).sum();
                let current_tx: u64 = networks.values().map(|n| n.transmitted()).sum();

                let rx_bytes_per_sec =
                    (current_rx.saturating_sub(prev_rx) as f64 / elapsed_secs) as u64;
                let tx_bytes_per_sec =
                    (current_tx.saturating_sub(prev_tx) as f64 / elapsed_secs) as u64;

                // Update max values for normalization (with minimum threshold)
                const MIN_NETWORK_RATE: u64 = 1_000_000; // 1 MB/s minimum scale
                max_values.network_rx = max_values
                    .network_rx
                    .max(rx_bytes_per_sec)
                    .max(MIN_NETWORK_RATE);
                max_values.network_tx = max_values
                    .network_tx
                    .max(tx_bytes_per_sec)
                    .max(MIN_NETWORK_RATE);

                let network_rx_rate = rx_bytes_per_sec as f32 / max_values.network_rx as f32;
                let network_tx_rate = tx_bytes_per_sec as f32 / max_values.network_tx as f32;

                prev_rx = current_rx;
                prev_tx = current_tx;
                prev_time = now;

                // Update metrics
                let new_metrics = SystemMetrics {
                    cpu_usage: cpu_usage.clamp(0.0, 1.0),
                    memory_usage: memory_usage.clamp(0.0, 1.0),
                    network_rx_rate: network_rx_rate.clamp(0.0, 1.0),
                    network_tx_rate: network_tx_rate.clamp(0.0, 1.0),
                };

                // Update shared metrics
                if let Ok(mut m) = metrics.write() {
                    *m = new_metrics;
                }

                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    /// Stop the background monitoring thread.
    pub fn stop(&self) {
        if let Ok(mut running) = self.running.write() {
            *running = false;
        }
    }

    /// Get the current system metrics.
    /// Non-blocking when possible; the writer holds the lock only briefly, so
    /// block rather than fabricate zeros if it is momentarily contended.
    pub fn get_metrics(&self) -> SystemMetrics {
        if let Ok(m) = self.metrics.try_read() {
            return m.clone();
        }
        if let Ok(m) = self.metrics.read() {
            return m.clone();
        }
        // Lock poisoned: last resort.
        SystemMetrics::default()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SystemMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_default() {
        let metrics = SystemMetrics::default();
        assert_eq!(metrics.cpu_usage, 0.0);
        assert_eq!(metrics.memory_usage, 0.0);
    }

    #[test]
    fn test_monitor_creation() {
        let monitor = SystemMonitor::new();
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.cpu_usage, 0.0);
    }
}
