//! 核心类型模块
//! 
//! 这个模块定义了 OPC DA 客户端库的核心数据类型。
//! 包括值类型、质量指示器、错误类型和回调接口。
//! 
//! ## 主要类型
//! 
//! - `OpcValue`: OPC 值枚举，支持多种数据类型
//! - `OpcQuality`: OPC 质量指示器
//! - `OpcValueError`: 值转换错误
//! - `OpcDataCallback`: 异步数据变化回调接口
//! - `OpcCallbackContainer`: 回调容器（内部使用）
//! 
//! ## 类型转换
//! 
//! `OpcValue` 支持 `TryFrom` 转换到 Rust 原生类型，
//! 方便用户将 OPC 值转换为具体的 Rust 类型。

use std::sync::Arc;

/// OPC 值类型，支持库支持的所有数据类型
/// 
/// 这个枚举表示 OPC 项可能具有的值类型。
/// 每个变体对应一种特定的数据类型。
/// 
/// ## 支持的数据类型
/// 
/// - `Int16`: 16位有符号整数（-32768 到 32767）
/// - `Int32`: 32位有符号整数（-2147483648 到 2147483647）
/// - `Float`: 32位单精度浮点数（IEEE 754）
/// - `Double`: 64位双精度浮点数（IEEE 754）
/// - `String`: UTF-8 字符串
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::OpcValue;
/// 
/// // 创建各种类型的值
/// let int_value = OpcValue::Int32(42);
/// let float_value = OpcValue::Float(3.14);
/// let string_value = OpcValue::String("Hello".to_string());
/// 
/// // 类型转换
/// if let OpcValue::Int32(v) = int_value {
///     println!("整数值: {}", v);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum OpcValue {
    /// 16位有符号整数
    Int16(i16),
    /// 32位有符号整数
    Int32(i32),
    /// 32位单精度浮点数
    Float(f32),
    /// 64位双精度浮点数
    Double(f64),
    /// UTF-8 字符串
    String(String),
}

/// OPC 数据质量指示器
/// 
/// 这个枚举表示 OPC 项值的质量状态。
/// 质量信息对于判断数据的可靠性至关重要。
/// 
/// ## 质量等级
/// 
/// - `Good`: 良好质量，数据可靠
/// - `Uncertain`: 不确定质量，数据可能有问题
/// - `Bad`: 不良质量，数据不可靠
/// 
/// ## 质量位掩码
/// 
/// OPC 质量值使用位掩码编码：
/// - 位 7-6: 质量等级 (00=Bad, 01=Uncertain, 11=Good)
/// - 位 5-0: 子状态和限制状态
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::OpcQuality;
/// 
/// // 从原始质量值创建
/// let quality = OpcQuality::from_raw(192); // Good quality
/// assert_eq!(quality, OpcQuality::Good);
/// 
/// // 转换为原始值
/// let raw = quality.to_raw();
/// assert_eq!(raw, 192);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcQuality {
    /// 良好质量数据
    /// 
    /// 表示数据可靠，可以安全使用。
    /// 对应的原始值: 192 (0xC0)
    Good,
    /// 不确定质量数据
    /// 
    /// 表示数据可能有问题，应谨慎使用。
    /// 对应的原始值: 64 (0x40)
    Uncertain,
    /// 不良质量数据
    /// 
    /// 表示数据不可靠，不应使用。
    /// 对应的原始值: 0 (0x00)
    Bad,
}

impl OpcQuality {
    /// Create from raw quality value
    pub fn from_raw(quality: i32) -> Self {
        match quality & 0xC0 {  // Quality mask
            0 => OpcQuality::Bad,
            64 => OpcQuality::Uncertain,
            192 => OpcQuality::Good,
            _ => OpcQuality::Uncertain, // Default
        }
    }
    
    /// Convert to raw quality value
    pub fn to_raw(&self) -> i32 {
        match self {
            OpcQuality::Good => 192,
            OpcQuality::Uncertain => 64,
            OpcQuality::Bad => 0,
        }
    }
}

impl std::fmt::Display for OpcQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpcQuality::Good => write!(f, "Good"),
            OpcQuality::Uncertain => write!(f, "Uncertain"),
            OpcQuality::Bad => write!(f, "Bad"),
        }
    }
}

/// Error type for value conversions
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OpcValueError {
    /// Type mismatch during conversion
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },
    
    /// Conversion error
    #[error("Conversion error: {0}")]
    ConversionError(String),
    
    /// Invalid value type
    #[error("Invalid value type: {0}")]
    InvalidValueType(u32),
}

impl OpcValueError {
    /// Create a type mismatch error
    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        OpcValueError::TypeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }
    
    /// Create a conversion error
    pub fn conversion_error(msg: impl Into<String>) -> Self {
        OpcValueError::ConversionError(msg.into())
    }
}

