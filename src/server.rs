//! OPC Server representation and operations

use std::ptr;
use crate::error::{OpcError, OpcResult};
use crate::group::OpcGroup;
use crate::utils;

/// Represents a connection to an OPC server
pub struct OpcServer {
    ptr: *mut std::ffi::c_void,
    host_ptr: *mut std::ffi::c_void,
}

impl OpcServer {
    /// Create a new server instance (internal use)
    pub(crate) fn new(server_ptr: *mut std::ffi::c_void, host_ptr: *mut std::ffi::c_void) -> Self {
        OpcServer {
            ptr: server_ptr,
            host_ptr,
        }
    }
    
    /// Get the server status and vendor information
    pub fn get_status(&self) -> OpcResult<(u32, String)> {
        let mut state: u32 = 0;
        let mut vendor_info_ptr: *mut u16 = ptr::null_mut();
        
        let result = unsafe {
            crate::ffi::opc_server_get_status(self.ptr, &mut state, &mut vendor_info_ptr)
        };
        
        if result == 0 {
            let vendor_info = if !vendor_info_ptr.is_null() {
                let info = utils::from_wide_string(vendor_info_ptr);
                utils::free_wide_string(vendor_info_ptr);
                info
            } else {
                String::new()
            };
            
            Ok((state, vendor_info))
        } else {
            Err(OpcError::operation_failed("Failed to get server status"))
        }
    }
    
    /// Create a new OPC group
    pub fn create_group(
        &self,
        name: &str,
        active: bool,
        requested_update_rate: u32,
        deadband: f64,
    ) -> OpcResult<OpcGroup> {
        let group_name_wide = utils::to_wide_string(name);
        let mut actual_update_rate: u32 = 0;
        let mut group_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        let result = unsafe {
            crate::ffi::opc_server_make_group(
                self.ptr,
                group_name_wide.as_ptr(),
                if active { 1 } else { 0 },
                requested_update_rate,
                &mut actual_update_rate,
                deadband,
                &mut group_ptr,
            )
        };
        
        if result == 0 && !group_ptr.is_null() {
            Ok(OpcGroup::new(group_ptr))
        } else {
            Err(OpcError::GroupCreationFailed(
                format!("Failed to create group '{}'", name)
            ))
        }
    }
    
    /// Get all available item names from the server
    pub fn get_item_names(&self) -> OpcResult<Vec<String>> {
        let mut item_names_ptr: *mut *mut u16 = ptr::null_mut();
        let mut count: u32 = 0;
        
        let result = unsafe {
            crate::ffi::opc_server_get_item_names(self.ptr, &mut item_names_ptr, &mut count)
        };
        
        if result == 0 && !item_names_ptr.is_null() {
            let mut items = Vec::with_capacity(count as usize);
            
            unsafe {
                for i in 0..count {
                    let item_name_ptr = *item_names_ptr.add(i as usize);
                    if !item_name_ptr.is_null() {
                        let item_name = utils::from_wide_string(item_name_ptr);
                        items.push(item_name);
                    }
                }
                
                utils::free_wide_string_array(item_names_ptr, count);
            }
            
            Ok(items)
        } else {
            Err(OpcError::operation_failed("Failed to get item names"))
        }
    }
    
    /// Get the raw server pointer (for internal use)
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr
    }
}

impl Drop for OpcServer {
    fn drop(&mut self) {
        unsafe {
            crate::ffi::opc_server_free(self.ptr);
            crate::ffi::opc_host_free(self.host_ptr);
        }
    }
}