//! OPC DA Client Rust 封装库
//! 
//! 这是一个为 OPC DA Client Toolkit 提供的安全、符合 Rust 习惯的封装库。
//! 该库为工业自动化应用程序提供同步和异步 API，用于与 OPC DA (Data Access) 服务器进行通信。
//! 
//! OPC DA (OLE for Process Control Data Access) 是工业自动化领域广泛使用的标准，
//! 用于从 PLC、DCS、SCADA 系统等设备中读取和写入实时数据。
//! 
//! ## 主要特性
//! 
//! - **内存安全**: 使用 Rust 的所有权系统确保内存安全，避免 C++ 库中的常见内存错误
//! - **类型安全**: 强类型系统确保数据类型的正确性，减少运行时错误
//! - **同步操作**: 提供阻塞式的读写操作，适用于简单的数据采集场景
//! - **异步订阅**: 支持数据变化回调，适用于实时监控应用
//! - **跨平台编译**: 代码可在非 Windows 平台编译（但运行时仅支持 Windows）
//! - **错误处理**: 完善的错误类型和错误处理机制
//! 
//! ## 平台支持
//! 
//! 由于 OPC DA 是基于 Windows COM 技术的标准，因此**本库仅在 Windows 平台上可用**。
//! 在非 Windows 平台上，库可以编译但所有操作都会返回错误。
//! 
//! **支持的 Windows 架构**:
//! - x86 (32位)
//! - x86_64 (64位)
//! 
//! ## 快速开始
//! 
//! ### 添加依赖
//! 
//! 在 `Cargo.toml` 中添加:
//! 
//! ```toml
//! [dependencies]
//! opc_da_client = { git = "https://github.com/yourusername/OPCDaclientRs.git" }
//! ```
//! 
//! ### 基本使用示例
//! 
//! ```no_run
//! use opc_da_client::{OpcClient, OpcValue};
//! 
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. 创建 OPC 客户端
//!     let client = OpcClient::new()?;
//!     
//!     // 2. 连接到本地 OPC 服务器
//!     //    常见的仿真服务器: "Matrikon.OPC.Simulation.1", "OPCSim.KEPServerEX.V6"
//!     let server = client.connect_to_server("localhost", "Matrikon.OPC.Simulation.1")?;
//!     
//!     // 3. 创建 OPC 组
//!     //    参数: 组名, 是否激活, 请求更新速率(ms), 死区值
//!     let group = server.create_group("TestGroup", true, 1000, 0.0)?;
//!     
//!     // 4. 添加 OPC 项
//!     //    项名格式通常为: "设备名.变量名" 或 "命名空间.变量名"
//!     let item = group.add_item("Bucket Brigade.UInt2")?;
//!     
//!     // 5. 同步读取值
//!     let (value, quality, timestamp) = item.read_sync()?;
//!     println!("读取值: {:?}, 质量: {:?}, 时间戳: {} ms", value, quality, timestamp);
//!     
//!     // 6. 同步写入值
//!     item.write_sync(&OpcValue::Int32(12345))?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## 架构概述
//! 
//! 本库采用分层架构设计:
//! 
//! 1. **FFI 层**: 通过 `extern "C"` 函数调用底层的 C++ OPC 库
//! 2. **安全包装层**: 将原始指针包装为安全的 Rust 类型，实现 RAII 模式
//! 3. **API 层**: 提供符合 Rust 习惯的高级 API
//! 
//! ### 核心类型层次
//! 
//! ```text
//! OpcClient
//!     ├── OpcServer
//!     │     ├── OpcGroup
//!     │     │     ├── OpcItem
//!     │     │     └── OpcDataCallback (异步回调)
//!     │     └── 服务器状态/浏览功能
//!     └── 连接管理/资源清理
//! ```
//! 
//! ## 模块结构
//! 
//! - `client.rs` - 主客户端和连接管理
//! - `server.rs` - 服务器连接和操作
//! - `group.rs` - 组管理和订阅功能
//! - `item.rs` - 项读写操作
//! - `types.rs` - 核心数据类型和转换
//! - `error.rs` - 错误类型和处理
//! - `utils.rs` - 字符串转换工具函数
//! 
//! ## 依赖关系
//! 
//! 本库依赖于 [OPC-Client-X64](https://github.com/dushibaiyu/OPC-Client-X64) 项目编译的动态库。
//! 预编译的库文件已包含在 `libs/` 目录中。
//! 
//! ## 许可证
//! 
//! Apache 2.0 许可证
#[allow(unused_imports)]
#[allow(unused)]

