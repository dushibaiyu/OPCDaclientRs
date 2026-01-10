//! OPC Group representation and operations

use std::ptr;
use std::sync::Arc;
use crate::error::{OpcError, OpcResult};
use crate::item::OpcItem;
use crate::types::{OpcValue, OpcQuality, OpcDataCallback, OpcCallbackContainer};
use crate::utils;

/// Represents an OPC group containing items
pub struct OpcGroup {
    ptr: *mut std::ffi::c_void,
}

impl OpcGroup {
    /// Create a new group instance (internal use)
    pub(crate) fn new(group_ptr: *mut std::ffi::c_void) -> Self {
        OpcGroup {
            ptr: group_ptr,
        }
    }
    
    /// Add an item to the group
    pub fn add_item(&self, name: &str) -> OpcResult<OpcItem> {
        let item_name_wide = utils::to_wide_string(name);
        let mut item_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        let result = unsafe {
            crate::ffi::opc_group_add_item(self.ptr, item_name_wide.as_ptr(), &mut item_ptr)
        };
        
        if result == 0 && !item_ptr.is_null() {
            Ok(OpcItem::new(item_ptr))
        } else {
            Err(OpcError::ItemNotFound(
                format!("Failed to add item '{}' to group", name)
            ))
        }
    }
    
    /// Enable asynchronous data change notifications
    pub fn enable_async_subscription(&self, callback: Arc<dyn OpcDataCallback>) -> OpcResult<()> {
        let container = Box::into_raw(Box::new(OpcCallbackContainer {
            callback,
        }));
        
        let result = unsafe {
            crate::ffi::opc_group_enable_async(
                self.ptr,
                opc_data_change_callback,
                container as *mut std::ffi::c_void,
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            // Clean up if enabling failed
            unsafe {
                let _ = Box::from_raw(container);
            }
            Err(OpcError::AsyncSubscriptionFailed(
                "Failed to enable async subscription".to_string()
            ))
        }
    }
    
    /// Refresh all items in the group
    pub fn refresh(&self) -> OpcResult<()> {
        let result = unsafe {
            crate::ffi::opc_group_refresh(self.ptr)
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(OpcError::operation_failed("Failed to refresh group"))
        }
    }
    
    /// Read item value synchronously
    pub fn read_sync(&self, item: &OpcItem) -> OpcResult<(OpcValue, OpcQuality)> {
        item.read_sync()
    }
    
    /// Write item value synchronously
    pub fn write_sync(&self, item: &OpcItem, value: &OpcValue) -> OpcResult<()> {
        item.write_sync(value)
    }
    
    
    /// Get the raw group pointer (for internal use)
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr
    }
}

impl Drop for OpcGroup {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::opc_group_free(self.ptr);
        }
    }
}

/// Internal callback function for FFI
extern "C" fn opc_data_change_callback(
    user_data: *mut std::ffi::c_void,
    group_name: *const u16,
    item_name: *const u16,
    value: *mut std::ffi::c_void,
    quality: i32,
    value_type: u32,
) {
    if user_data.is_null() {
        return;
    }
    
    // Get the callback container
    let container = unsafe { &*(user_data as *const OpcCallbackContainer) };
    
    // Extract the names
    let group_name_str = utils::from_wide_string(group_name);
    let item_name_str = utils::from_wide_string(item_name);
    
    // Convert value and quality
    let opc_value = match OpcValue::from_raw(value, value_type) {
        Ok(value) => value,
        Err(_) => OpcValue::Int32(0), // Default fallback on error
    };
    
    let opc_quality = OpcQuality::from_raw(quality);
    
    // Call the user-provided callback
    container.callback.on_data_change(&group_name_str, &item_name_str, opc_value, opc_quality);
}