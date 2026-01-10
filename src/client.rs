//! OPC 客户端模块
//! 
//! 这个模块提供了 OPC 客户端的主要功能，用于管理到 OPC 服务器的连接。
//! `OpcClient` 是使用本库的起点，负责初始化和清理 OPC 库资源。
//! 
//! ## 主要功能
//! 
//! - OPC 库的初始化和清理
//! - 连接到本地或远程 OPC 服务器
//! - 管理连接生命周期
//! - 提供便捷的连接函数
//! 
//! ## 使用流程
//! 
//! 1. 创建 `OpcClient` 实例（自动初始化 OPC 库）
//! 2. 使用 `connect_to_server` 或 `connect_to_local_server` 连接到服务器
//! 3. 通过返回的 `OpcServer` 对象进行后续操作
//! 4. `OpcClient` 销毁时自动清理资源
//! 
//! ## 注意
//! 
//! - `OpcClient` 实现了 `Drop` trait，确保资源正确释放
//! - 在非 Windows 平台上，创建客户端会返回错误
//! - 一个进程通常只需要一个 `OpcClient` 实例

use std::ptr;
use crate::error::{OpcError, OpcResult};
use crate::server::OpcServer;
use crate::utils;

/// OPC 客户端，用于管理 OPC 连接
/// 
/// 这是使用 OPC DA 客户端库的主要入口点。它负责：
/// 1. 初始化底层的 OPC 库
/// 2. 创建到 OPC 服务器的连接
/// 3. 在销毁时清理所有资源
/// 
/// ## 生命周期管理
/// 
/// `OpcClient` 使用 RAII (Resource Acquisition Is Initialization) 模式：
/// - 创建时初始化 OPC 库
/// - 销毁时自动调用 `opc_client_stop()` 清理资源
/// 
/// ## 线程安全
/// 
/// `OpcClient` 不是 `Send` 或 `Sync` 的，因为底层的 OPC 库可能有线程限制。
/// 建议在主线程中创建和使用 `OpcClient`。
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::OpcClient;
/// 
/// let client = OpcClient::new()?;
/// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
/// // 使用 server 进行后续操作...
/// ```
pub struct OpcClient {
    /// 标记 OPC 库是否已初始化
    initialized: bool,
}

impl OpcClient {
    /// 创建新的 OPC 客户端
    /// 
    /// 这个方法会初始化底层的 OPC 库，为后续的 OPC 操作做准备。
    /// 
    /// # 返回值
    /// - `Ok(OpcClient)`: 成功创建客户端
    /// - `Err(OpcError)`: 创建失败，可能的原因包括：
    ///   - 非 Windows 平台（OPC DA 仅支持 Windows）
    ///   - OPC 库初始化失败
    ///   - COM 初始化失败
    /// 
    /// # 平台支持
    /// - ✅ Windows (x86, x86_64)
    /// - ❌ 非 Windows 平台（编译时返回错误）
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::OpcClient;
    /// 
    /// match OpcClient::new() {
    ///     Ok(client) => println!("OPC 客户端创建成功"),
    ///     Err(e) => println!("创建 OPC 客户端失败: {}", e),
    /// }
    /// ```
    /// 
    /// # 注意
    /// - 一个进程通常只需要一个 `OpcClient` 实例
    /// - 客户端销毁时会自动清理 OPC 库资源
    /// - 在非 Windows 平台上，此方法总是返回错误
    pub fn new() -> OpcResult<Self> {
        #[cfg(not(windows))]
        {
            // 非 Windows 平台不支持 OPC DA
            return Err(OpcError::ComInitializationFailed(
                "OPC DA Client is only supported on Windows platforms".to_string()
            ));
        }
        
        #[cfg(windows)]
        {
            // 调用 FFI 函数初始化 OPC 库
            let result = unsafe { crate::ffi::opc_client_init() };
            
            if result == 0 {
                // 初始化成功，创建客户端实例
                Ok(OpcClient {
                    initialized: true,
                })
            } else {
                // 初始化失败，返回错误
                Err(OpcError::ComInitializationFailed(
                    format!("Failed to initialize OPC client with error code: {}", result)
                ))
            }
        }
    }
    
    /// 连接到本地机器上的 OPC 服务器
    /// 
    /// 这是 `connect_to_server("localhost", server_name)` 的便捷方法。
    /// 
    /// # 参数
    /// - `server_name`: OPC 服务器名称
    ///   - 常见仿真服务器: "Matrikon.OPC.Simulation.1"
    ///   - KEPServerEX 仿真: "OPCSim.KEPServerEX.V6"
    ///   - 其他服务器: 参考 OPC 服务器的文档
    /// 
    /// # 返回值
    /// - `Ok(OpcServer)`: 成功连接到服务器
    /// - `Err(OpcError)`: 连接失败
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::OpcClient;
    /// 
    /// let client = OpcClient::new()?;
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// println!("成功连接到服务器");
    /// ```
    pub fn connect_to_local_server(&self, server_name: &str) -> OpcResult<OpcServer> {
        self.connect_to_server("localhost", server_name)
    }
    
