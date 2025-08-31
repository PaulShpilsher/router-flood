# Public API Baseline Documentation

This document captures the current public API before restructuring to ensure compatibility.

## Core Exports from lib.rs

### Configuration Types
- `Config` - Main configuration structure
- `Target` - Target configuration
- `LoadConfig` - Load/attack configuration  
- `Safety` - Safety settings
- `Monitoring` - Monitoring configuration
- `Export` - Export settings
- `ExportFormat` - Export format enum
- `ProtocolMix` - Protocol distribution settings

### Core Simulation Types
- `Simulation` - Main simulation runner
- `SimulationRAII` - RAII wrapper for simulation
- `Workers` - Worker management
- `MultiPortTarget` - Multi-port target management

### Error Types
- `Result` - Type alias for Result<T, RouterFloodError>
- `RouterFloodError` - Main error type

### Packet Types
- `PacketBuilder` - Packet construction
- `PacketStrategy` - Packet building strategy trait
- `PacketType` - Packet type enum
- `PacketTarget` - Packet target configuration

### Statistics
- `Stats` - Statistics collection

### Utility Types
- `BufferPool` - Buffer management
- `Terminal` - Terminal control
- `TerminalGuard` - Terminal RAII guard
- `ResourceGuard` - Resource RAII guard

### Type Aliases
- `StatsRef = Arc<Stats>`
- `ConfigRef = Arc<Config>`
- `PoolRef = Arc<BufferPool>`
- `WorkersRef = Arc<Workers>`

## CLI Module Exports
- `parse_arguments()` - Parse CLI arguments
- `process_cli_config()` - Process CLI configuration
- `handle_pre_execution_commands()` - Handle pre-execution commands
- `parse_ports()` - Parse port specifications
- `parse_positive_number()` - Parse positive numbers
- `parse_export_format()` - Parse export format
- `Interactive` - Interactive CLI mode
- `GuidedCli` - Guided CLI mode
- `GuidanceLevel` - Guidance level enum
- `validate_target_ip()` - Validate target IP

## Notes
- This API must be maintained for backward compatibility
- New functionality should be additive, not breaking
- Deprecated items should be marked but not removed immediately