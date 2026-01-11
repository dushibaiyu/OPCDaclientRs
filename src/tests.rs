//! Test suite for OPC DA Client wrapper

#[cfg(test)]
mod unit_tests {
use crate::types::{OpcValue, OpcQuality, OpcValueError};
    #[test]
    fn test_opc_value_conversions() {
        // Test i16 conversion
        let value = OpcValue::Int16(42);
        let result: i16 = value.try_into().unwrap();
        assert_eq!(result, 42);
        
        // Test i32 conversion
        let value = OpcValue::Int32(1000);
        let result: i32 = value.try_into().unwrap();
        assert_eq!(result, 1000);
        
        // Test f32 conversion
        let value = OpcValue::Float(3.14);
        let result: f32 = value.try_into().unwrap();
        assert_eq!(result, 3.14);
        
        // Test f64 conversion
        let value = OpcValue::Double(2.71828);
        let result: f64 = value.try_into().unwrap();
        assert_eq!(result, 2.71828);
        
        // Test String conversion
        let value = OpcValue::String("test".to_string());
        let result: String = value.try_into().unwrap();
        assert_eq!(result, "test");
    }
    
    #[test]
    fn test_opc_value_type_mismatch() {
        let value = OpcValue::Int16(42);
        
        // Should fail when trying to convert to wrong type
        let result: Result<i32, OpcValueError> = value.try_into();
        assert!(result.is_err());
        
        if let Err(OpcValueError::TypeMismatch { expected, actual }) = result {
            assert_eq!(expected, "Int32");
            assert_eq!(actual, "Int16");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }
    
    #[test]
    fn test_opc_quality_from_raw() {
        assert_eq!(OpcQuality::from_raw(192), OpcQuality::Good);
        assert_eq!(OpcQuality::from_raw(64), OpcQuality::Uncertain);
        assert_eq!(OpcQuality::from_raw(0), OpcQuality::Bad);
        assert_eq!(OpcQuality::from_raw(128), OpcQuality::Uncertain); // Default
    }
    
    #[test]
    fn test_opc_quality_to_raw() {
        assert_eq!(OpcQuality::Good.to_raw(), 192);
        assert_eq!(OpcQuality::Uncertain.to_raw(), 64);
        assert_eq!(OpcQuality::Bad.to_raw(), 0);
    }
    
    #[test]
    fn test_opc_value_type_name() {
        assert_eq!(OpcValue::Int16(0).type_name(), "Int16");
        assert_eq!(OpcValue::Int32(0).type_name(), "Int32");
        assert_eq!(OpcValue::Float(0.0).type_name(), "Float");
        assert_eq!(OpcValue::Double(0.0).type_name(), "Double");
        assert_eq!(OpcValue::String("".to_string()).type_name(), "String");
    }
    
    #[test]
    fn test_opc_value_raw_type() {
        assert_eq!(OpcValue::Int16(0).raw_type(), 2); // VT_I2
        assert_eq!(OpcValue::Int32(0).raw_type(), 3); // VT_I4
        assert_eq!(OpcValue::Float(0.0).raw_type(), 4); // VT_R4
        assert_eq!(OpcValue::Double(0.0).raw_type(), 5); // VT_R8
        assert_eq!(OpcValue::String("".to_string()).raw_type(), 8); // VT_BSTR
    }
}

#[cfg(test)]
mod integration_tests {
    
    #[test]
    #[ignore = "Requires OPC server"]
    fn test_client_initialization() {
        let _client = crate::OpcClient::new();
    }
    
    #[test]
    #[ignore = "Requires OPC server"]
    fn test_connect_to_server() {
        let client = crate::OpcClient::new().unwrap();
        let _result = client.connect_to_local_server("Matrikon.OPC.Simulation.1");
    }
}

#[cfg(test)]
mod mock_tests {
    use crate::types::{OpcValue, OpcQuality};
    use crate::OpcError;
    use std::sync::Arc;
    use crate::types::OpcDataCallback;
    
    /// Mock callback for testing
    struct MockCallback {
        pub calls: std::sync::Mutex<Vec<(String, String, OpcValue, OpcQuality, u64)>>,
    }
    
    impl MockCallback {
        fn new() -> Self {
            MockCallback {
                calls: std::sync::Mutex::new(Vec::new()),
            }
        }
        
        fn get_calls(&self) -> Vec<(String, String, OpcValue, OpcQuality, u64)> {
            self.calls.lock().unwrap().clone()
        }
    }
    
    impl OpcDataCallback for MockCallback {
        fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality, timestamp: u64) {
            self.calls.lock().unwrap().push((
                group_name.to_string(),
                item_name.to_string(),
                value,
                quality,
                timestamp,
            ));
        }
    }
    
    #[test]
    fn test_mock_callback() {
        let callback = Arc::new(MockCallback::new());
        
        // Simulate a data change
        callback.on_data_change(
            "TestGroup",
            "TestItem",
            OpcValue::Int32(42),
            OpcQuality::Good,
            0, // timestamp
        );
        
        let calls = callback.get_calls();
        assert_eq!(calls.len(), 1);
        
        let (group_name, item_name, value, quality, timestamp) = &calls[0];
        assert_eq!(group_name, "TestGroup");
        assert_eq!(item_name, "TestItem");
        assert_eq!(value, &OpcValue::Int32(42));
        assert_eq!(quality, &OpcQuality::Good);
        assert_eq!(timestamp, &0);
    }
    
    #[test]
    fn test_error_types() {
        let operation_error = OpcError::operation_failed("test operation");
        assert!(matches!(operation_error, OpcError::OperationFailed(_)));
        
        let connection_error = OpcError::connection_failed("test connection");
        assert!(matches!(connection_error, OpcError::ConnectionFailed(_)));
        
        let param_error = OpcError::invalid_parameters("test params");
        assert!(matches!(param_error, OpcError::InvalidParameters(_)));
    }
}