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



fn setup_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn parse_arguments() -> clap::ArgMatches {
    Command::new("Router Flood - Enhanced Network Stress Tester")
        .version(env!("CARGO_PKG_VERSION"))
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
        .get_matches()
}

fn process_config(matches: &clap::ArgMatches) -> Result<Config, String> {
    let mut config = if let Some(config_path) = matches.get_one::<String>("config") {
        load_config(Some(config_path))?
    } else {
        get_default_config()
    };

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

    Ok(config)
}

fn setup_channels(
    dry_run: bool,
    selected_interface: &Option<pnet::datalink::NetworkInterface>,
) -> (
    Option<Arc<Mutex<pnet::transport::TransportSender>>>,
    Option<Arc<Mutex<pnet::transport::TransportSender>>>,
    Option<Arc<Mutex<Box<dyn pnet::datalink::DataLinkSender>>>>,
) {
    if !dry_run {
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
    }
}

pub async fn run() {
    setup_logging();
    let matches = parse_arguments();

    if handle_pre_execution_commands(&matches) {
        return;
    }

    let config = initialize_configuration(&matches);
    let target_ip = parse_and_validate_ip(&config.target.ip);

    perform_validations(&config, &target_ip).unwrap_or_else(|err| {
        error!("Validation failed: {}", err);
        std::process::exit(1);
    });

    let selected_interface = setup_network_interface(&config);

    let stats = Arc::new(FloodStats::new(
        config.export.enabled.then_some(config.export.clone()),
    ));

    setup_audit_log(&config, &target_ip, &selected_interface, &stats.session_id);

    run_simulation(config, target_ip, selected_interface, stats).await;
}

fn handle_pre_execution_commands(matches: &clap::ArgMatches) -> bool {
    if matches.get_flag("list-interfaces") {
        println!("Available network interfaces:");
        for iface in list_network_interfaces() {
            println!(
                "  {} - {} (Up: {}, IPs: {:?})",
                iface.name,
                iface.description,
                iface.is_up(),
                iface.ips
            );
        }
        return true;
    }
    false
}

fn initialize_configuration(matches: &clap::ArgMatches) -> Config {
    match process_config(matches) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to process config: {}", e);
            std::process::exit(1);
        }
    }
}

fn parse_and_validate_ip(ip_str: &str) -> IpAddr {
    ip_str.parse().expect("Invalid IP format")
}

fn perform_validations(config: &Config, target_ip: &IpAddr) -> Result<(), String> {
    validate_comprehensive_security(
        target_ip,
        &config.target.ports,
        config.attack.threads,
        config.attack.packet_rate,
    )?;
    validate_system_requirements(config.safety.dry_run)?;
    Ok(())
}

fn setup_network_interface(config: &Config) -> Option<pnet::datalink::NetworkInterface> {
    if let Some(iface_name) = &config.target.interface {
        match find_interface_by_name(iface_name) {
            Some(iface) => {
                info!("Using specified interface: {}", iface.name);
                Some(iface)
            }
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
            }
            None => {
                warn!("No suitable network interface found");
                None
            }
        }
    }
}

