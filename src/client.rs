//! OPC Client for managing connections to OPC servers

use std::ptr;
use crate::error::{OpcError, OpcResult};
use crate::server::OpcServer;
use crate::utils;

/// Main OPC client for managing connections
pub struct OpcClient {
    initialized: bool,
}

impl OpcClient {
    /// Create a new OPC client
    pub fn new() -> OpcResult<Self> {
        #[cfg(not(windows))]
        {
            return Err(OpcError::ComInitializationFailed(
                "OPC DA Client is only supported on Windows platforms".to_string()
            ));
        }
        
        #[cfg(windows)]
        {
            let result = unsafe { crate::ffi::opc_client_init() };
            
            if result == 0 {
                Ok(OpcClient {
                    initialized: true,
                })
            } else {
                Err(OpcError::ComInitializationFailed(
                    format!("Failed to initialize OPC client with error code: {}", result)
                ))
            }
        }
    }
    
    /// Connect to an OPC server on the local machine
    pub fn connect_to_local_server(&self, server_name: &str) -> OpcResult<OpcServer> {
        self.connect_to_server("localhost", server_name)
    }
    
    /// Connect to an OPC server on a specific host
    pub fn connect_to_server(&self, hostname: &str, server_name: &str) -> OpcResult<OpcServer> {
        if !self.initialized {
            return Err(OpcError::ComInitializationFailed(
                "OPC client not initialized".to_string()
            ));
        }
        
        // Create host connection
        let hostname_wide = utils::to_wide_string(hostname);
        let mut host_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        let result = unsafe {
            crate::ffi::opc_make_host(hostname_wide.as_ptr(), &mut host_ptr)
        };
        
        if result != 0 || host_ptr.is_null() {
            return Err(OpcError::connection_failed(
                format!("Failed to connect to host '{}'", hostname)
            ));
        }
        
        // Connect to server
        let server_name_wide = utils::to_wide_string(server_name);
        let mut server_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        let result = unsafe {
            crate::ffi::opc_host_connect_da_server(
                host_ptr,
                server_name_wide.as_ptr(),
                &mut server_ptr,
            )
        };
        
        if result == 0 && !server_ptr.is_null() {
            Ok(OpcServer::new(server_ptr, host_ptr))
        } else {
            // Clean up host if server connection failed
            unsafe {
                crate::ffi::opc_host_free(host_ptr);
            }
            Err(OpcError::connection_failed(
                format!("Failed to connect to server '{}' on host '{}'", server_name, hostname)
            ))
        }
    }
    
    /// Check if the client is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Drop for OpcClient {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                crate::ffi::opc_client_stop();
            }
        }
    }
}

/// Convenience function to connect to a server on localhost
pub fn connect_to_server(server_name: &str) -> OpcResult<OpcServer> {
    let client = OpcClient::new()?;
    client.connect_to_local_server(server_name)
}

/// Convenience function to connect to a server on a specific host
pub fn connect_to_server_on_host(hostname: &str, server_name: &str) -> OpcResult<OpcServer> {
    let client = OpcClient::new()?;
    client.connect_to_server(hostname, server_name)
}