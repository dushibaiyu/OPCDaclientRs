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
#[cfg(windows)]
use windows::Win32::System::Com as olecom;

// VARENUM constants (VARTYPE values)
// These correspond to the Windows VARENUM enumeration
const VT_EMPTY: u32 = 0;
const VT_NULL: u32 = 1;
const VT_I2: u32 = 2;
const VT_I4: u32 = 3;
const VT_R4: u32 = 4;
const VT_R8: u32 = 5;
const VT_CY: u32 = 6;
const VT_DATE: u32 = 7;
const VT_BSTR: u32 = 8;
const VT_DISPATCH: u32 = 9;
const VT_ERROR: u32 = 10;
const VT_BOOL: u32 = 11;
const VT_VARIANT: u32 = 12;
const VT_UNKNOWN: u32 = 13;
const VT_DECIMAL: u32 = 14;
const VT_I1: u32 = 16;
const VT_UI1: u32 = 17;
const VT_UI2: u32 = 18;
const VT_UI4: u32 = 19;
const VT_I8: u32 = 20;
const VT_UI8: u32 = 21;
const VT_INT: u32 = 22;
const VT_UINT: u32 = 23;
const VT_VOID: u32 = 24;
const VT_HRESULT: u32 = 25;
const VT_PTR: u32 = 26;
const VT_SAFEARRAY: u32 = 27;
const VT_CARRAY: u32 = 28;
const VT_USERDEFINED: u32 = 29;
const VT_LPSTR: u32 = 30;
const VT_LPWSTR: u32 = 31;
const VT_RECORD: u32 = 36;
const VT_INT_PTR: u32 = 37;
const VT_UINT_PTR: u32 = 38;
const VT_FILETIME: u32 = 64;
const VT_BLOB: u32 = 65;
const VT_STREAM: u32 = 66;
const VT_STORAGE: u32 = 67;
const VT_STREAMED_OBJECT: u32 = 68;
const VT_STORED_OBJECT: u32 = 69;
const VT_BLOB_OBJECT: u32 = 70;
const VT_CF: u32 = 71;
const VT_CLSID: u32 = 72;
const VT_VERSIONED_STREAM: u32 = 73;
const VT_BSTR_BLOB: u32 = 0xfff;
const VT_VECTOR: u32 = 0x1000;
const VT_ARRAY: u32 = 0x2000;
const VT_BYREF: u32 = 0x4000;
const VT_RESERVED: u32 = 0x8000;
const VT_ILLEGAL: u32 = 0xffff;
const VT_ILLEGALMASKED: u32 = 0xfff;
const VT_TYPEMASK: u32 = 0xfff;

// Windows DECIMAL structure (simplified)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Decimal {
    w_reserved: u16,
    scale: u8,
    sign: u8,
    hi32: u32,
    lo64: u64,
}

impl Decimal {
    fn to_string(&self) -> String {
        // Convert DECIMAL to string representation
        // DECIMAL is stored as 96-bit integer (hi32 + lo64) with scale
        // sign: 0 for positive, 0x80 for negative
        // scale: number of decimal digits (0-28)
        
        let is_negative = self.sign == 0x80;
        
        // Combine hi32 and lo64 to get 96-bit value
        // lo64 is already 64 bits, hi32 is high 32 bits
        // We'll use u128 for simplicity
        let value_lo = self.lo64 as u128;
        let value_hi = (self.hi32 as u128) << 64;
        let mut value = value_lo + value_hi;
        
        if is_negative {
            value = (!value).wrapping_add(1); // Two's complement for negative
        }
        
        // Apply scaling: divide by 10^scale
        let scale = self.scale as u32;
        if scale == 0 {
            format!("{}", value as i128)
        } else {
            let divisor = 10u128.pow(scale);
            let integer_part = value / divisor;
            let fractional_part = value % divisor;
            
            if fractional_part == 0 {
                format!("{}", integer_part as i128)
            } else {
                // Format with proper decimal places
                format!("{}.{:0width$}", integer_part as i128, fractional_part, width = scale as usize)
            }
        }
    }
}

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
    /// 8位有符号整数
    Int8(i8),
    /// 8位无符号整数
    UInt8(u8),
    /// 16位有符号整数
    Int16(i16),
    /// 16位无符号整数
    UInt16(u16),
    /// 32位有符号整数
    Int32(i32),
    /// 32位无符号整数
    UInt32(u32),
    /// 64位有符号整数
    Int64(i64),
    /// 64位无符号整数
    UInt64(u64),
    /// 平台相关有符号整数
    INT(isize),
    /// 平台相关无符号整数
    UINT(usize),
    /// 32位单精度浮点数
    Float(f32),
    /// 64位双精度浮点数
    Double(f64),
    /// 布尔值
    Bool(bool),
    /// 货币类型 (64位整数，缩放10000)
    Cy(i64),
    /// 小数类型 (96位整数，缩放因子)
    Decimal(String), // TODO: 实现合适的十进制类型
    /// 日期类型 (OLE自动化日期)
    Date(f64),
    /// UTF-8 字符串
    String(String),
    /// 16位有符号整数数组
    ArrayInt16(Vec<i16>),
    /// 16位无符号整数数组
    ArrayUInt16(Vec<u16>),
    /// 32位有符号整数数组
    ArrayInt32(Vec<i32>),
    /// 32位无符号整数数组
    ArrayUInt32(Vec<u32>),
    /// 64位有符号整数数组
    ArrayInt64(Vec<i64>),
    /// 64位无符号整数数组
    ArrayUInt64(Vec<u64>),
    /// 32位浮点数数组
    ArrayFloat(Vec<f32>),
    /// 64位浮点数数组
    ArrayDouble(Vec<f64>),
    /// 布尔值数组
    ArrayBool(Vec<bool>),
    /// 字符串数组
    ArrayString(Vec<String>),
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

