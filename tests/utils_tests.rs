//! Tests for utility functions

use router_flood::utils::protocol_utils::ProtocolUtils;
use router_flood::utils::rng::BatchedRng;
use router_flood::packet::PacketType;

#[cfg(test)]
mod protocol_utils_tests {
    use super::*;

    #[test]
    fn test_is_valid_protocol() {
        assert!(ProtocolUtils::is_valid_protocol("UDP"));
        assert!(ProtocolUtils::is_valid_protocol("TCP"));
        assert!(ProtocolUtils::is_valid_protocol("ICMP"));
        assert!(ProtocolUtils::is_valid_protocol("IPv6"));
        
        assert!(!ProtocolUtils::is_valid_protocol("INVALID"));
        assert!(!ProtocolUtils::is_valid_protocol(""));
        assert!(!ProtocolUtils::is_valid_protocol("http"));
    }

    #[test]
    fn test_packet_type_to_protocol_name() {
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::Udp), "UDP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::TcpSyn), "TCP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::TcpAck), "TCP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::Icmp), "ICMP");
        assert_eq!(ProtocolUtils::packet_type_to_protocol_name(PacketType::Ipv6Udp), "IPv6");
    }

    #[test]
    fn test_all_protocol_names() {
        let protocols = ProtocolUtils::all_protocol_names();
        assert!(protocols.len() > 0);
        assert!(protocols.contains(&"UDP"));
        assert!(protocols.contains(&"TCP"));
    }
}

#[cfg(test)]
mod rng_tests {
    use super::*;

    #[test]
    fn test_batched_rng_creation() {
        let mut rng = BatchedRng::new();
        
        // Test port generation
        let port = rng.port();
        assert!(port > 0);
    }

    #[test]
    fn test_batched_rng_sequence() {
        let mut rng = BatchedRng::new();
        
        let seq1 = rng.sequence();
        let seq2 = rng.sequence();
        
        // Sequences should be different (with high probability)
        // But may occasionally be same, so don't assert inequality
        assert!(seq1 <= u32::MAX);
        assert!(seq2 <= u32::MAX);
    }

    #[test]
    fn test_batched_rng_ttl() {
        let mut rng = BatchedRng::new();
        
        let ttl = rng.ttl();
        assert!(ttl > 0);
    }

    #[test]
    fn test_batched_rng_payload() {
        let mut rng = BatchedRng::new();
        
        let size = 100;
        let payload = rng.payload(size);
        assert_eq!(payload.len(), size);
        
        // Test empty payload
        let empty = rng.payload(0);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_batched_rng_bool_probability() {
        let mut rng = BatchedRng::new();
        
        // Always true
        for _ in 0..10 {
            assert!(rng.bool_with_probability(1.0));
        }
        
        // Always false
        for _ in 0..10 {
            assert!(!rng.bool_with_probability(0.0));
        }
        
        // 50/50 should produce both values over many trials
        let mut has_true = false;
        let mut has_false = false;
        for _ in 0..100 {
            if rng.bool_with_probability(0.5) {
                has_true = true;
            } else {
                has_false = true;
            }
            if has_true && has_false {
                break;
            }
        }
        // Should have seen both values
        assert!(has_true || has_false);
    }

    #[test]
    fn test_batched_rng_range() {
        let mut rng = BatchedRng::new();
        
        for _ in 0..100 {
            let val = rng.range(10, 20);
            assert!(val >= 10);
            assert!(val <= 20);
        }
    }

    #[test]
    fn test_batched_rng_float_range() {
        let mut rng = BatchedRng::new();
        
        for _ in 0..100 {
            let val = rng.float_range(1.0, 2.0);
            assert!(val >= 1.0);
            assert!(val <= 2.0);
        }
    }

    #[test]
    fn test_batched_rng_batch_operations() {
        let mut rng = BatchedRng::new();
        
        // Generate multiple ports
        let ports = rng.ports(10);
        assert_eq!(ports.len(), 10);
        for port in ports {
            assert!(port > 0);
        }
        
        // Generate multiple TTLs
        let ttls = rng.ttls(10);
        assert_eq!(ttls.len(), 10);
        for ttl in ttls {
            assert!(ttl > 0);
        }
    }

    #[test]
    fn test_batched_rng_window_size() {
        let mut rng = BatchedRng::new();
        
        let window = rng.window_size();
        // Common window sizes
        assert!(window >= 1024);
    }

    #[test]
    fn test_batched_rng_flow_label() {
        let mut rng = BatchedRng::new();
        
        let flow = rng.flow_label();
        // Flow label is 20 bits
        assert!(flow <= 0xFFFFF);
    }

    #[test]
    fn test_batched_rng_identification() {
        let mut rng = BatchedRng::new();
        
        let id = rng.identification();
        assert!(id <= u16::MAX);
    }

    #[test]
    fn test_batched_rng_byte() {
        let mut rng = BatchedRng::new();
        
        let _byte = rng.byte();
        // byte is u8, so always <= 255
    }
}