//! Core types for OPC DA Client operations

use std::sync::Arc;

/// OPC value types supported by the library
#[derive(Debug, Clone, PartialEq)]
pub enum OpcValue {
    /// 16-bit signed integer
    Int16(i16),
    /// 32-bit signed integer
    Int32(i32),
    /// 32-bit floating point number
    Float(f32),
    /// 64-bit floating point number
    Double(f64),
    /// String value
    String(String),
}

/// OPC data quality indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcQuality {
    /// Good quality data
    Good,
    /// Uncertain quality data
    Uncertain,
    /// Bad quality data
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
    fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality);
}

/// Internal callback container for FFI
pub(crate) struct OpcCallbackContainer {
    pub callback: Arc<dyn OpcDataCallback>,
}