impl TryFrom<OpcValue> for i8 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::Int8(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("I1", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for u8 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::UInt8(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("UI1", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for u16 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::UInt16(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("UI2", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for u32 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::UInt32(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("UI4", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for i64 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::Int64(v) => Ok(v),
            OpcValue::Cy(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("I8 or Cy", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for u64 {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::UInt64(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("UI8", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for isize {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::INT(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("INT", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for usize {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::UINT(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("UINT", value.type_name())),
        }
    }
}

impl TryFrom<OpcValue> for bool {
    type Error = OpcValueError;
    
    fn try_from(value: OpcValue) -> Result<Self, Self::Error> {
        match value {
            OpcValue::Bool(v) => Ok(v),
            _ => Err(OpcValueError::type_mismatch("Bool", value.type_name())),
        }
    }
}



impl OpcValue {
    /// Get the type name of the value
    pub fn type_name(&self) -> &'static str {
        match self {
            OpcValue::Int8(_) => "I1",
            OpcValue::UInt8(_) => "UI1",
            OpcValue::Int16(_) => "Int16",
            OpcValue::UInt16(_) => "UI2",
            OpcValue::Int32(_) => "Int32",
            OpcValue::UInt32(_) => "UI4",
            OpcValue::Int64(_) => "I8",
            OpcValue::UInt64(_) => "UI8",
            OpcValue::INT(_) => "INT",
            OpcValue::UINT(_) => "UINT",
            OpcValue::Float(_) => "Float",
            OpcValue::Double(_) => "Double",
            OpcValue::Bool(_) => "Bool",
            OpcValue::Cy(_) => "Cy",
            OpcValue::Decimal(_) => "Decimal",
            OpcValue::Date(_) => "Date",
            OpcValue::String(_) => "String",
            OpcValue::ArrayInt16(_) => "ArrayInt16",
            OpcValue::ArrayUInt16(_) => "ArrayUI2",
            OpcValue::ArrayInt32(_) => "ArrayInt32",
            OpcValue::ArrayUInt32(_) => "ArrayUI4",
            OpcValue::ArrayInt64(_) => "ArrayI8",
            OpcValue::ArrayUInt64(_) => "ArrayUI8",
            OpcValue::ArrayFloat(_) => "ArrayFloat",
            OpcValue::ArrayDouble(_) => "ArrayDouble",
            OpcValue::ArrayBool(_) => "ArrayBool",
            OpcValue::ArrayString(_) => "ArrayString",
        }
    }
    
    /// Get the raw value type code for FFI (VARTYPE value)
    pub fn raw_type(&self) -> u32 {
        match self {
            OpcValue::Int8(_) => VT_I1,
            OpcValue::UInt8(_) => VT_UI1,
            OpcValue::Int16(_) => VT_I2,
            OpcValue::UInt16(_) => VT_UI2,
            OpcValue::Int32(_) => VT_I4,
            OpcValue::UInt32(_) => VT_UI4,
            OpcValue::Int64(_) => VT_I8,
            OpcValue::UInt64(_) => VT_UI8,
            OpcValue::INT(_) => VT_INT,
            OpcValue::UINT(_) => VT_UINT,
            OpcValue::Float(_) => VT_R4,
            OpcValue::Double(_) => VT_R8,
            OpcValue::Bool(_) => VT_BOOL,
            OpcValue::Cy(_) => VT_CY,
            OpcValue::Decimal(_) => VT_DECIMAL,
            OpcValue::Date(_) => VT_DATE,
            OpcValue::String(_) => VT_BSTR,
            OpcValue::ArrayInt16(_) => VT_ARRAY | VT_I2,
            OpcValue::ArrayUInt16(_) => VT_ARRAY | VT_UI2,
            OpcValue::ArrayInt32(_) => VT_ARRAY | VT_I4,
            OpcValue::ArrayUInt32(_) => VT_ARRAY | VT_UI4,
            OpcValue::ArrayInt64(_) => VT_ARRAY | VT_I8,
            OpcValue::ArrayUInt64(_) => VT_ARRAY | VT_UI8,
            OpcValue::ArrayFloat(_) => VT_ARRAY | VT_R4,
            OpcValue::ArrayDouble(_) => VT_ARRAY | VT_R8,
            OpcValue::ArrayBool(_) => VT_ARRAY | VT_BOOL,
            OpcValue::ArrayString(_) => VT_ARRAY | VT_BSTR,
        }
    }
    
    /// Create from raw value and type
    /// value_type is Windows VARTYPE (VARENUM value)
    /// free_string_memory: if true, free allocated string memory after copying (for async callbacks)
    pub fn from_raw(value: *mut std::ffi::c_void, value_type: u32, free_string_memory: bool) -> Result<Self, OpcValueError> {
        // Handle array types
        if value_type & VT_ARRAY != 0 {
            #[cfg(not(windows))]
            {
                return Err(OpcValueError::conversion_error(format!("Array type 0x{:x} not supported on non-Windows platform", value_type)));
            }
            #[cfg(windows)]
            {
                use windows::Win32::System::Ole::{SafeArrayAccessData, SafeArrayGetDim, SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayLock, SafeArrayUnaccessData, SafeArrayUnlock};
                
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null SAFEARRAY pointer"));
                }
                
                let sa = value as *mut olecom::SAFEARRAY;
                unsafe {
                    // Check dimensions (only support 1-dimensional arrays for now)
                    let dims = SafeArrayGetDim(sa);
                    if dims != 1 {
                        return Err(OpcValueError::conversion_error(format!("Multi-dimensional arrays not supported (dims={})", dims)));
                    }
                    
                    // Get bounds
                    let lower_bound = match SafeArrayGetLBound(sa, 1) {
                        Ok(bound) => bound,
                        Err(_) => return Err(OpcValueError::conversion_error("Failed to get array lower bound")),
                    };
                    let upper_bound = match SafeArrayGetUBound(sa, 1) {
                        Ok(bound) => bound,
                        Err(_) => return Err(OpcValueError::conversion_error("Failed to get array upper bound")),
                    };
                    
                    let element_count = (upper_bound - lower_bound + 1) as usize;
                    if element_count == 0 {
                        // Empty array
                        return match value_type & VT_TYPEMASK {
                            VT_I2 => Ok(OpcValue::ArrayInt16(Vec::new())),
                            VT_UI2 => Ok(OpcValue::ArrayUInt16(Vec::new())),
                            VT_I4 => Ok(OpcValue::ArrayInt32(Vec::new())),
                            VT_UI4 => Ok(OpcValue::ArrayUInt32(Vec::new())),
                            VT_I8 => Ok(OpcValue::ArrayInt64(Vec::new())),
                            VT_UI8 => Ok(OpcValue::ArrayUInt64(Vec::new())),
                            VT_R4 => Ok(OpcValue::ArrayFloat(Vec::new())),
                            VT_R8 => Ok(OpcValue::ArrayDouble(Vec::new())),
                            VT_BOOL => Ok(OpcValue::ArrayBool(Vec::new())),
                            VT_BSTR => Ok(OpcValue::ArrayString(Vec::new())),
                            _ => Err(OpcValueError::conversion_error(format!("Unsupported array element type: 0x{:x}", value_type & VT_TYPEMASK))),
                        };
                    }
                    
                    // Access array data (locks the array)
                    let mut p_data = std::ptr::null_mut();
                    if SafeArrayAccessData(sa, &mut p_data as *mut *mut std::ffi::c_void).is_err() {
                        return Err(OpcValueError::conversion_error("Failed to access SAFEARRAY data"));
                    }
                    
                    // Create vector based on element type
                    let result = match value_type & VT_TYPEMASK {
                        VT_I2 => {
                            let slice = std::slice::from_raw_parts(p_data as *const i16, element_count);
                            OpcValue::ArrayInt16(slice.to_vec())
                        }
                        VT_UI2 => {
                            let slice = std::slice::from_raw_parts(p_data as *const u16, element_count);
                            OpcValue::ArrayUInt16(slice.to_vec())
                        }
                        VT_I4 => {
                            let slice = std::slice::from_raw_parts(p_data as *const i32, element_count);
                            OpcValue::ArrayInt32(slice.to_vec())
                        }
                        VT_UI4 => {
                            let slice = std::slice::from_raw_parts(p_data as *const u32, element_count);
                            OpcValue::ArrayUInt32(slice.to_vec())
                        }
                        VT_I8 => {
                            let slice = std::slice::from_raw_parts(p_data as *const i64, element_count);
                            OpcValue::ArrayInt64(slice.to_vec())
                        }
                        VT_UI8 => {
                            let slice = std::slice::from_raw_parts(p_data as *const u64, element_count);
                            OpcValue::ArrayUInt64(slice.to_vec())
                        }
                        VT_R4 => {
                            let slice = std::slice::from_raw_parts(p_data as *const f32, element_count);
                            OpcValue::ArrayFloat(slice.to_vec())
                        }
                        VT_R8 => {
                            let slice = std::slice::from_raw_parts(p_data as *const f64, element_count);
                            OpcValue::ArrayDouble(slice.to_vec())
                        }
                        VT_BOOL => {
                            let slice = std::slice::from_raw_parts(p_data as *const i16, element_count);
                            OpcValue::ArrayBool(slice.iter().map(|&v| v != 0).collect())
                        }
                        VT_BSTR => {
                            // Array of BSTR strings
                            let slice = std::slice::from_raw_parts(p_data as *const *const u16, element_count);
                            let strings: Vec<String> = slice.iter().map(|&bstr| {
                                if bstr.is_null() {
                                    String::new()
                                } else {
                                    // Convert BSTR to String
                                    let mut len = 0;
                                    while *bstr.add(len) != 0 {
                                        len += 1;
                                    }
                                    let slice = std::slice::from_raw_parts(bstr, len);
                                    String::from_utf16_lossy(slice)
                                }
                            }).collect();
                            // Free allocated BSTR memory if requested (async callbacks)
                            if free_string_memory {
                                for &bstr in slice.iter() {
                                    if !bstr.is_null() {
                                        unsafe {
                                            crate::ffi::opc_free_string(bstr as *mut u16);
                                        }
                                    }
                                }
                            }
                            OpcValue::ArrayString(strings)
                        }
                        _ => {
                            let _ = SafeArrayUnaccessData(sa);
                            return Err(OpcValueError::conversion_error(format!("Unsupported array element type: 0x{:x}", value_type & VT_TYPEMASK)));
                        }
                    };
                    
                    let _ = SafeArrayUnaccessData(sa);
                    return Ok(result)
                }
            }
        }
        
        // Handle byref types - value is a pointer to the actual data pointer
        let mut actual_value = value;
        let mut value_type_for_processing = value_type;
        
        if value_type & VT_BYREF != 0 {
            // For byref types, value points to a pointer that points to the actual data
            // We need to dereference once to get the actual data pointer
            if value.is_null() {
                return Err(OpcValueError::conversion_error("Null pointer for byref type"));
            }
            actual_value = unsafe { *(value as *const *mut std::ffi::c_void) };
            // Remove VT_BYREF flag for further processing
            value_type_for_processing = value_type & !VT_BYREF;
        }
        
        // Check for byref types (after handling VT_BYREF flag)
        let base_type = value_type_for_processing & VT_TYPEMASK;
        
        match base_type {
            VT_I1 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for I1"));
                }
                Ok(OpcValue::Int8(unsafe { *(value as *const i8) }))
            }
            VT_UI1 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for UI1"));
                }
                Ok(OpcValue::UInt8(unsafe { *(value as *const u8) }))
            }
            VT_I2 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Int16"));
                }
                Ok(OpcValue::Int16(unsafe { *(value as *const i16) }))
            }
            VT_UI2 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for UI2"));
                }
                Ok(OpcValue::UInt16(unsafe { *(value as *const u16) }))
            }
            VT_I4 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Int32"));
                }
                Ok(OpcValue::Int32(unsafe { *(value as *const i32) }))
            }
            VT_UI4 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for UI4"));
                }
                Ok(OpcValue::UInt32(unsafe { *(value as *const u32) }))
            }
            VT_I8 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for I8"));
                }
                Ok(OpcValue::Int64(unsafe { *(value as *const i64) }))
            }
            VT_UI8 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for UI8"));
                }
                Ok(OpcValue::UInt64(unsafe { *(value as *const u64) }))
            }
            VT_INT => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for INT"));
                }
                // INT is platform-dependent, assume i32 on Windows
                Ok(OpcValue::INT(unsafe { *(value as *const i32) } as isize))
            }
            VT_UINT => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for UINT"));
                }
                // UINT is platform-dependent, assume u32 on Windows
                Ok(OpcValue::UINT(unsafe { *(value as *const u32) } as usize))
            }
            VT_R4 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Float"));
                }
                Ok(OpcValue::Float(unsafe { *(value as *const f32) }))
            }
            VT_R8 => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Double"));
                }
                Ok(OpcValue::Double(unsafe { *(value as *const f64) }))
            }
            VT_BOOL => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Bool"));
                }
                // VARIANT_BOOL is short, where -1 is TRUE and 0 is FALSE
                let raw = unsafe { *(value as *const i16) };
                Ok(OpcValue::Bool(raw != 0))
            }
            VT_CY => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Cy"));
                }
                // CY is 64-bit integer scaled by 10000
                Ok(OpcValue::Cy(unsafe { *(value as *const i64) }))
            }
            VT_DATE => {
                if value.is_null() {
                    return Err(OpcValueError::conversion_error("Null pointer for Date"));
                }
                // DATE is f64 (OLE automation date)
                Ok(OpcValue::Date(unsafe { *(value as *const f64) }))
            }
            VT_BSTR => {
                if value.is_null() {
                    return Ok(OpcValue::String(String::new()));
                }
                // BSTR is a pointer to a null-terminated wide string (UTF-16)
                // BSTR has a length prefix but we can treat it as null-terminated
                let wide_ptr = value as *const u16;
                let result = unsafe {
                    // Find null terminator
                    let mut len = 0;
                    while *wide_ptr.add(len) != 0 {
                        len += 1;
                    }
                    // Create slice and convert to String
                    let slice = std::slice::from_raw_parts(wide_ptr, len);
                    String::from_utf16_lossy(slice)
                };
                // Free allocated memory if requested (async callbacks)
                if free_string_memory {
                    unsafe {
                        crate::ffi::opc_free_string(value as *mut u16);
                    }
                }
                Ok(OpcValue::String(result))
            }
            VT_LPSTR => {
                if value.is_null() {
                    return Ok(OpcValue::String(String::new()));
                }
                // LPSTR is a pointer to a null-terminated ANSI string (char*)
                let ansi_ptr = value as *const i8;
                let result = unsafe {
                    // Convert C string to Rust String
                    let c_str = std::ffi::CStr::from_ptr(ansi_ptr);
                    c_str.to_string_lossy().into_owned()
                };
                // Free allocated memory if requested (async callbacks)
                if free_string_memory {
                    unsafe {
                        crate::ffi::opc_free_string_ansi(value as *mut i8);
                    }
                }
                Ok(OpcValue::String(result))
            }
            VT_LPWSTR => {
                if value.is_null() {
                    return Ok(OpcValue::String(String::new()));
                }
                // LPWSTR is a pointer to a null-terminated wide string (UTF-16)
                let wide_ptr = value as *const u16;
                let result = unsafe {
                    // Find null terminator
                    let mut len = 0;
                    while *wide_ptr.add(len) != 0 {
                        len += 1;
                    }
                    // Create slice and convert to String
                    let slice = std::slice::from_raw_parts(wide_ptr, len);
                    String::from_utf16_lossy(slice)
                };
                // Free allocated memory if requested (async callbacks)
                if free_string_memory {
                    unsafe {
                        crate::ffi::opc_free_string(value as *mut u16);
                    }
                }
                Ok(OpcValue::String(result))
            }
            VT_DECIMAL => {
                if value.is_null() {
                    return Ok(OpcValue::Decimal("0".to_string()));
                }
                let decimal_ptr = value as *const Decimal;
                let decimal = unsafe { &*decimal_ptr };
                Ok(OpcValue::Decimal(decimal.to_string()))
            }
            _ => {
                // For unsupported types, return as string serialized
                // In a real implementation, we would convert the VARIANT to string
                Err(OpcValueError::InvalidValueType(value_type))
            }
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

    #[test]
    fn test_opc_value_from_raw_numeric() {
        use super::*;
        
        let val: i8 = -42;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_I1, false).unwrap();
        assert_eq!(result, OpcValue::Int8(val));
        unsafe { drop(Box::from_raw(ptr as *mut i8)); }
        
        let val: u8 = 200;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_UI1, false).unwrap();
        assert_eq!(result, OpcValue::UInt8(val));
        unsafe { drop(Box::from_raw(ptr as *mut u8)); }
        
        let val: i16 = -1234;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_I2, false).unwrap();
        assert_eq!(result, OpcValue::Int16(val));
        unsafe { drop(Box::from_raw(ptr as *mut i16)); }
        
        let val: u16 = 4567;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_UI2, false).unwrap();
        assert_eq!(result, OpcValue::UInt16(val));
        unsafe { drop(Box::from_raw(ptr as *mut u16)); }
        
        let val: i32 = -98765;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_I4, false).unwrap();
        assert_eq!(result, OpcValue::Int32(val));
        unsafe { drop(Box::from_raw(ptr as *mut i32)); }
        
        let val: u32 = 123456;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_UI4, false).unwrap();
        assert_eq!(result, OpcValue::UInt32(val));
        unsafe { drop(Box::from_raw(ptr as *mut u32)); }
        
        let val: i64 = -999999999;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_I8, false).unwrap();
        assert_eq!(result, OpcValue::Int64(val));
        unsafe { drop(Box::from_raw(ptr as *mut i64)); }
        
        let val: u64 = 9999999999;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_UI8, false).unwrap();
        assert_eq!(result, OpcValue::UInt64(val));
        unsafe { drop(Box::from_raw(ptr as *mut u64)); }
        
        let val: i32 = -111;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_INT, false).unwrap();
        assert_eq!(result, OpcValue::INT(val as isize));
        unsafe { drop(Box::from_raw(ptr as *mut i32)); }
        
        let val: u32 = 222;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_UINT, false).unwrap();
        assert_eq!(result, OpcValue::UINT(val as usize));
        unsafe { drop(Box::from_raw(ptr as *mut u32)); }
        
        let val: f32 = 3.14159;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_R4, false).unwrap();
        match result {
            OpcValue::Float(v) => assert!((v - val).abs() < 0.0001),
            _ => panic!("Expected Float"),
        }
        unsafe { drop(Box::from_raw(ptr as *mut f32)); }
        
        let val: f64 = 2.71828;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_R8, false).unwrap();
        match result {
            OpcValue::Double(v) => assert!((v - val).abs() < 0.0001),
            _ => panic!("Expected Double"),
        }
        unsafe { drop(Box::from_raw(ptr as *mut f64)); }
        
        let val: i16 = -1;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_BOOL, false).unwrap();
        assert_eq!(result, OpcValue::Bool(true));
        unsafe { drop(Box::from_raw(ptr as *mut i16)); }
        
        let val: i16 = 0;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_BOOL, false).unwrap();
        assert_eq!(result, OpcValue::Bool(false));
        unsafe { drop(Box::from_raw(ptr as *mut i16)); }
        
        let val: i64 = 1234567890;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_CY, false).unwrap();
        assert_eq!(result, OpcValue::Cy(val));
        unsafe { drop(Box::from_raw(ptr as *mut i64)); }
        
        let val: f64 = 45123.456;
        let ptr = Box::into_raw(Box::new(val)) as *mut std::ffi::c_void;
        let result = OpcValue::from_raw(ptr, VT_DATE, false).unwrap();
        match result {
            OpcValue::Date(v) => assert!((v - val).abs() < 0.0001),
            _ => panic!("Expected Date"),
        }
        unsafe { drop(Box::from_raw(ptr as *mut f64)); }
    }
}