pub mod error;
pub mod types;
pub mod client;
pub mod server;
pub mod group;
pub mod item;

// Re-export main types
pub use client::OpcClient;
pub use error::{OpcError, OpcResult};
pub use types::{OpcValue, OpcQuality, OpcDataCallback};
pub use server::OpcServer;
pub use group::OpcGroup;
pub use item::OpcItem;


// 内部 FFI 绑定模块
// 
// 这个模块提供了与底层 C++ OPC 库的 FFI (Foreign Function Interface) 绑定。
// 在 Windows 平台上，我们使用真正的 OPC 库函数。
// 在非 Windows 平台上，我们使用桩(stub)实现，以便代码可以编译但运行时返回错误。
// 
// ## FFI 设计原则
// 
// 1. **安全性**: 所有 FFI 调用都包装在 unsafe 块中，确保 Rust 的安全保证
// 2. **资源管理**: 使用 RAII 模式确保资源正确释放
// 3. **错误处理**: 将 C 风格的错误码转换为 Rust 的 Result 类型
// 4. **类型转换**: 处理 Rust 类型与 C 类型之间的转换
#[cfg(windows)]
mod ffi {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    
    // 尝试链接 OPC 库
    // 如果编译失败，我们将使用桩(stub)实现
    // 
    // 注意: 这些函数声明必须与 opc_ffi.h 中的声明完全匹配
    extern "C" {
        // ============================================
        // 客户端函数
        // ============================================
        
        /// 初始化 OPC 客户端库
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        /// 
        /// # 注意
        /// 必须在调用其他 OPC 函数之前调用此函数
        pub fn opc_client_init() -> u32;
        
        /// 停止 OPC 客户端库，释放所有资源
        /// 
        /// # 安全要求
        /// 调用此函数后，不应再使用任何 OPC 对象
        pub fn opc_client_stop();
        
        // ============================================
        // 主机函数
        // ============================================
        
        /// 创建 OPC 主机对象
        /// 
        /// # 参数
        /// - `hostname`: 主机名（UTF-16 字符串），为空表示本地主机
        /// - `host`: 输出参数，接收创建的主机对象指针
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_make_host(hostname: *const u16, host: *mut *mut c_void) -> u32;
        
        /// 释放主机对象
        /// 
        /// # 参数
        /// - `host`: 要释放的主机对象指针
        pub fn opc_host_free(host: *mut c_void);
        
        // ============================================
        // 服务器函数
        // ============================================
        
        /// 连接到 OPC DA 服务器
        /// 
        /// # 参数
        /// - `host`: 主机对象指针
        /// - `server_name`: 服务器名称（UTF-16 字符串）
        /// - `server`: 输出参数，接收创建的服务器对象指针
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_host_connect_da_server(
            host: *mut c_void,
            server_name: *const u16,
            server: *mut *mut c_void,
        ) -> u32;
        
        /// 释放服务器对象
        /// 
        /// # 参数
        /// - `server`: 要释放的服务器对象指针
        pub fn opc_server_free(server: *mut c_void);
        
        /// 获取服务器状态信息
        /// 
        /// # 参数
        /// - `server`: 服务器对象指针
        /// - `state`: 输出参数，接收服务器状态
        /// - `vendor_info`: 输出参数，接收厂商信息字符串（需要调用 opc_free_string 释放）
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_server_get_status(
            server: *mut c_void,
            state: *mut u32,
            vendor_info: *mut *mut u16,
        ) -> u32;
        
        // ============================================
        // 组函数
        // ============================================
        
        /// 创建 OPC 组
        /// 
        /// # 参数
        /// - `server`: 服务器对象指针
        /// - `group_name`: 组名（UTF-16 字符串）
        /// - `active`: 是否激活组（1=激活，0=非激活）
        /// - `req_update_rate`: 请求的更新速率（毫秒）
        /// - `actual_update_rate`: 输出参数，接收实际的更新速率
        /// - `deadband`: 死区值（0.0-100.0）
        /// - `group`: 输出参数，接收创建的组对象指针
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_server_make_group(
            server: *mut c_void,
            group_name: *const u16,
            active: i32,
            req_update_rate: u32,
            actual_update_rate: *mut u32,
            deadband: f64,
            group: *mut *mut c_void,
        ) -> u32;
        
        /// 释放组对象
        /// 
        /// # 参数
        /// - `group`: 要释放的组对象指针
        pub fn opc_group_free(group: *mut c_void);
        
        // ============================================
        // 项函数
        // ============================================
        
        /// 向组中添加 OPC 项
        /// 
        /// # 参数
        /// - `group`: 组对象指针
        /// - `item_name`: 项名（UTF-16 字符串）
        /// - `item`: 输出参数，接收创建的项对象指针
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_group_add_item(
            group: *mut c_void,
            item_name: *const u16,
            item: *mut *mut c_void,
        ) -> u32;
        
        /// 释放项对象
        /// 
        /// # 参数
        /// - `item`: 要释放的项对象指针
        pub fn opc_item_free(item: *mut c_void);
        
        // ============================================
        // 同步操作函数
        // ============================================
        
        /// 同步读取项值
        /// 
        /// # 参数
        /// - `item`: 项对象指针
        /// - `value`: 输出参数，接收值（缓冲区必须足够大以容纳值）
        /// - `quality`: 输出参数，接收质量码
        /// - `value_type`: 输出参数，接收值类型
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        /// 
        /// # 注意
        /// 调用者需要根据 value_type 正确解释 value 指针
        pub fn opc_item_read_sync(
            item: *mut c_void,
            value: *mut c_void,
            quality: *mut i32,
            value_type: *mut u32,
            timestamp_ms: *mut u64,
        ) -> u32;
        
        /// 同步写入项值
        /// 
        /// # 参数
        /// - `item`: 项对象指针
        /// - `value`: 要写入的值指针
        /// - `value_type`: 值类型
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_item_write_sync(item: *mut c_void, value: *const c_void, value_type: u32) -> u32;
        
        // ============================================
        // 异步操作函数
        // ============================================
        
        /// 启用组的异步数据变化通知
        /// 
        /// # 参数
        /// - `group`: 组对象指针
        /// - `callback`: 回调函数指针
        /// - `user_data`: 用户数据，会传递给回调函数
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_group_enable_async(
            group: *mut c_void,
            callback: extern "C" fn(*mut c_void, *const u16, *const u16, *mut c_void, i32, u32, u64),
            user_data: *mut c_void,
        ) -> u32;
        
        /// 异步读取项值
        /// 
        /// # 参数
        /// - `item`: 项对象指针
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        /// 
        /// # 注意
        /// 读取结果通过回调函数返回
        pub fn opc_item_read_async(item: *mut c_void) -> u32;
        
        /// 异步写入项值
        /// 
        /// # 参数
        /// - `item`: 项对象指针
        /// - `value`: 要写入的值指针
        /// - `value_type`: 值类型
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_item_write_async(item: *mut c_void, value: *const c_void, value_type: u32) -> u32;
        
        // ============================================
        // 组操作函数
        // ============================================
        
        /// 刷新组中的所有项
        /// 
        /// # 参数
        /// - `group`: 组对象指针
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        pub fn opc_group_refresh(group: *mut c_void) -> u32;
        
        // ============================================
        // 浏览函数
        // ============================================
        
        /// 获取服务器中所有可用的项名
        /// 
        /// # 参数
        /// - `server`: 服务器对象指针
        /// - `item_names`: 输出参数，接收项名数组指针
        /// - `count`: 输出参数，接收项名数量
        /// 
        /// # 返回值
        /// - 0: 成功
        /// - 非0: 错误码
        /// 
        /// # 注意
        /// 返回的数组需要调用 opc_free_string_array 释放
        pub fn opc_server_get_item_names(
            server: *mut c_void,
            item_names: *mut *mut *mut u16,
            count: *mut u32,
        ) -> u32;
        
        // ============================================
        // 工具函数
        // ============================================
        
        /// 释放字符串数组
        /// 
        /// # 参数
        /// - `strings`: 要释放的字符串数组指针
        /// - `count`: 字符串数量
        pub fn opc_free_string_array(strings: *mut *mut u16, count: u32);
        
        /// 释放单个字符串
        /// 
        /// # 参数
        /// - `str`: 要释放的字符串指针
        pub fn opc_free_string(str: *mut u16);
        
        /// 释放 ANSI 字符串
        /// 
        /// # 参数
        /// - `str`: 要释放的 ANSI 字符串指针
        pub fn opc_free_string_ansi(str: *mut i8);
    }
}