    /// 连接到指定主机上的 OPC 服务器
    /// 
    /// 这个方法会建立到远程或本地 OPC 服务器的连接。
    /// 
    /// # 参数
    /// - `hostname`: 主机名或 IP 地址
    ///   - "localhost" 或 "127.0.0.1": 本地机器
    ///   - 远程主机名: "192.168.1.100" 或 "opc-server"
    /// - `server_name`: OPC 服务器名称（ProgID）
    /// 
    /// # 返回值
    /// - `Ok(OpcServer)`: 成功连接到服务器
    /// - `Err(OpcError)`: 连接失败，可能的原因包括：
    ///   - 客户端未初始化
    ///   - 主机不可达
    ///   - 服务器未安装或未运行
    ///   - 权限不足
    ///   - 网络问题
    /// 
    /// # 连接过程
    /// 1. 创建主机对象
    /// 2. 连接到 OPC DA 服务器
    /// 3. 如果服务器连接失败，自动清理主机对象
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::OpcClient;
    /// 
    /// let client = OpcClient::new()?;
    /// 
    /// // 连接到本地服务器
    /// let local_server = client.connect_to_server("localhost", "Matrikon.OPC.Simulation.1")?;
    /// 
    /// // 连接到远程服务器
    /// let remote_server = client.connect_to_server("192.168.1.100", "MyOPCServer")?;
    /// ```
    /// 
    /// # 注意
    /// - 需要确保 OPC 服务器已安装并在目标主机上运行
    /// - 远程连接可能需要配置 DCOM 权限
    /// - 连接失败时会自动清理已分配的资源
    pub fn connect_to_server(&self, hostname: &str, server_name: &str) -> OpcResult<OpcServer> {
        // 检查客户端是否已初始化
        if !self.initialized {
            return Err(OpcError::ComInitializationFailed(
                "OPC client not initialized".to_string()
            ));
        }
        
        // ============================================
        // 第一步：创建主机连接
        // ============================================
        
        // 将主机名转换为 UTF-16 宽字符串
        let hostname_wide = utils::to_wide_string(hostname);
        let mut host_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        // 调用 FFI 函数创建主机对象
        let result = unsafe {
            crate::ffi::opc_make_host(hostname_wide.as_ptr(), &mut host_ptr)
        };
        
        // 检查主机创建是否成功
        if result != 0 || host_ptr.is_null() {
            return Err(OpcError::connection_failed(
                format!("Failed to connect to host '{}'", hostname)
            ));
        }
        
        // ============================================
        // 第二步：连接到 OPC 服务器
        // ============================================
        
        // 将服务器名转换为 UTF-16 宽字符串
        let server_name_wide = utils::to_wide_string(server_name);
        let mut server_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        // 调用 FFI 函数连接到服务器
        let result = unsafe {
            crate::ffi::opc_host_connect_da_server(
                host_ptr,
                server_name_wide.as_ptr(),
                &mut server_ptr,
            )
        };
        
        // 检查服务器连接是否成功
        if result == 0 && !server_ptr.is_null() {
            // 连接成功，创建 OpcServer 对象
            Ok(OpcServer::new(server_ptr, host_ptr))
        } else {
            // 连接失败，清理已创建的主机对象
            unsafe {
                crate::ffi::opc_host_free(host_ptr);
            }
            Err(OpcError::connection_failed(
                format!("Failed to connect to server '{}' on host '{}'", server_name, hostname)
            ))
        }
    }
    
    /// 检查客户端是否已初始化
    /// 
    /// # 返回值
    /// - `true`: 客户端已成功初始化
    /// - `false`: 客户端未初始化或初始化失败
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::OpcClient;
    /// 
    /// let client = OpcClient::new()?;
    /// if client.is_initialized() {
    ///     println!("客户端已初始化");
    /// }
    /// ```
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Drop for OpcClient {
    /// 清理 OPC 客户端资源
    /// 
    /// 当 `OpcClient` 离开作用域时，会自动调用此方法。
    /// 这会停止 OPC 库并释放所有相关资源。
    /// 
    /// # 注意
    /// - 调用此方法后，不应再使用任何由此客户端创建的 OPC 对象
    /// - 如果客户端未初始化，则不执行任何操作
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                // 调用 FFI 函数停止 OPC 库
                crate::ffi::opc_client_stop();
            }
        }
    }
}

/// 便捷函数：连接到本地主机上的 OPC 服务器
/// 
/// 这个函数封装了创建客户端和连接服务器的常见操作。
/// 
/// # 参数
/// - `server_name`: OPC 服务器名称
/// 
/// # 返回值
/// - `Ok(OpcServer)`: 成功连接到服务器
/// - `Err(OpcError)`: 连接失败
/// 
/// # 示例
/// ```
/// use opc_da_client::connect_to_server;
/// 
/// let server = connect_to_server("Matrikon.OPC.Simulation.1")?;
/// // 使用 server 进行后续操作...
/// ```
/// 
/// # 注意
/// - 这个函数会创建临时的 `OpcClient` 实例
/// - 对于需要重复连接的情况，建议手动创建 `OpcClient`
pub fn connect_to_server(server_name: &str) -> OpcResult<OpcServer> {
    let client = OpcClient::new()?;
    client.connect_to_local_server(server_name)
}

/// 便捷函数：连接到指定主机上的 OPC 服务器
/// 
/// 这个函数封装了创建客户端和连接远程服务器的常见操作。
/// 
/// # 参数
/// - `hostname`: 主机名或 IP 地址
/// - `server_name`: OPC 服务器名称
/// 
/// # 返回值
/// - `Ok(OpcServer)`: 成功连接到服务器
/// - `Err(OpcError)`: 连接失败
/// 
/// # 示例
/// ```
/// use opc_da_client::connect_to_server_on_host;
/// 
/// // 连接到本地服务器
/// let local = connect_to_server_on_host("localhost", "Matrikon.OPC.Simulation.1")?;
/// 
/// // 连接到远程服务器
/// let remote = connect_to_server_on_host("192.168.1.100", "MyOPCServer")?;
/// ```
/// 
/// # 注意
/// - 这个函数会创建临时的 `OpcClient` 实例
/// - 对于需要重复连接的情况，建议手动创建 `OpcClient`
pub fn connect_to_server_on_host(hostname: &str, server_name: &str) -> OpcResult<OpcServer> {
    let client = OpcClient::new()?;
    client.connect_to_server(hostname, server_name)
}