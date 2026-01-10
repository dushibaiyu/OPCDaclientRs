//! OPC 服务器模块
//! 
//! 这个模块提供了 OPC 服务器的表示和操作功能。
//! `OpcServer` 表示到 OPC DA 服务器的连接，提供了服务器状态查询、
//! 组创建和项浏览等功能。
//! 
//! ## 主要功能
//! 
//! - 获取服务器状态和厂商信息
//! - 创建和管理 OPC 组
//! - 浏览服务器中的可用项
//! - 管理服务器连接生命周期
//! 
//! ## 生命周期
//! 
//! `OpcServer` 使用 RAII 模式管理底层连接：
//! - 创建时建立到服务器的连接
//! - 销毁时自动释放服务器和主机资源
//! - 确保资源不会泄漏
//! 
//! ## 线程安全
//! 
//! `OpcServer` 不是线程安全的，因为底层的 OPC COM 对象可能有线程限制。
//! 建议在创建 `OpcServer` 的同一线程中使用它。

use std::ptr;
use crate::error::{OpcError, OpcResult};
use crate::group::OpcGroup;
use crate::utils;

/// OPC 服务器连接
/// 
/// 表示到 OPC DA 服务器的活动连接。通过这个对象可以：
/// 1. 查询服务器状态和信息
/// 2. 创建和管理 OPC 组
/// 3. 浏览服务器中的可用数据项
/// 
/// ## 内部结构
/// 
/// - `ptr`: 指向底层 OPC 服务器对象的指针
/// - `host_ptr`: 指向主机对象的指针（用于资源清理）
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::{OpcClient, OpcServer};
/// 
/// let client = OpcClient::new()?;
/// let server: OpcServer = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
/// 
/// // 获取服务器状态
/// let (state, vendor) = server.get_status()?;
/// println!("服务器状态: {}, 厂商: {}", state, vendor);
/// 
/// // 创建组
/// let group = server.create_group("MyGroup", true, 1000, 0.0)?;
/// ```
pub struct OpcServer {
    /// 指向底层 OPC 服务器对象的指针
    ptr: *mut std::ffi::c_void,
    /// 指向主机对象的指针（需要与服务器一起清理）
    host_ptr: *mut std::ffi::c_void,
}

impl OpcServer {
    /// 创建新的服务器实例（内部使用）
    /// 
    /// # 参数
    /// - `server_ptr`: 指向底层 OPC 服务器对象的指针
    /// - `host_ptr`: 指向主机对象的指针
    /// 
    /// # 注意
    /// 这个方法仅供内部使用，用户应该通过 `OpcClient::connect_to_server` 获取 `OpcServer` 实例。
    pub(crate) fn new(server_ptr: *mut std::ffi::c_void, host_ptr: *mut std::ffi::c_void) -> Self {
        OpcServer {
            ptr: server_ptr,
            host_ptr,
        }
    }
    