// Non-Windows stub FFI module (production)
// This provides stub implementations that return errors for all operations
#[cfg(all(not(windows), not(test)))]
mod ffi {
    use std::ffi::c_void;
    
    // Stub function implementations that return errors for non-Windows platforms
    // Note: Function signatures must exactly match the Windows version
    
    // Client functions
    pub unsafe fn opc_client_init() -> u32 { 1 } // OPC_RESULT_ERROR
    pub unsafe fn opc_client_stop() { }
    
    // Host functions
    pub unsafe fn opc_make_host(_hostname: *const u16, _host: *mut *mut c_void) -> u32 { 1 }
    pub unsafe fn opc_host_free(_host: *mut c_void) { }
    pub unsafe fn opc_host_connect_da_server(
        _host: *mut c_void,
        _server_name: *const u16,
        _server: *mut *mut c_void
    ) -> u32 { 1 }
    
    // Server functions
    pub unsafe fn opc_server_free(_server: *mut c_void) { }
    pub unsafe fn opc_server_get_status(
        _server: *mut c_void,
        _state: *mut u32,
        _vendor_info: *mut *mut u16
    ) -> u32 { 1 }
    pub unsafe fn opc_server_make_group(
        _server: *mut c_void,
        _group_name: *const u16,
        _active: i32,
        _requested_update_rate: u32,
        _actual_update_rate: *mut u32,
        _deadband: f64,
        _group: *mut *mut c_void
    ) -> u32 { 1 }
    pub unsafe fn opc_server_get_item_names(
        _server: *mut c_void,
        _item_names: *mut *mut *mut u16,
        _count: *mut u32
    ) -> u32 { 1 }
    
