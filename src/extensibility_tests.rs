//! Tests demonstrating extensibility patterns
//!
//! This module demonstrates how the implemented design patterns enable
//! extensibility without modifying existing code.

#[cfg(test)]
mod tests {
    use crate::packet::*;
    use crate::stats::*;
    use crate::error::Result;
    use std::sync::Arc;
    use std::net::IpAddr;
    
    // === Plugin System Tests ===
    
    #[test]
    fn test_plugin_registry() {
        let registry = PluginRegistry::new();
        
        // Verify registry is created
        assert_eq!(registry.plugins().len(), 0);
        
        // Test getting strategies for unregistered type
        let strategies = registry.get_strategies(PacketType::Udp);
        assert_eq!(strategies.len(), 0);
    }
    
    // === Observer Pattern Tests ===
    
    #[test]
    fn test_stats_observer() {
        use crate::stats::observer::*;
        
        let subject = StatsSubject::new();
        
        // Create different observers
        let console_observer = Arc::new(ConsoleObserver::new(false));
        let metrics_observer = Arc::new(MetricsObserver::new());
        
        // Attach observers
        subject.attach(console_observer.clone());
        subject.attach(metrics_observer.clone());
        
        assert_eq!(subject.observer_count(), 2);
        
        // Send events
        subject.notify(&StatsEvent::PacketSent {
            bytes: 1024,
            protocol: "UDP".to_string(),
        });
        
        // Verify metrics were updated
        let (packets, bytes, _, _) = metrics_observer.get_metrics();
        assert_eq!(packets, 1);
        assert_eq!(bytes, 1024);
    }
    
    #[test]
    fn test_composite_observer() {
        use crate::stats::observer::*;
        
        let composite = ObserverBuilder::new()
            .with_console(false)
            .with_metrics()
            .build();
        
        let subject = StatsSubject::new();
        
        // Check initial count
        assert_eq!(subject.observer_count(), 0);
        
        // Attach composite observer (keep Arc alive)
        subject.attach(composite.clone());
        
        // Should have one composite observer containing multiple sub-observers
        assert_eq!(subject.observer_count(), 1);
        
        // Observer is still alive since we hold a reference
        drop(composite);
        
        // Now it should be 0 since we dropped the Arc
        assert_eq!(subject.observer_count(), 0);
    }
    
    // === Chain of Responsibility Tests ===
    
    #[test]
    fn test_handler_chain() {
        use crate::packet::chain::*;
        
        let chain = ChainBuilder::new()
            .with_size_validation(20, 1500)
            .with_checksum()
            .with_ttl(64)
            .build();
        
        let target = crate::packet::Target::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            80
        );
        let mut context = PacketContext::new(
            vec![0u8; 100],
            100,
            target,
            "TCP".to_string(),
        );
        
        // Process through chain
        assert!(chain.process(&mut context).is_ok());
        
        // Verify TTL was set
        assert_eq!(context.metadata.ttl, Some(64));
        
