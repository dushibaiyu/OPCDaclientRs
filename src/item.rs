//! OPC 项模块
//! 
//! 这个模块提供了 OPC 项的表示和操作功能。
//! `OpcItem` 表示单个 OPC 数据点，可以进行读取和写入操作。
//! 
//! ## 主要功能
//! 
//! - 同步读取项值
//! - 同步写入项值
//! - 异步读取项值
//! - 异步写入项值
//! - 管理项生命周期
//! 
//! ## 项属性
//! 
//! 每个 OPC 项具有以下属性：
//! - 名称：项的完整路径，如 "设备名.变量名"
//! - 值：项的当前值，支持多种数据类型
//! - 质量：值的质量状态（良好、不确定、错误）
//! - 时间戳：值最后更新的时间
//! 
//! ## 数据类型
//! 
//! OPC 项支持多种数据类型，包括：
//! - 整数（Int16, Int32）
//! - 浮点数（Float, Double）
//! - 字符串（String）
//! - 布尔值（Boolean）
//! - 时间（DateTime）

use crate::error::{OpcError, OpcResult};
use crate::types::{OpcValue, OpcQuality};

/// OPC 项，表示单个数据点
/// 
/// 项是 OPC 数据访问的基本单位，表示一个可读写的变量。
/// 通过项可以：
/// 1. 读取当前值和质量
/// 2. 写入新值
/// 3. 进行同步或异步操作
/// 
/// ## 内部结构
/// 
/// - `ptr`: 指向底层 OPC 项对象的指针
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::{OpcClient, OpcValue};
/// 
/// let client = OpcClient::new()?;
/// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
/// let group = server.create_group("TestGroup", true, 1000, 0.0)?;
/// let item = group.add_item("Bucket Brigade.UInt2")?;
/// 
/// // 同步读取
/// let (value, quality) = item.read_sync()?;
/// println!("值: {:?}, 质量: {:?}", value, quality);
/// 
/// // 同步写入
/// item.write_sync(&OpcValue::Int32(100))?;
/// ```
pub struct OpcItem {
    /// 指向底层 OPC 项对象的指针
    ptr: *mut std::ffi::c_void,
}

impl OpcItem {
    /// 创建新的项实例（内部使用）
    /// 
    /// # 参数
    /// - `item_ptr`: 指向底层 OPC 项对象的指针
    /// 
    /// # 注意
    /// 这个方法仅供内部使用，用户应该通过 `OpcGroup::add_item` 获取 `OpcItem` 实例。
    pub(crate) fn new(item_ptr: *mut std::ffi::c_void) -> Self {
        OpcItem {
            ptr: item_ptr,
        }
    }
    