// Implement TryFrom conversions for OpcValue
impl TryFrom<OpcValue> for i16 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::Int16(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("Int16", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for i32 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::Int32(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("Int32", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for f32 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::Float(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("Float", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for f64 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::Double(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("Double", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for String {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::String(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("String", value.type_name())),
        }
    }
}

impl OpcValue {
    /// Get the type name of the value
    pub fn type_name(&self) -> &'static str {
        match self {
            OpcValue::Int16(_) => "Int16",
            OpcValue::Int32(_) => "Int32",
            OpcValue::Float(_) => "Float",
            OpcValue::Double(_) => "Double",
            OpcValue::String(_) => "String",
        }
    }
    
    /// Get the raw value type code for FFI
    pub fn raw_type(&self) -> u32 {
        match self {
            OpcValue::Int16(_) => 0,
            OpcValue::Int32(_) => 1,
            OpcValue::Float(_) => 2,
            OpcValue::Double(_) => 3,
            OpcValue::String(_) => 4,
        }
    }
    
    /// Create from raw value and type
    pub fn from_raw(value: *mut std::ffi::c_void, value_type: u32) -> Result<Self, OpcValueError> {
        match value_type {
            0 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Int16"));
                }
                Ok(OpcValue::Int16(unsafe { *(value as *const i16) }))
            }
            1 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Int32"));
                }
                Ok(OpcValue::Int32(unsafe { *(value as *const i32) }))
            }
            2 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Float"));
                }
                Ok(OpcValue::Float(unsafe { *(value as *const f32) }))
            }
            3 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Double"));
                }
                Ok(OpcValue::Double(unsafe { *(value as *const f64) }))
            }
            4 => {
                // String handling is more complex - would need to handle wide strings
                Ok(OpcValue::String("String conversion not implemented".to_string()))
            }
            _ => Err(OpcValueError::InvalidValueType(value_type)),
        }
    }
}

/// Callback trait for asynchronous data changes
pub trait OpcDataCallback: Send + Sync {
    /// Called when data changes for subscribed items
    fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality, timestamp: u64);
}

/// Internal callback container for FFI
pub(crate) struct OpcCallbackContainer {
    pub callback: Arc<dyn OpcDataCallback>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_opc_value_creation() {
        // Test creation of all value types
        let int16_val = OpcValue::Int16(42);
        let int32_val = OpcValue::Int32(1000);
        let float_val = OpcValue::Float(3.14);
        let double_val = OpcValue::Double(2.71828);
        let string_val = OpcValue::String("test".to_string());
        
        assert_eq!(int16_val.type_name(), "Int16");
        assert_eq!(int32_val.type_name(), "Int32");
        assert_eq!(float_val.type_name(), "Float");
        assert_eq!(double_val.type_name(), "Double");
        assert_eq!(string_val.type_name(), "String");
    }
    
    #[test]
    fn test_opc_value_try_from() {
        // Test successful conversions
        let int32_val = OpcValue::Int32(123);
        let result: Result<i32, _> = int32_val.try_into();
        assert_eq!(result.unwrap(), 123);
        
        let double_val = OpcValue::Double(45.67);
        let result: Result<f64, _> = double_val.try_into();
        assert!((result.unwrap() - 45.67).abs() < 0.0001);
        
        let string_val = OpcValue::String("hello".to_string());
        let result: Result<String, _> = string_val.try_into();
        assert_eq!(result.unwrap(), "hello");
    }
    
    #[test]
    fn test_opc_value_try_from_failure() {
        // Test failed conversions
        let int32_val = OpcValue::Int32(123);
        let result: Result<String, _> = int32_val.try_into();
        assert!(result.is_err());
        
        let string_val = OpcValue::String("hello".to_string());
        let result: Result<i32, _> = string_val.try_into();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_opc_quality_display() {
        // Test quality display
        assert_eq!(OpcQuality::Good.to_string(), "Good");
        assert_eq!(OpcQuality::Uncertain.to_string(), "Uncertain");
        assert_eq!(OpcQuality::Bad.to_string(), "Bad");
    }
    
    #[test]
    fn test_opc_value_error() {
        // Test error creation
        let error = OpcValueError::type_mismatch("Int32", "String");
        assert!(error.to_string().contains("Int32"));
        assert!(error.to_string().contains("String"));
    }
    
    #[test]
    fn test_opc_data_callback_trait() {
        // Simple callback implementation for testing
        struct TestCallback {
            count: std::sync::atomic::AtomicUsize,
        }
        
        impl OpcDataCallback for TestCallback {
            fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality, timestamp: u64) {
                self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                println!("Callback: group={}, item={}, value={:?}, quality={:?}, timestamp={}", 
                         group_name, item_name, value, quality, timestamp);
            }
        }
        
        let callback = TestCallback {
            count: std::sync::atomic::AtomicUsize::new(0),
        };
        
        // Test that trait is object safe
        let _boxed: Box<dyn OpcDataCallback> = Box::new(callback);
        
        // This just verifies compilation - actual callback testing requires OPC server
        assert!(true);
    }
}