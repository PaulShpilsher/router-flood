//! Buffer pool integration tests
//!
//! Tests for buffer pool functionality and zero-copy packet building integration.

use router_flood::buffer_pool::WorkerBufferPool;
use router_flood::config::ProtocolMix;
use router_flood::packet::{PacketBuilder, PacketType};
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_worker_buffer_pool_basic() {
    let mut pool = WorkerBufferPool::new(1400, 5, 10);
    
    // Should start with 5 buffers
    assert_eq!(pool.pool_size(), 5);
    
    // Get a buffer
    let buffer = pool.get_buffer();
    assert_eq!(buffer.len(), 1400);
    assert_eq!(pool.pool_size(), 4);
    
    // Return the buffer
    pool.return_buffer(buffer);
    assert_eq!(pool.pool_size(), 5);
}

#[test]
fn test_buffer_pool_reuse() {
    let mut pool = WorkerBufferPool::new(1024, 2, 3);
    
    // Get all buffers
    let buf1 = pool.get_buffer();
    let buf2 = pool.get_buffer();
    assert_eq!(pool.pool_size(), 0);
    
    // Getting another buffer should create a new one
    let buf3 = pool.get_buffer();
    assert_eq!(pool.pool_size(), 0);
    
    // Return buffers
    pool.return_buffer(buf1);
    pool.return_buffer(buf2);
    pool.return_buffer(buf3);
    
    // Should be at max pool size
    assert_eq!(pool.pool_size(), 3);
}

#[test]
fn test_buffer_pool_max_size_limit() {
    let mut pool = WorkerBufferPool::new(512, 2, 3);
    
    // Create extra buffers beyond pool capacity
    let buf1 = vec![0u8; 512];
    let buf2 = vec![0u8; 512];
    let buf3 = vec![0u8; 512];
    let buf4 = vec![0u8; 512]; // This should be dropped
    
    pool.return_buffer(buf1);
    pool.return_buffer(buf2);
    pool.return_buffer(buf3);
    pool.return_buffer(buf4);
    
    // Should not exceed max size
    assert_eq!(pool.pool_size(), 3);
}

#[test]
fn test_zero_copy_with_buffer_pool_simulation() {
    // Simulate the worker thread workflow
    let mut pool = WorkerBufferPool::new(1400, 3, 5);
    let protocol_mix = ProtocolMix {
        udp_ratio: 0.5,
        tcp_syn_ratio: 0.3,
        tcp_ack_ratio: 0.1,
        icmp_ratio: 0.1,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1200), protocol_mix);
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
    let target_port = 80;
    
    // Simulate multiple packet building cycles
    for _ in 0..10 {
        let packet_type = packet_builder.next_packet_type();
        
        // Get buffer from pool
        let mut buffer = pool.get_buffer();
        
        // Build packet into buffer
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            target_ip,
            target_port,
        );
        
        match result {
            Ok((packet_size, _protocol_name)) => {
                // Verify packet was built successfully
                assert!(packet_size > 20); // At least IP header
                assert!(packet_size <= buffer.len());
                
                // Simulate sending the packet (using slice)
                let _packet_data = &buffer[..packet_size];
                
                // Return buffer to pool
                pool.return_buffer(buffer);
            }
            Err(_) => {
                // Return buffer even on error
                pool.return_buffer(buffer);
                panic!("Packet building should not fail with adequate buffer");
            }
        }
    }
    
    // Pool should still have buffers available for reuse
    assert!(pool.pool_size() > 0);
}

#[test]
fn test_zero_copy_all_packet_types() {
    let mut pool = WorkerBufferPool::new(1500, 2, 4);
    let protocol_mix = ProtocolMix {
        udp_ratio: 0.2,
        tcp_syn_ratio: 0.2,
        tcp_ack_ratio: 0.2,
        icmp_ratio: 0.2,
        ipv6_ratio: 0.1,
        arp_ratio: 0.1,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 1400), protocol_mix);
    let target_ipv4 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let target_ipv6 = IpAddr::V6("2001:db8::1".parse().unwrap());
    let target_port = 443;
    
    let packet_types = vec![
        (PacketType::Udp, target_ipv4),
        (PacketType::TcpSyn, target_ipv4),
        (PacketType::TcpAck, target_ipv4),
        (PacketType::Icmp, target_ipv4),
        (PacketType::Ipv6Udp, target_ipv6),
        (PacketType::Ipv6Tcp, target_ipv6),
        (PacketType::Ipv6Icmp, target_ipv6),
        (PacketType::Arp, target_ipv4),
    ];
    
    for (packet_type, target_ip) in packet_types {
        let mut buffer = pool.get_buffer();
        
        let result = packet_builder.build_packet_into_buffer(
            &mut buffer,
            packet_type,
            target_ip,
            target_port,
        );
        
        match result {
            Ok((packet_size, protocol_name)) => {
                // Verify packet characteristics
                assert!(packet_size > 0);
                assert!(packet_size <= buffer.len());
                assert!(!protocol_name.is_empty());
                
                // Verify buffer has non-zero content
                let packet_data = &buffer[..packet_size];
                assert!(packet_data.iter().any(|&b| b != 0));
                
                pool.return_buffer(buffer);
            }
            Err(e) => {
                pool.return_buffer(buffer);
                panic!("Failed to build {:?} packet: {}", packet_type, e);
            }
        }
    }
}

#[test]
fn test_buffer_pool_performance_characteristics() {
    let mut pool = WorkerBufferPool::new(1400, 10, 20);
    let initial_pool_size = pool.pool_size();
    
    // Simulate high-throughput scenario
    let mut buffers = Vec::new();
    
    // Get many buffers quickly
    for _ in 0..15 {
        buffers.push(pool.get_buffer());
    }
    
    // Pool should be depleted but still functional
    assert_eq!(pool.pool_size(), 0);
    
    // Return all buffers
    for buffer in buffers {
        pool.return_buffer(buffer);
    }
    
    // Should not exceed max pool size
    assert!(pool.pool_size() <= 20);
    assert!(pool.pool_size() >= initial_pool_size);
}

#[test] 
fn test_buffer_size_validation() {
    let mut pool = WorkerBufferPool::new(50, 2, 4); // Very small buffers
    let protocol_mix = ProtocolMix {
        udp_ratio: 1.0,
        tcp_syn_ratio: 0.0,
        tcp_ack_ratio: 0.0,
        icmp_ratio: 0.0,
        ipv6_ratio: 0.0,
        arp_ratio: 0.0,
    };
    
    let mut packet_builder = PacketBuilder::new((64, 200), protocol_mix); // Reasonable payload size range
    let target_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let target_port = 80;
    
    let mut buffer = pool.get_buffer();
    
    // Should fail with buffer too small error
    let result = packet_builder.build_packet_into_buffer(
        &mut buffer,
        PacketType::Udp,
        target_ip,
        target_port,
    );
    
    assert!(result.is_err());
    let error_msg = result.err().unwrap();
    assert!(error_msg.contains("Buffer too small"));
    
    // Always return buffer even on error
    pool.return_buffer(buffer);
}