    // Group functions
    pub unsafe fn opc_group_free(_group: *mut c_void) { }
    pub unsafe fn opc_group_add_item(
        _group: *mut c_void,
        _item_name: *const u16,
        _item: *mut *mut c_void
    ) -> u32 { 1 }
    pub unsafe fn opc_group_enable_async(
        _group: *mut c_void,
        _callback: extern "C" fn(*mut c_void, *const u16, *const u16, *mut c_void, i32, u32, u64),
        _user_data: *mut c_void
    ) -> u32 { 1 }
    pub unsafe fn opc_group_refresh(_group: *mut c_void) -> u32 { 1 }
    
    // Item functions
    pub unsafe fn opc_item_free(_item: *mut c_void) { }
    pub unsafe fn opc_item_read_sync(
        _item: *mut c_void,
        _value: *mut c_void,
        _quality: *mut i32,
        _value_type: *mut u32,
        _timestamp_ms: *mut u64,
    ) -> u32 { 1 }
    pub unsafe fn opc_item_write_sync(_item: *mut c_void, _value: *const c_void, _value_type: u32) -> u32 { 1 }
    pub unsafe fn opc_item_read_async(_item: *mut c_void) -> u32 { 1 }
    pub unsafe fn opc_item_write_async(_item: *mut c_void, _value: *const c_void, _value_type: u32) -> u32 { 1 }
    