        // Verify checksum was calculated
        assert!(context.metadata.custom.contains_key("checksum"));
    }
    
    #[test]
    fn test_chain_abort() {
        use crate::packet::chain::*;
        
        let chain = ChainBuilder::new()
            .with_size_validation(100, 200)  // Will reject our 50-byte packet
            .with_checksum()
            .build();
        
        let target = crate::packet::Target::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            80
        );
        let mut context = PacketContext::new(
            vec![0u8; 50],
            50,
            target,
            "UDP".to_string(),
        );
        
        // Should fail size validation
        assert!(chain.process(&mut context).is_err());
    }
    
    // === Decorator Pattern Tests ===
    
    #[test]
    fn test_strategy_decorator() {
        use crate::packet::decorator::*;
        
        // Create a mock strategy
        struct MockStrategy;
        
        impl PacketStrategy for MockStrategy {
            fn build_packet(&mut self, _target: &Target, buffer: &mut [u8]) -> Result<usize> {
                let data = b"test packet";
                buffer[..data.len()].copy_from_slice(data);
                Ok(data.len())
            }
            
            fn protocol_name(&self) -> &'static str {
                "MOCK"
            }
            
            fn max_packet_size(&self) -> usize {
                1500
            }
            
            fn is_compatible_with(&self, _target_ip: IpAddr) -> bool {
                true
            }
        }
        
        // Decorate with encryption
        let decorated = DecoratorBuilder::new(Box::new(MockStrategy))
            .with_encryption(vec![0x42])
            .build();
        
        // Verify the decorator chain works
        assert_eq!(decorated.protocol_name(), "MOCK");
        assert_eq!(decorated.max_packet_size(), 1500);
    }
    
    #[test]
    fn test_fragmentation_decorator() {
        use crate::packet::decorator::*;
        
        struct LargePacketStrategy;
        
        impl PacketStrategy for LargePacketStrategy {
            fn build_packet(&mut self, _target: &Target, buffer: &mut [u8]) -> Result<usize> {
                // Create a large packet
                for i in 0..1000 {
                    buffer[i] = (i % 256) as u8;
                }
                Ok(1000)
            }
            
            fn protocol_name(&self) -> &'static str {
                "LARGE"
            }
            
            fn max_packet_size(&self) -> usize {
                2000
            }
            
            fn is_compatible_with(&self, _target_ip: IpAddr) -> bool {
                true
            }
        }
        
        // Decorate with fragmentation
        let mut decorated = FragmentationDecorator::new(Box::new(LargePacketStrategy), 500);
        
        let target = crate::packet::Target::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            80
        );
        let mut buffer = vec![0u8; 2000];
        
        // First fragment should be 500 bytes
        let size = decorated.build_packet(&target, &mut buffer).unwrap();
        assert_eq!(size, 500);
    }
    
    // === Integration Test ===
    
    #[test]
    fn test_extensibility_integration() {
        use crate::packet::chain::*;
        use crate::stats::observer::*;
        
        // Create a processing chain
        let chain = ChainBuilder::new()
            .with_size_validation(20, 1500)
            .with_ttl(64)
            .with_checksum()
            .build();
        
        // Create statistics observers
        let subject = StatsSubject::new();
        let metrics = Arc::new(MetricsObserver::new());
        subject.attach(metrics.clone());
        
        // Simulate packet processing
        for _ in 0..10 {
            let target = crate::packet::Target::new(
                "192.168.1.1".parse::<IpAddr>().unwrap(),
                80
            );
            let mut context = PacketContext::new(
                vec![0u8; 100],
                100,
                target,
                "TCP".to_string(),
            );
            
            // Process through chain
            if chain.process(&mut context).is_ok() {
                // Notify observers of successful packet
                subject.notify(&StatsEvent::PacketSent {
                    bytes: 100,
                    protocol: "TCP".to_string(),
                });
            }
        }
        
        // Verify metrics
        let (packets, bytes, _, _) = metrics.get_metrics();
        assert_eq!(packets, 10);
        assert_eq!(bytes, 1000);
    }
    
    // === Custom Extension Examples ===
    
    /// Example custom handler that adds custom headers
    struct CustomHeaderHandler {
        header_name: String,
        header_value: String,
    }
    
    impl PacketHandler for CustomHeaderHandler {
        fn handle(&self, context: &mut PacketContext) -> Result<ProcessResult> {
            context.metadata.custom.insert(
                self.header_name.clone(),
                self.header_value.clone(),
            );
            Ok(ProcessResult::Continue)
        }
        
        fn name(&self) -> &str {
            "CustomHeader"
        }
    }
    
    #[test]
    fn test_custom_handler() {
        let handler = Arc::new(CustomHeaderHandler {
            header_name: "X-Custom".to_string(),
            header_value: "test-value".to_string(),
        });
        
        let chain = HandlerChain::new()
            .add_handler(handler);
        
        let target = crate::packet::Target::new(
            "192.168.1.1".parse::<IpAddr>().unwrap(),
            80
        );
        let mut context = PacketContext::new(
            vec![0u8; 100],
            100,
            target,
            "HTTP".to_string(),
        );
        
        chain.process(&mut context).unwrap();
        
        assert_eq!(
            context.metadata.custom.get("X-Custom"),
            Some(&"test-value".to_string())
        );
    }
    
    /// Example custom observer that filters events
    struct FilteredObserver {
        protocol_filter: String,
        count: std::sync::Mutex<u64>,
    }
    
    impl StatsObserver for FilteredObserver {
        fn on_event(&self, event: &StatsEvent) {
            if let StatsEvent::PacketSent { protocol, .. } = event {
                if protocol == &self.protocol_filter {
                    let mut count = self.count.lock().unwrap();
                    *count += 1;
                }
            }
        }
    }
    
    #[test]
    fn test_custom_observer() {
        use crate::stats::observer::*;
        
        let observer = Arc::new(FilteredObserver {
            protocol_filter: "UDP".to_string(),
            count: std::sync::Mutex::new(0),
        });
        
        let subject = StatsSubject::new();
        subject.attach(observer.clone());
        
        // Send mixed events
        subject.notify(&StatsEvent::PacketSent {
            bytes: 100,
            protocol: "TCP".to_string(),
        });
        subject.notify(&StatsEvent::PacketSent {
            bytes: 200,
            protocol: "UDP".to_string(),
        });
        subject.notify(&StatsEvent::PacketSent {
            bytes: 300,
            protocol: "UDP".to_string(),
        });
        
        // Should only count UDP packets
        let count = *observer.count.lock().unwrap();
        assert_eq!(count, 2);
    }
}