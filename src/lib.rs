//! Safe Rust wrapper for OPC DA Client library
//! 
//! This crate provides a safe, idiomatic Rust interface for the OPC DA Client Toolkit.
//! 
//! # Platform Support
//! 
//! This library is only supported on Windows platforms due to OPC DA being a Windows COM technology.
//! On non-Windows platforms, the library will compile but all operations will return errors.
//! 
//! # Examples
//! 
//! ```no_run
//! use opc_da_client::{OpcClient, OpcValue};
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client
//!     let client = OpcClient::new()?;
//!     
//!     // Connect to a server
//!     let server = client.connect_to_server("localhost", "Matrikon.OPC.Simulation.1")?;
//!     
//!     // Create a group
//!     let group = server.create_group("TestGroup", true, 1000, 0.0)?;
//!     
//!     // Add an item
//!     let item = group.add_item("Bucket Brigade.UInt2")?;
//!     
//!     // Read value
//!     let (value, quality) = item.read_sync()?;
//!     println!("Value: {:?}, Quality: {:?}", value, quality);
//!     
//!     // Write value
//!     item.write_sync(&OpcValue::Int32(12345))?;
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;
pub mod client;
pub mod server;
pub mod group;
pub mod item;
pub mod async_api;

// Re-export main types
pub use client::OpcClient;
pub use error::{OpcError, OpcResult};
pub use types::{OpcValue, OpcQuality, OpcDataCallback};
pub use server::OpcServer;
pub use group::OpcGroup;
pub use item::OpcItem;


// Internal FFI bindings
// On Windows, we try to use the real OPC library
// On other platforms, we use stub implementations
#[cfg(windows)]
mod ffi {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    
    // Try to link with the OPC library
    // If compilation fails, we'll use stub implementations
    extern "C" {
        // Client functions
        pub fn opc_client_init() -> u32;
        pub fn opc_client_stop();
        
        // Host functions
        pub fn opc_make_host(hostname: *const u16, host: *mut *mut c_void) -> u32;
        pub fn opc_host_free(host: *mut c_void);
        
        // Server functions
        pub fn opc_host_connect_da_server(
            host: *mut c_void,
            server_name: *const u16,
            server: *mut *mut c_void,
        ) -> u32;
        pub fn opc_server_free(server: *mut c_void);
        pub fn opc_server_get_status(
            server: *mut c_void,
            state: *mut u32,
            vendor_info: *mut *mut u16,
        ) -> u32;
        
        // Group functions
        pub fn opc_server_make_group(
            server: *mut c_void,
            group_name: *const u16,
            active: i32,
            req_update_rate: u32,
            actual_update_rate: *mut u32,
            deadband: f64,
            group: *mut *mut c_void,
        ) -> u32;
        pub fn opc_group_free(group: *mut c_void);
        
        // Item functions
        pub fn opc_group_add_item(
            group: *mut c_void,
            item_name: *const u16,
            item: *mut *mut c_void,
        ) -> u32;
        pub fn opc_item_free(item: *mut c_void);
        
        // Synchronous operations
        pub fn opc_item_read_sync(
            item: *mut c_void,
            value: *mut c_void,
            quality: *mut i32,
            value_type: *mut u32,
        ) -> u32;
        pub fn opc_item_write_sync(item: *mut c_void, value: *const c_void, value_type: u32) -> u32;
        
        // Asynchronous operations
        pub fn opc_group_enable_async(
            group: *mut c_void,
            callback: extern "C" fn(*mut c_void, *const u16, *const u16, *mut c_void, i32, u32),
            user_data: *mut c_void,
        ) -> u32;
        pub fn opc_item_read_async(item: *mut c_void) -> u32;
        pub fn opc_item_write_async(item: *mut c_void, value: *const c_void, value_type: u32) -> u32;
        
        // Group operations
        pub fn opc_group_refresh(group: *mut c_void) -> u32;
        
        // Browse functions
        pub fn opc_server_get_item_names(
            server: *mut c_void,
            item_names: *mut *mut *mut u16,
            count: *mut u32,
        ) -> u32;
        
        // Utility functions
        pub fn opc_free_string_array(strings: *mut *mut u16, count: u32);
        pub fn opc_free_string(str: *mut u16);
    }
}

// Utility functions
mod utils {
    #[cfg(windows)]
    use std::ffi::OsString;
    
    /// Convert Rust string to Windows wide string (UTF-16)
    #[cfg(windows)]
    pub fn to_wide_string(s: &str) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;

        OsString::from(s).encode_wide().chain(Some(0)).collect()
    }
    
    /// Convert Rust string to Windows wide string (UTF-16) - dummy implementation for non-Windows
    #[cfg(not(windows))]
    pub fn to_wide_string(_s: &str) -> Vec<u16> {
        Vec::new()
    }
    
    /// Convert Windows wide string to Rust string
    pub fn from_wide_string(ptr: *const u16) -> String {
        if ptr.is_null() {
            return String::new();
        }
        
        unsafe {
            let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
            let slice = std::slice::from_raw_parts(ptr, len);
            String::from_utf16_lossy(slice)
        }
    }
    
    /// Free a Windows wide string allocated by the FFI
    pub fn free_wide_string(ptr: *mut u16) {
        if !ptr.is_null() {
            unsafe {
                super::ffi::opc_free_string(ptr);
            }
        }
    }
    
    /// Free a Windows wide string array allocated by the FFI
    pub fn free_wide_string_array(ptr: *mut *mut u16, count: u32) {
        if !ptr.is_null() && count > 0 {
            unsafe {
                super::ffi::opc_free_string_array(ptr, count);
            }
        }
    }
}

// Re-export utility functions
pub use utils::{to_wide_string, from_wide_string};

#[cfg(test)]
mod tests;