    // Utility functions
    pub unsafe fn opc_free_string_array(_strings: *mut *mut u16, _count: u32) { }
    pub unsafe fn opc_free_string(_str: *mut u16) { }
    pub unsafe fn opc_free_string_ansi(_str: *mut i8) { }
    
    // Callback function type
    pub extern "C" fn opc_data_change_callback(
        _user_data: *mut c_void,
        _group_name: *const u16,
        _item_name: *const u16,
        _value: *mut c_void,
        _quality: i32,
        _value_type: u32,
        _timestamp_ms: u64
    ) { }
}

// 工具函数模块
// 
// 这个模块提供字符串转换和资源管理工具函数，用于处理 Rust 字符串和 Windows UTF-16 字符串之间的转换。
mod utils {
    #[cfg(windows)]
    use std::ffi::OsString;
    
    /// 将 Rust 字符串转换为 Windows 宽字符串 (UTF-16)
    /// 
    /// # 参数
    /// - `s`: 要转换的 Rust 字符串
    /// 
    /// # 返回值
    /// - UTF-16 编码的字节向量，以 null 字符结尾
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::to_wide_string;
    /// 
    /// let wide = to_wide_string("Hello");
    /// assert_eq!(wide, vec![72, 101, 108, 108, 111, 0]);
    /// ```
    #[cfg(windows)]
    pub fn to_wide_string(s: &str) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;

        // 将 Rust 字符串转换为 OsString，然后编码为 UTF-16 宽字符序列
        // chain(Some(0)) 添加 null 终止符，这是 C 风格字符串的要求
        OsString::from(s).encode_wide().chain(Some(0)).collect()
    }
    
    /// 将 Rust 字符串转换为 Windows 宽字符串 (UTF-16) - 非 Windows 平台的桩实现
    /// 
    /// # 注意
    /// 在非 Windows 平台上，此函数返回空向量，因为 OPC DA 仅支持 Windows
    #[cfg(not(windows))]
    pub fn to_wide_string(_s: &str) -> Vec<u16> {
        Vec::new()
    }
    
    /// 将 Windows 宽字符串转换为 Rust 字符串
    /// 
    /// # 参数
    /// - `ptr`: 指向 UTF-16 编码的宽字符串的指针，以 null 字符结尾
    /// 
    /// # 返回值
    /// - 转换后的 Rust 字符串。如果指针为空，返回空字符串。
    /// 
    /// # 安全性
    /// - 调用者必须确保指针有效且指向以 null 结尾的 UTF-16 字符串
    /// - 如果字符串包含无效的 UTF-16 序列，将使用替换字符 (U+FFFD)
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::from_wide_string;
    /// 
    /// let wide = vec![72, 101, 108, 108, 111, 0]; // "Hello" in UTF-16
    /// let s = from_wide_string(wide.as_ptr());
    /// assert_eq!(s, "Hello");
    /// ```
    pub fn from_wide_string(ptr: *const u16) -> String {
        if ptr.is_null() {
            return String::new();
        }
        
        unsafe {
            // 计算字符串长度（直到遇到 null 字符）
            let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
            
            // 从指针创建切片
            let slice = std::slice::from_raw_parts(ptr, len);
            
            // 将 UTF-16 转换为 Rust 字符串，处理无效序列
            String::from_utf16_lossy(slice)
        }
    }
    
    /// 释放由 FFI 分配的 Windows 宽字符串
    /// 
    /// # 参数
    /// - `ptr`: 要释放的字符串指针
    /// 
    /// # 注意
    /// - 此函数用于释放由 OPC 库分配的字符串
    /// - 如果指针为空，则不执行任何操作
    /// - 调用此函数后，不应再使用该指针
    pub fn free_wide_string(ptr: *mut u16) {
        if !ptr.is_null() {
            unsafe {
                super::ffi::opc_free_string(ptr);
            }
        }
    }
    
    /// 释放由 FFI 分配的 Windows 宽字符串数组
    /// 
    /// # 参数
    /// - `ptr`: 要释放的字符串数组指针
    /// - `count`: 数组中的字符串数量
    /// 
    /// # 注意
    /// - 此函数用于释放由 OPC 库分配的字符串数组
    /// - 如果指针为空或数量为 0，则不执行任何操作
    /// - 调用此函数后，不应再使用该指针
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

