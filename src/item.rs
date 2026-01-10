//! OPC Item representation and operations


use crate::error::{OpcError, OpcResult};
use crate::types::{OpcValue, OpcQuality};

/// Represents an OPC item that can be read/written
pub struct OpcItem {
    ptr: *mut std::ffi::c_void,
}

impl OpcItem {
    /// Create a new item instance (internal use)
    pub(crate) fn new(item_ptr: *mut std::ffi::c_void) -> Self {
        OpcItem {
            ptr: item_ptr,
        }
    }
    
    /// Read item value synchronously
    pub fn read_sync(&self) -> OpcResult<(OpcValue, OpcQuality)> {
        // We need temporary storage for the value
        let mut temp_buffer: [u8; 64] = [0; 64]; // Large enough for most types
        let mut quality: i32 = 0;
        let mut value_type: u32 = 0;
        
        let result = unsafe {
            crate::ffi::opc_item_read_sync(
                self.ptr,
                temp_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                &mut quality,
                &mut value_type,
            )
        };
        
        if result == 0 {
            let opc_value = OpcValue::from_raw(
                temp_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                value_type,
            )?;
            
            let opc_quality = OpcQuality::from_raw(quality);
            
            Ok((opc_value, opc_quality))
        } else {
            Err(OpcError::operation_failed("Failed to read item synchronously"))
        }
    }
    
    /// Write item value synchronously
    pub fn write_sync(&self, value: &OpcValue) -> OpcResult<()> {
        let (value_ptr, value_type) = match value {
            OpcValue::Int16(v) => (v as *const i16 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int32(v) => (v as *const i32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Float(v) => (v as *const f32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Double(v) => (v as *const f64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::String(_) => {
                // String handling would be more complex
                return Err(OpcError::operation_failed("String writes not implemented in sync"));
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
        let (value_ptr, value_type) = match value {
            OpcValue::Int16(v) => (v as *const i16 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Int32(v) => (v as *const i32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Float(v) => (v as *const f32 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::Double(v) => (v as *const f64 as *const std::ffi::c_void, value.raw_type()),
            OpcValue::String(_) => {
                // String handling would be more complex
                return Err(OpcError::operation_failed("String writes not implemented in async"));
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