    /// 获取服务器状态和厂商信息
    /// 
    /// 这个方法查询 OPC 服务器的当前状态和厂商信息。
    /// 
    /// # 返回值
    /// - `Ok((state, vendor_info))`: 成功获取状态信息
    ///   - `state`: 服务器状态码（具体含义取决于服务器实现）
    ///   - `vendor_info`: 厂商信息字符串
    /// - `Err(OpcError)`: 获取状态失败
    /// 
    /// # 服务器状态码
    /// 常见的 OPC 服务器状态包括：
    /// - 1: 运行中 (Running)
    /// - 2: 失败 (Failed)
    /// - 3: 无配置 (No Config)
    /// - 4: 挂起 (Suspended)
    /// - 5: 测试 (Test)
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::{OpcClient, OpcServer};
    /// 
    /// let client = OpcClient::new()?;
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// 
    /// match server.get_status() {
    ///     Ok((state, vendor)) => {
    ///         println!("服务器状态: {}", state);
    ///         println!("厂商信息: {}", vendor);
    ///     }
    ///     Err(e) => println!("获取状态失败: {}", e),
    /// }
    /// ```
    /// 
    /// # 注意
    /// - 厂商信息字符串由服务器提供，格式和内容因厂商而异
    /// - 如果服务器不提供厂商信息，返回空字符串
    pub fn get_status(&self) -> OpcResult<(u32, String)> {
        let mut state: u32 = 0;
        let mut vendor_info_ptr: *mut u16 = ptr::null_mut();
        
        // 调用 FFI 函数获取服务器状态
        let result = unsafe {
            crate::ffi::opc_server_get_status(self.ptr, &mut state, &mut vendor_info_ptr)
        };
        
        if result == 0 {
            // 成功获取状态，处理厂商信息
            let vendor_info = if !vendor_info_ptr.is_null() {
                let info = utils::from_wide_string(vendor_info_ptr);
                // 释放 FFI 分配的字符串
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
    
    /// 创建新的 OPC 组
    /// 
    /// OPC 组是项的容器，具有共享的属性如更新速率和死区值。
    /// 组可以处于激活或非激活状态，激活的组会接收数据变化通知。
    /// 
    /// # 参数
    /// - `name`: 组名，在服务器中必须唯一
    /// - `active`: 是否激活组
    ///   - `true`: 组激活，接收数据变化通知
    ///   - `false`: 组非激活，不接收通知
    /// - `requested_update_rate`: 请求的更新速率（毫秒）
    ///   - 服务器可能返回不同的实际更新速率
    ///   - 0 表示尽可能快的更新
    /// - `deadband`: 死区值（0.0-100.0）
    ///   - 0.0: 所有变化都通知
    ///   - 1.0: 变化超过 1% 才通知
    ///   - 仅对模拟量有效
    /// 
    /// # 返回值
    /// - `Ok(OpcGroup)`: 成功创建组
    /// - `Err(OpcError)`: 创建失败，可能的原因包括：
    ///   - 组名已存在
    ///   - 参数无效
    ///   - 服务器资源不足
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::{OpcClient, OpcServer};
    /// 
    /// let client = OpcClient::new()?;
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// 
    /// // 创建激活的组，每秒更新一次，无死区
    /// let group = server.create_group("DataGroup", true, 1000, 0.0)?;
    /// 
    /// // 创建非激活的组，用于批量读取
    /// let batch_group = server.create_group("BatchGroup", false, 0, 0.0)?;
    /// ```
    /// 
    /// # 注意
    /// - 组名在服务器中必须唯一
    /// - 实际更新速率可能不同于请求的速率
    /// - 死区值仅对模拟量（浮点数）有效
    /// - 组销毁时会自动清理所有项
    pub fn create_group(
        &self,
        name: &str,
        active: bool,
        requested_update_rate: u32,
        deadband: f64,
    ) -> OpcResult<OpcGroup> {
        // 将组名转换为 UTF-16 宽字符串
        let group_name_wide = utils::to_wide_string(name);
        let mut actual_update_rate: u32 = 0;
        let mut group_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        // 调用 FFI 函数创建组
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
    
    /// 获取服务器中所有可用的项名
    /// 
    /// 这个方法浏览服务器命名空间，返回所有可访问的数据项名称。
    /// 项名通常采用 "设备名.变量名" 或 "命名空间.变量名" 的格式。
    /// 
    /// # 返回值
    /// - `Ok(Vec<String>)`: 成功获取项名列表
    /// - `Err(OpcError)`: 获取失败，可能的原因包括：
    ///   - 服务器不支持浏览功能
    ///   - 权限不足
    ///   - 服务器错误
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::{OpcClient, OpcServer};
    /// 
    /// let client = OpcClient::new()?;
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// 
    /// match server.get_item_names() {
    ///     Ok(items) => {
    ///         println!("找到 {} 个项:", items.len());
    ///         for (i, item) in items.iter().enumerate().take(10) {
    ///             println!("  {}. {}", i + 1, item);
    ///         }
    ///         if items.len() > 10 {
    ///             println!("  ... 还有 {} 个项", items.len() - 10);
    ///         }
    ///     }
    ///     Err(e) => println!("浏览项失败: {}", e),
    /// }
    /// ```
    /// 
    /// # 注意
    /// - 不是所有 OPC 服务器都支持浏览功能
    /// - 返回的项名列表可能很大，特别是对于大型系统
    /// - 项名格式取决于服务器实现
    /// - 对于仿真服务器，常见的项名包括：
    ///   - "Bucket Brigade.*" (桶队列项)
    ///   - "Random.*" (随机数项)
    ///   - "Triangle Waves.*" (三角波形项)
    pub fn get_item_names(&self) -> OpcResult<Vec<String>> {
        let mut item_names_ptr: *mut *mut u16 = ptr::null_mut();
        let mut count: u32 = 0;
        
        // 调用 FFI 函数获取项名列表
        let result = unsafe {
            crate::ffi::opc_server_get_item_names(self.ptr, &mut item_names_ptr, &mut count)
        };
        
        if result == 0 && !item_names_ptr.is_null() {
            // 创建向量存储项名
            let mut items = Vec::with_capacity(count as usize);
            
            unsafe {
                // 遍历项名数组
                for i in 0..count {
                    let item_name_ptr = *item_names_ptr.add(i as usize);
                    if !item_name_ptr.is_null() {
                        // 转换宽字符串为 Rust 字符串
                        let item_name = utils::from_wide_string(item_name_ptr);
                        items.push(item_name);
                    }
                }
                
                // 释放 FFI 分配的字符串数组
                utils::free_wide_string_array(item_names_ptr, count);
            }
            
            Ok(items)
        } else {
            Err(OpcError::operation_failed("Failed to get item names"))
        }
    }
    
    /// 获取原始服务器指针（内部使用）
    /// 
    /// # 注意
    /// 这个方法仅供内部使用，用于 FFI 调用。
    /// 用户不应直接使用原始指针。
    pub(crate) fn as_ptr(&self) -> *mut std::ffi::c_void {
        self.ptr
    }
}

impl Drop for OpcServer {
    /// 清理服务器资源
    /// 
    /// 当 `OpcServer` 离开作用域时，会自动调用此方法。
    /// 这会释放服务器和主机对象，确保没有资源泄漏。
    /// 
    /// # 清理顺序
    /// 1. 释放服务器对象 (`opc_server_free`)
    /// 2. 释放主机对象 (`opc_host_free`)
    /// 
    /// # 注意
    /// - 调用此方法后，不应再使用此服务器或由其创建的任何组/项
    /// - 资源清理是自动的，用户通常不需要手动调用
    fn drop(&mut self) {
        unsafe {
            // 先释放服务器对象
            crate::ffi::opc_server_free(self.ptr);
            // 再释放主机对象
            crate::ffi::opc_host_free(self.host_ptr);
        }
    }
}