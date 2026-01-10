//! OPC 组模块
//! 
//! 这个模块提供了 OPC 组的表示和操作功能。
//! `OpcGroup` 是 OPC 项的容器，具有共享的属性如更新速率和死区值。
//! 组还负责管理异步数据变化订阅。
//! 
//! ## 主要功能
//! 
//! - 向组中添加和移除 OPC 项
//! - 启用异步数据变化通知
//! - 刷新组中的所有项
//! - 管理组生命周期
//! 
//! ## 组属性
//! 
//! OPC 组具有以下共享属性：
//! - 名称：组的唯一标识符
//! - 激活状态：决定是否接收数据变化通知
//! - 更新速率：数据变化的通知频率
//! - 死区值：模拟量变化的最小阈值
//! 
//! ## 异步订阅
//! 
//! 激活的组可以接收异步数据变化通知。用户需要实现 `OpcDataCallback` trait
//! 并调用 `enable_async_subscription` 来启用订阅。

use std::ptr;
use std::sync::Arc;
use crate::error::{OpcError, OpcResult};
use crate::item::OpcItem;
use crate::types::{OpcValue, OpcQuality, OpcDataCallback, OpcCallbackContainer};
use crate::utils;

/// OPC 组，包含多个 OPC 项
/// 
/// 组是项的容器，具有共享的属性。通过组可以：
/// 1. 批量管理项（添加、移除）
/// 2. 控制数据更新行为（更新速率、死区值）
/// 3. 启用异步数据变化通知
/// 4. 刷新所有项的值
/// 
/// ## 内部结构
/// 
/// - `ptr`: 指向底层 OPC 组对象的指针
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::{OpcClient, OpcGroup};
/// use std::sync::Arc;
/// 
/// let client = OpcClient::new()?;
/// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
/// let group = server.create_group("MyGroup", true, 1000, 0.0)?;
/// 
/// // 向组中添加项
/// let item1 = group.add_item("Bucket Brigade.UInt2")?;
/// let item2 = group.add_item("Random.Int2")?;
/// 
/// // 读取项值
/// let (value1, quality1) = group.read_sync(&item1)?;
/// let (value2, quality2) = group.read_sync(&item2)?;
/// ```
pub struct OpcGroup {
    /// 指向底层 OPC 组对象的指针
    ptr: *mut std::ffi::c_void,
}

impl OpcGroup {
    /// 创建新的组实例（内部使用）
    /// 
    /// # 参数
    /// - `group_ptr`: 指向底层 OPC 组对象的指针
    /// 
    /// # 注意
    /// 这个方法仅供内部使用，用户应该通过 `OpcServer::create_group` 获取 `OpcGroup` 实例。
    pub(crate) fn new(group_ptr: *mut std::ffi::c_void) -> Self {
        OpcGroup {
            ptr: group_ptr,
        }
    }
    
    /// 向组中添加 OPC 项
    /// 
    /// 这个方法将指定的数据项添加到组中。项名必须存在于服务器命名空间中。
    /// 
    /// # 参数
    /// - `name`: 项名，格式通常为 "设备名.变量名"
    /// 
    /// # 返回值
    /// - `Ok(OpcItem)`: 成功添加项
    /// - `Err(OpcError)`: 添加失败，可能的原因包括：
    ///   - 项名不存在
    ///   - 项名格式无效
    ///   - 权限不足
    ///   - 服务器资源不足
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::{OpcClient, OpcGroup};
    /// 
    /// let client = OpcClient::new()?;
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// let group = server.create_group("TestGroup", true, 1000, 0.0)?;
    /// 
    /// // 添加项到组中
    /// let item = group.add_item("Bucket Brigade.UInt2")?;
    /// println!("成功添加项: Bucket Brigade.UInt2");
    /// ```
    /// 
    /// # 注意
    /// - 项必须在服务器中存在
    /// - 同一个项可以添加到多个组中
    /// - 项会继承组的属性（更新速率、死区值）
    pub fn add_item(&self, name: &str) -> OpcResult<OpcItem> {
        // 将项名转换为 UTF-16 宽字符串
        let item_name_wide = utils::to_wide_string(name);
        let mut item_ptr: *mut std::ffi::c_void = ptr::null_mut();
        
        // 调用 FFI 函数添加项
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
    
    /// 启用异步数据变化通知
    /// 
    /// 这个方法启用组的异步数据变化订阅。当组中的项值发生变化时，
    /// 会调用提供的回调函数。
    /// 
    /// # 参数
    /// - `callback`: 实现了 `OpcDataCallback` trait 的回调对象
    /// 
    /// # 返回值
    /// - `Ok(())`: 成功启用订阅
    /// - `Err(OpcError)`: 启用失败，可能的原因包括：
    ///   - 组未激活
    ///   - 回调设置失败
    ///   - 服务器不支持异步通知
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::{OpcClient, OpcDataCallback, OpcValue, OpcQuality};
    /// use std::sync::Arc;
    /// 
    /// struct MyCallback;
    /// 
    /// impl OpcDataCallback for MyCallback {
    ///     fn on_data_change(&self, group_name: &str, item_name: &str, 
    ///                       value: OpcValue, quality: OpcQuality) {
    ///         println!("数据变化: {}:{} = {:?}", group_name, item_name, value);
    ///     }
    /// }
    /// 
    /// let client = OpcClient::new()?;
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// let group = server.create_group("SubGroup", true, 1000, 0.0)?;
    /// 
    /// let callback = Arc::new(MyCallback);
    /// group.enable_async_subscription(callback)?;
    /// ```
    /// 
    /// # 注意
    /// - 组必须处于激活状态才能接收异步通知
    /// - 回调函数可能在后台线程中调用
    /// - 回调对象必须实现 `Send + Sync`
    /// - 启用订阅后，组会开始接收数据变化通知
    pub fn enable_async_subscription(&self, callback: Arc<dyn OpcDataCallback>) -> OpcResult<()> {
        // 创建回调容器，将 Rust 回调包装为 FFI 可用的形式
        let container = Box::into_raw(Box::new(OpcCallbackContainer {
            callback,
        }));
        
        // 调用 FFI 函数启用异步订阅
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
            // 启用失败，清理已分配的内存
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