    /// 同步读取项值
    /// 
    /// 这个方法阻塞当前线程，直到从服务器读取到项的值和质量。
    /// 
    /// # 返回值
    /// - `Ok((value, quality, timestamp))`: 成功读取值、质量和时间戳
    ///   - `value`: 项的值，类型为 `OpcValue`
    ///   - `quality`: 值的质量，类型为 `OpcQuality`
    ///   - `timestamp`: 时间戳，Unix毫秒，类型为 `u64`
    /// - `Err(OpcError)`: 读取失败，可能的原因包括：
    ///   - 项不可读
    ///   - 服务器连接中断
    ///   - 权限不足
    ///   - 数据类型转换失败
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::{OpcClient, OpcValue};
    /// 
    /// let client = OpcClient::new()?;
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// let group = server.create_group("TestGroup", true, 1000, 0.0)?;
    /// let item = group.add_item("Bucket Brigade.UInt2")?;
    /// 
    /// match item.read_sync() {
    ///     Ok((value, quality, timestamp)) => {
    ///         println!("读取成功: 值 = {:?}, 质量 = {:?}, 时间戳 = {} ms", value, quality, timestamp);
    ///         // 可以将值转换为具体类型
    ///         if let Ok(int_value) = i32::try_from(value) {
    ///             println!("整数值: {}", int_value);
    ///         }
    ///     }
    ///     Err(e) => println!("读取失败: {}", e),
    /// }
    /// ```
    /// 
    /// # 注意
    /// - 这是阻塞操作，在慢速网络上可能会有延迟
    /// - 返回的值需要根据类型进行转换
    /// - 质量指示数据的可靠性
    pub fn read_sync(&self) -> OpcResult<(OpcValue, OpcQuality, u64)> {
        // 创建临时缓冲区存储值（64字节足够大多数类型）
        let mut temp_buffer: [u8; 64] = [0; 64];
        let mut quality: i32 = 0;
        let mut value_type: u32 = 0;
        let mut timestamp_ms: u64 = 0;
        
        // 调用 FFI 函数同步读取
        let result = unsafe {
            crate::ffi::opc_item_read_sync(
                self.ptr,
                temp_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                &mut quality,
                &mut value_type,
                &mut timestamp_ms,
            )
        };
        
        if result == 0 {
            // 将原始值转换为 OpcValue
            let opc_value = OpcValue::from_raw(
                temp_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                value_type,
                false, // sync read: free_allocated_string_memory will handle freeing
            )?;
            
            // 将原始质量转换为 OpcQuality
            let opc_quality = OpcQuality::from_raw(quality);
            
            // 对于字符串类型，需要释放 C++ 分配的内存
            // C++ 的 opc_item_read_sync() 为字符串分配了新内存
            // 我们需要在转换后释放它
            Self::free_allocated_string_memory(&mut temp_buffer, value_type);
            
            Ok((opc_value, opc_quality, timestamp_ms))
        } else {
            Err(OpcError::operation_failed("Failed to read item synchronously"))
        }
    }
    