fn setup_audit_log(
    config: &Config,
    target_ip: &IpAddr,
    selected_interface: &Option<pnet::datalink::NetworkInterface>,
    session_id: &str,
) {
    if config.safety.audit_logging {
        if let Err(e) = create_audit_entry(
            target_ip,
            &config.target.ports,
            config.attack.threads,
            config.attack.packet_rate,
            config.attack.duration,
            selected_interface.as_ref().map(|i| i.name.as_str()),
            session_id,
        ) {
            error!("Audit Log Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_simulation(
    config: Config,
    target_ip: IpAddr,
    selected_interface: Option<pnet::datalink::NetworkInterface>,
    stats: Arc<FloodStats>,
) {
    let running = Arc::new(AtomicBool::new(true));
    let system_monitor = Arc::new(SystemMonitor::new(config.monitoring.system_monitoring));

    spawn_monitoring_tasks(
        &config,
        stats.clone(),
        system_monitor.clone(),
        running.clone(),
    );

    let (tx_ipv4, tx_ipv6, tx_l2) = setup_channels(config.safety.dry_run, &selected_interface);
    let multi_port_target = Arc::new(MultiPortTarget::new(config.target.ports.clone()));

    let handles = spawn_worker_threads(
        &config,
        stats.clone(),
        running.clone(),
        multi_port_target,
        target_ip,
        tx_ipv4,
        tx_ipv6,
        tx_l2,
        config.safety.dry_run,
    );

    print_simulation_start_info(&config, &target_ip, &selected_interface, &stats.session_id);

    // Graceful shutdown handling
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

    finalize_execution(&config, &stats).await;
}

fn print_simulation_start_info(
    config: &Config,
    target_ip: &IpAddr,
    selected_interface: &Option<pnet::datalink::NetworkInterface>,
    session_id: &str,
) {
    if config.safety.dry_run {
        info!("🔍 Starting Enhanced Router Flood SIMULATION v{} (DRY-RUN)", env!("CARGO_PKG_VERSION"));
        info!("   ⚠️  DRY-RUN MODE: No actual packets will be sent!");
    } else {
        info!("🚀 Starting Enhanced Router Flood Simulation v{}", env!("CARGO_PKG_VERSION"));
    }
    info!("   Session ID: {}", session_id);
    info!("   Target: {} (Ports: {:?})", target_ip, config.target.ports);
    info!("   Threads: {}, Rate: {} pps/thread", config.attack.threads, config.attack.packet_rate);
    if let Some(d) = config.attack.duration {
        info!("   Duration: {} seconds", d);
    }
    if let Some(iface) = selected_interface {
        info!("   Interface: {}", iface.name);
    }
    info!(
        "   Protocols: UDP({:.0}%), TCP-SYN({:.0}%), TCP-ACK({:.0}%), ICMP({:.0}%), IPv6({:.0}%), ARP({:.0}%)",
        config.target.protocol_mix.udp_ratio * 100.0,
        config.target.protocol_mix.tcp_syn_ratio * 100.0,
        config.target.protocol_mix.tcp_ack_ratio * 100.0,
        config.target.protocol_mix.icmp_ratio * 100.0,
        config.target.protocol_mix.ipv6_ratio * 100.0,
        config.target.protocol_mix.arp_ratio * 100.0
    );
    if config.safety.dry_run {
        info!("   📋 Mode: SIMULATION ONLY - Safe for testing configurations");
    }
    info!("   Press Ctrl+C to stop gracefully");
    println!();
}

async fn finalize_execution(config: &Config, stats: &Arc<FloodStats>) {
    time::sleep(StdDuration::from_millis(100)).await;
    if config.safety.dry_run {
        info!("📈 Final Simulation Statistics (DRY-RUN):");
    } else {
        info!("📈 Final Statistics:");
    }
    stats.print_stats(None);

    if config.export.enabled {
        if let Err(e) = stats.export_stats(None).await {
            error!("Failed to export final stats: {}", e);
        }
    }

    if config.safety.dry_run {
        info!("✅ Simulation completed successfully (NO PACKETS SENT)");
        info!("📋 Dry-run mode: Configuration validated, packet generation tested");
    } else {
        info!("✅ Simulation completed successfully");
    }
}

fn spawn_monitoring_tasks(
    config: &Config,
    stats: Arc<FloodStats>,
    system_monitor: Arc<SystemMonitor>,
    running: Arc<AtomicBool>,
) {
    // Spawn statistics reporter with system monitoring
    let stats_clone = stats.clone();
    let running_clone = running.clone();
    let stats_interval = config.monitoring.stats_interval;
    tokio::spawn(async move {
        while running_clone.load(Ordering::Relaxed) {
            time::sleep(StdDuration::from_secs(stats_interval)).await;
            let sys_stats = system_monitor.get_system_stats().await;
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
}

fn spawn_worker_threads(
    config: &Config,
    stats: Arc<FloodStats>,
    running: Arc<AtomicBool>,
    multi_port_target: Arc<MultiPortTarget>,
    target_ip: IpAddr,
    tx_ipv4: Option<Arc<Mutex<pnet::transport::TransportSender>>>,
    tx_ipv6: Option<Arc<Mutex<pnet::transport::TransportSender>>>,
    tx_l2: Option<Arc<Mutex<Box<dyn pnet::datalink::DataLinkSender>>>>,
    dry_run: bool,
) -> Vec<tokio::task::JoinHandle<()>> {
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
                if dry_run {
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
    handles
}