// 测试专用的桩ffi模块
#[cfg(all(not(windows), test))]
mod ffi {
    use std::ffi::c_void;
    
    // 桩函数实现，仅用于测试编译
    // 注意：这些函数的签名必须与Windows版本完全匹配
    
    // 客户端函数
    pub unsafe fn opc_client_init() -> u32 { 0 }
    pub unsafe fn opc_client_stop() { }
    
    // 主机函数
    pub unsafe fn opc_make_host(_hostname: *const u16, _host: *mut *mut c_void) -> u32 { 0 }
    pub unsafe fn opc_host_free(_host: *mut c_void) { }
    pub unsafe fn opc_host_connect_da_server(
        _host: *mut c_void,
        _server_name: *const u16,
        _server: *mut *mut c_void
    ) -> u32 { 0 }
    
    // 服务器函数
    pub unsafe fn opc_server_free(_server: *mut c_void) { }
    pub unsafe fn opc_server_get_status(
        _server: *mut c_void,
        _state: *mut u32,
        _vendor_info: *mut *mut u16
    ) -> u32 { 0 }
    pub unsafe fn opc_server_make_group(
        _server: *mut c_void,
        _group_name: *const u16,
        _active: u32,
        _requested_update_rate: u32,
        _actual_update_rate: *mut u32,
        _deadband: f64,
        _group: *mut *mut c_void
    ) -> u32 { 0 }
    pub unsafe fn opc_server_get_item_names(
        _server: *mut c_void,
        _item_names: *mut *mut *mut u16,
        _count: *mut u32
    ) -> u32 { 0 }
    
    // 组函数
    pub unsafe fn opc_group_free(_group: *mut c_void) { }
    pub unsafe fn opc_group_add_item(
        _group: *mut c_void,
        _item_name: *const u16,
        _item: *mut *mut c_void
    ) -> u32 { 0 }
    pub unsafe fn opc_group_enable_async(
        _group: *mut c_void,
        _callback: extern "C" fn(*mut c_void, *const u16, *const u16, *mut c_void, i32, u32, u64),
        _user_data: *mut c_void
    ) -> u32 { 0 }
    pub unsafe fn opc_group_refresh(_group: *mut c_void) -> u32 { 0 }
    
    // 项函数
    pub unsafe fn opc_item_free(_item: *mut c_void) { }
    pub unsafe fn opc_item_read_sync(
        _item: *mut c_void,
        _value: *mut c_void,
        _quality: *mut i32,
        _value_type: *mut u32,
        _timestamp_ms: *mut u64,
    ) -> u32 { 0 }
    pub unsafe fn opc_item_write_sync(_item: *mut c_void, _value: *const c_void, _value_type: u32) -> u32 { 0 }
    pub unsafe fn opc_item_read_async(_item: *mut c_void) -> u32 { 0 }
    pub unsafe fn opc_item_write_async(_item: *mut c_void, _value: *const c_void, _value_type: u32) -> u32 { 0 }
    
    // 工具函数
    pub unsafe fn opc_free_string_array(_strings: *mut *mut u16, _count: u32) { }
    pub unsafe fn opc_free_string(_str: *mut u16) { }
    pub unsafe fn opc_free_string_ansi(_str: *mut i8) { }
    
    // 回调函数类型
    pub extern "C" fn opc_data_change_callback(
        _user_data: *mut c_void,
        _group_name: *const u16,
        _item_name: *const u16,
        _value: *mut c_void,
        _quality: i32,
        _value_type: u32,
        _timestamp_ms: u64
    ) { }
}