    fn free_allocated_string_memory(temp_buffer: &mut [u8; 64], value_type: u32) {
        const VT_BSTR: u32 = 8;
        const VT_LPSTR: u32 = 30;
        const VT_LPWSTR: u32 = 31;
        
        match value_type {
            VT_BSTR | VT_LPWSTR => {
                let ptr_ptr = temp_buffer.as_mut_ptr() as *mut *mut std::ffi::c_void;
                unsafe {
                    let allocated_ptr = *ptr_ptr;
                    if !allocated_ptr.is_null() {
                        let wstr_ptr = allocated_ptr as *mut u16;
                        crate::ffi::opc_free_string(wstr_ptr);
                    }
                }
            }
            VT_LPSTR => {
                let ptr_ptr = temp_buffer.as_mut_ptr() as *mut *mut std::ffi::c_void;
                unsafe {
                    let allocated_ptr = *ptr_ptr;
                    if !allocated_ptr.is_null() {
                        let ansi_ptr = allocated_ptr as *mut i8;
                        crate::ffi::opc_free_string_ansi(ansi_ptr);
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Write item value synchronously
    pub fn write_sync(&self, value: &OpcValue) -> OpcResult<()> {
        // Temporary holders for string data to keep them alive during FFI call
        let mut _wide_holder: Option<Vec<u16>> = None;
        let mut _ansi_holder: Option<std::ffi::CString> = None;
        let (value_ptr, value_type) = match value {
            // Numeric types
            OpcValue::Int8(v) => (v as *const i8 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt8(v) => (v as *const u8 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int16(v) => (v as *const i16 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt16(v) => (v as *const u16 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int32(v) => (v as *const i32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt32(v) => (v as *const u32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int64(v) => (v as *const i64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt64(v) => (v as *const u64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::INT(v) => (v as *const isize as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UINT(v) => (v as *const usize as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Float(v) => (v as *const f32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Double(v) => (v as *const f64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Bool(v) => (v as *const bool as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Cy(v) => (v as *const i64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Date(v) => (v as *const f64 as *const std::ffi::c_void, value.raw_type()),
            
            // String types - need special handling
            OpcValue::String(s) => {
                let wide = crate::to_wide_string(s);
                let ptr = wide.as_ptr();
                _wide_holder = Some(wide);
                let ptr_ptr: *const *const u16 = &ptr;
                (ptr_ptr as *const std::ffi::c_void, value.raw_type())
            }
            // Decimal type - not implemented
            OpcValue::Decimal(_) => {
                return Err(OpcError::operation_failed("Decimal writes not implemented"));
            }
            
            // Array types - not implemented
            OpcValue::ArrayInt16(_) | OpcValue::ArrayUInt16(_) | OpcValue::ArrayInt32(_) |
            OpcValue::ArrayUInt32(_) | OpcValue::ArrayInt64(_) | OpcValue::ArrayUInt64(_) |
            OpcValue::ArrayFloat(_) | OpcValue::ArrayDouble(_) | OpcValue::ArrayBool(_) |
            OpcValue::ArrayString(_) => {
                return Err(OpcError::operation_failed("Array writes not implemented"));
            }
        };
        
        let result = unsafe {
            crate::ffi::opc_item_write_sync(self.ptr, value_ptr, value_type)
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(OpcError::operation_failed("Failed to write item synchronously"))
        }
    }
    
    /// Read item value asynchronously
    pub fn read_async(&self) -> OpcResult<()> {
        let result = unsafe {
            crate::ffi::opc_item_read_async(self.ptr)
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(OpcError::operation_failed("Failed to read item asynchronously"))
        }
    }
    
    /// Write item value asynchronously
    pub fn write_async(&self, value: &OpcValue) -> OpcResult<()> {
        // Temporary holders for string data to keep them alive during FFI call
        let mut _wide_holder: Option<Vec<u16>> = None;
        let mut _ansi_holder: Option<std::ffi::CString> = None;
        let (value_ptr, value_type) = match value {
            // Numeric types
            OpcValue::Int8(v) => (v as *const i8 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt8(v) => (v as *const u8 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int16(v) => (v as *const i16 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt16(v) => (v as *const u16 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int32(v) => (v as *const i32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt32(v) => (v as *const u32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int64(v) => (v as *const i64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UInt64(v) => (v as *const u64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::INT(v) => (v as *const isize as *const std::ffi::c_void, value.raw_type()),
            OpcValue::UINT(v) => (v as *const usize as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Float(v) => (v as *const f32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Double(v) => (v as *const f64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Bool(v) => (v as *const bool as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Cy(v) => (v as *const i64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Date(v) => (v as *const f64 as *const std::ffi::c_void, value.raw_type()),
            
            // String types - need special handling
            OpcValue::String(s) => {
                let wide = crate::to_wide_string(s);
                let ptr = wide.as_ptr();
                _wide_holder = Some(wide);
                let ptr_ptr: *const *const u16 = &ptr;
                (ptr_ptr as *const std::ffi::c_void, value.raw_type())
            }
        
            // Decimal type - not implemented
            OpcValue::Decimal(_) => {
                return Err(OpcError::operation_failed("Decimal writes not implemented"));
            }
            
            // Array types - not implemented
            OpcValue::ArrayInt16(_) | OpcValue::ArrayUInt16(_) | OpcValue::ArrayInt32(_) |
            OpcValue::ArrayUInt32(_) | OpcValue::ArrayInt64(_) | OpcValue::ArrayUInt64(_) |
            OpcValue::ArrayFloat(_) | OpcValue::ArrayDouble(_) | OpcValue::ArrayBool(_) |
            OpcValue::ArrayString(_) => {
                return Err(OpcError::operation_failed("Array writes not implemented"));
            }
        };
        
        let result = unsafe {
            crate::ffi::opc_item_write_async(self.ptr, value_ptr, value_type)
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(OpcError::operation_failed("Failed to write item asynchronously"))
        }
    }
    
    /// Get the raw item pointer (for internal use)
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr
    }
}

impl Drop for OpcItem {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::opc_item_free(self.ptr);
        }
    }
}