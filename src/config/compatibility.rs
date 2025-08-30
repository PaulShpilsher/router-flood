//! Compatibility layer for transitioning from old Config to new ApplicationConfig
//!
//! This module provides conversion functions to maintain backward compatibility
//! during the configuration system refactoring.

use crate::config::{
    Config, TargetConfig, AttackConfig, SafetyConfig, MonitoringConfig, ExportConfig,
    ProtocolMix as OldProtocolMix, BurstPattern as OldBurstPattern, ExportFormat as OldExportFormat,
};
use crate::config::application::{
    ApplicationConfig, TargetSettings, ExecutionSettings, SafetySettings, ObservabilitySettings,
    MonitoringSettings, ExportSettings, ProtocolMix, BurstPattern, ExportFormat,
};

impl From<Config> for ApplicationConfig {
    fn from(old_config: Config) -> Self {
        Self {
            target: old_config.target.into(),
            execution: old_config.attack.into(),
            safety: old_config.safety.into(),
            observability: ObservabilitySettings {
                monitoring: old_config.monitoring.into(),
                export: old_config.export.into(),
            },
        }
    }
}

impl From<ApplicationConfig> for Config {
    fn from(new_config: ApplicationConfig) -> Self {
        Self {
            target: new_config.target.into(),
            attack: new_config.execution.into(),
            safety: new_config.safety.into(),
            monitoring: new_config.observability.monitoring.into(),
            export: new_config.observability.export.into(),
        }
    }
}

impl From<TargetConfig> for TargetSettings {
    fn from(old: TargetConfig) -> Self {
        Self {
            ip: old.ip,
            ports: old.ports,
            protocol_mix: old.protocol_mix.into(),
            interface: old.interface,
        }
    }
}

impl From<TargetSettings> for TargetConfig {
    fn from(new: TargetSettings) -> Self {
        Self {
            ip: new.ip,
            ports: new.ports,
            protocol_mix: new.protocol_mix.into(),
            interface: new.interface,
        }
    }
}

impl From<AttackConfig> for ExecutionSettings {
    fn from(old: AttackConfig) -> Self {
        Self {
            threads: old.threads,
            packet_rate: old.packet_rate,
            duration: old.duration,
            packet_size_range: old.packet_size_range,
            burst_pattern: old.burst_pattern.into(),
            randomize_timing: old.randomize_timing,
        }
    }
}

impl From<ExecutionSettings> for AttackConfig {
    fn from(new: ExecutionSettings) -> Self {
        Self {
            threads: new.threads,
            packet_rate: new.packet_rate,
            duration: new.duration,
            packet_size_range: new.packet_size_range,
            burst_pattern: new.burst_pattern.into(),
            randomize_timing: new.randomize_timing,
        }
    }
}

impl From<SafetyConfig> for SafetySettings {
    fn from(old: SafetyConfig) -> Self {
        Self {
            max_threads: old.max_threads,
            max_packet_rate: old.max_packet_rate,
            require_private_ranges: old.require_private_ranges,
            audit_logging: old.audit_logging,
            dry_run: old.dry_run,
            perfect_simulation: old.perfect_simulation,
        }
    }
}

impl From<SafetySettings> for SafetyConfig {
    fn from(new: SafetySettings) -> Self {
        Self {
            max_threads: new.max_threads,
            max_packet_rate: new.max_packet_rate,
            require_private_ranges: new.require_private_ranges,
            enable_monitoring: true, // Default value for backward compatibility
            audit_logging: new.audit_logging,
            dry_run: new.dry_run,
            perfect_simulation: new.perfect_simulation,
        }
    }
}

impl From<MonitoringConfig> for MonitoringSettings {
    fn from(old: MonitoringConfig) -> Self {
        Self {
            stats_interval: old.stats_interval,
            system_monitoring: old.system_monitoring,
            performance_tracking: old.performance_tracking,
        }
    }
}

impl From<MonitoringSettings> for MonitoringConfig {
    fn from(new: MonitoringSettings) -> Self {
        Self {
            stats_interval: new.stats_interval,
            system_monitoring: new.system_monitoring,
            export_interval: None, // Will be set from ExportSettings
            performance_tracking: new.performance_tracking,
        }
    }
}

impl From<ExportConfig> for ExportSettings {
    fn from(old: ExportConfig) -> Self {
        Self {
            enabled: old.enabled,
            format: old.format.into(),
            filename_pattern: old.filename_pattern,
            include_system_stats: old.include_system_stats,
            export_interval: None, // This was in MonitoringConfig in old structure
        }
    }
}

impl From<ExportSettings> for ExportConfig {
    fn from(new: ExportSettings) -> Self {
        Self {
            enabled: new.enabled,
            format: new.format.into(),
            filename_pattern: new.filename_pattern,
            include_system_stats: new.include_system_stats,
        }
    }
}

impl From<OldProtocolMix> for ProtocolMix {
    fn from(old: OldProtocolMix) -> Self {
        Self {
            udp_ratio: old.udp_ratio,
            tcp_syn_ratio: old.tcp_syn_ratio,
            tcp_ack_ratio: old.tcp_ack_ratio,
            icmp_ratio: old.icmp_ratio,
            ipv6_ratio: old.ipv6_ratio,
            arp_ratio: old.arp_ratio,
        }
    }
}

impl From<ProtocolMix> for OldProtocolMix {
    fn from(new: ProtocolMix) -> Self {
        Self {
            udp_ratio: new.udp_ratio,
            tcp_syn_ratio: new.tcp_syn_ratio,
            tcp_ack_ratio: new.tcp_ack_ratio,
            icmp_ratio: new.icmp_ratio,
            ipv6_ratio: new.ipv6_ratio,
            arp_ratio: new.arp_ratio,
        }
    }
}

impl From<OldBurstPattern> for BurstPattern {
    fn from(old: OldBurstPattern) -> Self {
        match old {
            OldBurstPattern::Sustained { rate } => BurstPattern::Sustained { rate },
            OldBurstPattern::Bursts { burst_size, burst_interval_ms } => {
                BurstPattern::Bursts { burst_size, burst_interval_ms }
            }
            OldBurstPattern::Ramp { start_rate, end_rate, ramp_duration } => {
                BurstPattern::Ramp { start_rate, end_rate, ramp_duration }
            }
        }
    }
}

impl From<BurstPattern> for OldBurstPattern {
    fn from(new: BurstPattern) -> Self {
        match new {
            BurstPattern::Sustained { rate } => OldBurstPattern::Sustained { rate },
            BurstPattern::Bursts { burst_size, burst_interval_ms } => {
                OldBurstPattern::Bursts { burst_size, burst_interval_ms }
            }
            BurstPattern::Ramp { start_rate, end_rate, ramp_duration } => {
                OldBurstPattern::Ramp { start_rate, end_rate, ramp_duration }
            }
        }
    }
}

impl From<OldExportFormat> for ExportFormat {
    fn from(old: OldExportFormat) -> Self {
        match old {
            OldExportFormat::Json => ExportFormat::Json,
            OldExportFormat::Csv => ExportFormat::Csv,
            OldExportFormat::Both => ExportFormat::Both,
        }
    }
}

impl From<ExportFormat> for OldExportFormat {
    fn from(new: ExportFormat) -> Self {
        match new {
            ExportFormat::Json => OldExportFormat::Json,
            ExportFormat::Csv => OldExportFormat::Csv,
            ExportFormat::Both => OldExportFormat::Both,
        }
    }
}

// Tests moved to tests/ directory
