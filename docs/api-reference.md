# API 参考文档

本文档提供了 OPCDaclientRs 库的完整 API 参考。

## 模块概览

### 主要模块
- `opc_da_client` - 库主模块，重新导出所有公共类型
- `client` - OPC 客户端管理
- `server` - OPC 服务器连接
- `group` - OPC 组管理  
- `item` - OPC 项读写
- `types` - 核心数据类型
- `error` - 错误处理

### 工具模块
- `utils` - 字符串转换工具（内部使用）

## 客户端 API

### OpcClient 结构体

OPC 客户端的主入口点，管理客户端生命周期和服务器连接。

#### 构造函数

```rust
impl OpcClient {
    /// 创建新的 OPC 客户端
    ///
    /// # 返回值
    /// - `Ok(OpcClient)`: 客户端创建成功
    /// - `Err(OpcError)`: 客户端创建失败
    ///
    /// # 示例
    /// ```
    /// use opc_da_client::OpcClient;
    ///
    /// let client = OpcClient::new()?;
    /// ```
    pub fn new() -> OpcResult<Self>
}
```

#### 方法

```rust
impl OpcClient {
    /// 连接到远程 OPC 服务器
    ///
    /// # 参数
    /// - `hostname`: 服务器主机名
    /// - `server_name`: OPC 服务器名称
    ///
    /// # 返回值
    /// - `Ok(OpcServer)`: 服务器连接成功
    /// - `Err(OpcError)`: 连接失败
    ///
    /// # 示例
    /// ```
    /// let server = client.connect_to_server("192.168.1.100", "Matrikon.OPC.Simulation.1")?;
    /// ```
    pub fn connect_to_server(
        &self,
        hostname: &str,
        server_name: &str
    ) -> OpcResult<OpcServer>

    /// 连接到本地 OPC 服务器
    ///
    /// # 参数
    /// - `server_name`: OPC 服务器名称
    ///
    /// # 返回值
    /// - `Ok(OpcServer)`: 服务器连接成功
    /// - `Err(OpcError)`: 连接失败
    ///
    /// # 示例
    /// ```
    /// let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    /// ```
    pub fn connect_to_local_server(
        &self,
        server_name: &str
    ) -> OpcResult<OpcServer>
}
```

## 服务器 API

### OpcServer 结构体

表示 OPC 服务器连接，提供服务器操作和组管理功能。

#### 方法

```rust
impl OpcServer {
    /// 获取服务器状态信息
    ///
    /// # 返回值
    /// - `Ok((state, vendor_info))`: 状态获取成功
    ///   - `state`: 服务器状态码
    ///   - `vendor_info`: 厂商信息字符串
    /// - `Err(OpcError)`: 状态获取失败
    ///
    /// # 示例
    /// ```
    /// let (state, vendor) = server.get_status()?;
    /// println!("服务器状态: {}, 厂商: {}", state, vendor);
    /// ```
    pub fn get_status(&self) -> OpcResult<(u32, String)>

    /// 创建新的 OPC 组
    ///
    /// # 参数
    /// - `name`: 组名称
    /// - `active`: 是否激活组
    /// - `requested_update_rate`: 请求的更新速率（毫秒）
    /// - `deadband`: 死区值（0.0-100.0）
    ///
    /// # 返回值
    /// - `Ok(OpcGroup)`: 组创建成功
    /// - `Err(OpcError)`: 组创建失败
    ///
    /// # 示例
    /// ```
    /// let group = server.create_group("DataGroup", true, 1000, 0.0)?;
    /// ```
    pub fn create_group(
        &self,
        name: &str,
        active: bool,
        requested_update_rate: u32,
        deadband: f64
    ) -> OpcResult<OpcGroup>

    /// 获取服务器上可用的项名称列表
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 项名称列表
    /// - `Err(OpcError)`: 获取失败
    ///
    /// # 示例
    /// ```
    /// let items = server.get_item_names()?;
    /// for item in items {
    ///     println!("可用项: {}", item);
    /// }
    /// ```
    pub fn get_item_names(&self) -> OpcResult<Vec<String>>
}
```

## 组 API

### OpcGroup 结构体

表示 OPC 数据组，管理组属性和数据项。

#### 方法

```rust
impl OpcGroup {
    /// 向组中添加 OPC 项
    ///
    /// # 参数
    /// - `item_name`: OPC 项名称
    ///
    /// # 返回值
    /// - `Ok(OpcItem)`: 项添加成功
    /// - `Err(OpcError)`: 项添加失败
    ///
    /// # 示例
    /// ```
    /// let item = group.add_item("Bucket Brigade.UInt2")?;
    /// ```
    pub fn add_item(&self, item_name: &str) -> OpcResult<OpcItem>

    /// 启用异步数据变化订阅
    ///
    /// # 参数
    /// - `callback`: 数据变化回调处理器
    ///
    /// # 返回值
    /// - `Ok(())`: 订阅启用成功
    /// - `Err(OpcError)`: 订阅启用失败
    ///
    /// # 示例
    /// ```
    /// struct MyCallback;
    /// impl OpcDataCallback for MyCallback {
    ///     fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality) {
    ///         println!("数据变化: {} - {} = {:?}", group_name, item_name, value);
    ///     }
    /// }
    ///
    /// group.enable_async_subscription(Box::new(MyCallback))?;
    /// ```
    pub fn enable_async_subscription(
        &self,
        callback: Box<dyn OpcDataCallback>
    ) -> OpcResult<()>

    /// 手动刷新组数据
    ///
    /// # 返回值
    /// - `Ok(())`: 刷新成功
    /// - `Err(OpcError)`: 刷新失败
    ///
    /// # 示例
    /// ```
    /// group.refresh()?;
    /// ```
    pub fn refresh(&self) -> OpcResult<()>
}
```

## 项 API

### OpcItem 结构体

表示 OPC 数据项，提供数据读写功能。

#### 方法

```rust
impl OpcItem {
    /// 同步读取项值
    ///
    /// # 返回值
    /// - `Ok((OpcValue, OpcQuality))`: 读取成功
    ///   - 第一个元素: 项值
    ///   - 第二个元素: 数据质量
    /// - `Err(OpcError)`: 读取失败
    ///
    /// # 示例
    /// ```
    /// let (value, quality) = item.read_sync()?;
    /// println!("值: {:?}, 质量: {:?}", value, quality);
    /// ```
    pub fn read_sync(&self) -> OpcResult<(OpcValue, OpcQuality)>

    /// 同步写入项值
    ///
    /// # 参数
    /// - `value`: 要写入的值
    ///
    /// # 返回值
    /// - `Ok(())`: 写入成功
    /// - `Err(OpcError)`: 写入失败
    ///
    /// # 示例
    /// ```
    /// item.write_sync(&OpcValue::Int32(100))?;
    /// ```
    pub fn write_sync(&self, value: &OpcValue) -> OpcResult<()>

    /// 异步读取项值
    ///
    /// # 返回值
    /// - `Ok(())`: 异步读取请求发送成功
    /// - `Err(OpcError)`: 请求发送失败
    ///
    /// # 注意
    /// 结果将通过 `OpcDataCallback` 回调返回
    ///
    /// # 示例
    /// ```
    /// item.read_async()?;
    /// ```
    pub fn read_async(&self) -> OpcResult<()>

    /// 异步写入项值
    ///
    /// # 参数
    /// - `value`: 要写入的值
    ///
    /// # 返回值
    /// - `Ok(())`: 异步写入请求发送成功
    /// - `Err(OpcError)`: 请求发送失败
    ///
    /// # 注意
    /// 写入结果将通过 `OpcDataCallback` 回调返回
    ///
    /// # 示例
    /// ```
    /// item.write_async(&OpcValue::Float(3.14))?;
    /// ```
    pub fn write_async(&self, value: &OpcValue) -> OpcResult<()>
}
```

## 核心类型 API

### OpcValue 枚举

表示 OPC 数据值，支持多种数据类型。

#### 变体

```rust
pub enum OpcValue {
    /// 16位有符号整数
    Int16(i16),
    
    /// 32位有符号整数
    Int32(i32),
    
    /// 单精度浮点数
    Float(f32),
    
    /// 双精度浮点数
    Double(f64),
    
    /// 字符串
    String(String),
}
```

#### 方法

```rust
impl OpcValue {
    /// 获取值的类型名称
    ///
    /// # 返回值
    /// 类型名称字符串
    ///
    /// # 示例
    /// ```
    /// let value = OpcValue::Int32(42);
    /// assert_eq!(value.type_name(), "Int32");
    /// ```
    pub fn type_name(&self) -> &'static str

    /// 获取原始值类型代码（用于 FFI）
    ///
    /// # 返回值
    /// 原始类型代码
    ///
    /// # 示例
    /// ```
    /// let value = OpcValue::Float(3.14);
    /// assert_eq!(value.raw_type(), 2);
    /// ```
    pub fn raw_type(&self) -> u32

    /// 从原始值和类型创建 OpcValue
    ///
    /// # 参数
    /// - `value`: 原始值指针
    /// - `value_type`: 值类型代码
    ///
    /// # 返回值
    /// - `Ok(OpcValue)`: 创建成功
    /// - `Err(OpcValueError)`: 创建失败
    ///
    /// # 安全要求
    /// 调用者必须确保 `value` 指针有效且类型匹配
    pub fn from_raw(
        value: *mut std::ffi::c_void,
        value_type: u32
    ) -> Result<Self, OpcValueError>
}
```

#### 类型转换

`OpcValue` 支持通过 `TryFrom` trait 进行类型转换：

```rust
// 从 OpcValue 转换到具体类型
let opc_value = OpcValue::Int32(100);
let int_value: i32 = opc_value.try_into()?;

// 从具体类型转换到 OpcValue
let int_value = 100;
let opc_value = OpcValue::from(int_value);
```

支持的类型转换：
- `i16` ↔ `OpcValue::Int16`
- `i32` ↔ `OpcValue::Int32`
- `f32` ↔ `OpcValue::Float`
- `f64` ↔ `OpcValue::Double`
- `String` ↔ `OpcValue::String`

### OpcQuality 枚举

表示 OPC 数据质量。

#### 变体

```rust
pub enum OpcQuality {
    /// 良好质量数据
    Good,
    
    /// 不确定质量数据
    Uncertain,
    
    /// 不良质量数据
    Bad,
}
```

#### 方法

```rust
impl OpcQuality {
    /// 从原始质量值创建 OpcQuality
    ///
    /// # 参数
    /// - `quality`: 原始质量值
    ///
    /// # 返回值
    /// 对应的 OpcQuality 值
    ///
    /// # 示例
    /// ```
    /// let quality = OpcQuality::from_raw(192);
    /// assert_eq!(quality, OpcQuality::Good);
    /// ```
    pub fn from_raw(quality: i32) -> Self

    /// 转换为原始质量值
    ///
    /// # 返回值
    /// 原始质量值
    ///
    /// # 示例
    /// ```
    /// let quality = OpcQuality::Good;
    /// assert_eq!(quality.to_raw(), 192);
    /// ```
    pub fn to_raw(&self) -> i32
}
```

### OpcDataCallback trait

异步数据变化回调接口。

```rust
pub trait OpcDataCallback: Send + Sync {
    /// 数据变化回调方法
    ///
    /// # 参数
    /// - `group_name`: 组名称
    /// - `item_name`: 项名称
    /// - `value`: 新的值
    /// - `quality`: 数据质量
    ///
    /// # 示例
    /// ```
    /// struct MyCallback;
    /// impl OpcDataCallback for MyCallback {
    ///     fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality) {
    ///         println!("[{}] {} = {:?} (质量: {:?})", group_name, item_name, value, quality);
    ///     }
    /// }
    /// ```
    fn on_data_change(
        &self,
        group_name: &str,
        item_name: &str,
        value: OpcValue,
        quality: OpcQuality
    );
}
```

## 错误处理 API

### OpcError 枚举

OPC 操作错误类型。

#### 变体

```rust
pub enum OpcError {
    /// 常规 OPC 操作错误
    OperationFailed(String),
    
    /// 连接相关错误
    ConnectionFailed(String),
    
    /// 无效参数错误
    InvalidParameters(String),
    
    /// 值转换错误
    ValueConversionError(OpcValueError),
    
    /// COM 初始化错误
    ComInitializationFailed(String),
    
    /// 服务器未找到错误
    ServerNotFound(String),
    
    /// 项未找到错误
    ItemNotFound(String),
    
    /// 组创建错误
    GroupCreationFailed(String),
    
    /// 异步订阅错误
    AsyncSubscriptionFailed(String),
    
    /// 超时错误
    Timeout(String),
}
```

#### 便捷方法

```rust
impl OpcError {
    /// 创建操作失败错误
    pub fn operation_failed(msg: impl Into<String>) -> Self
    
    /// 创建连接失败错误
    pub fn connection_failed(msg: impl Into<String>) -> Self
    
    /// 创建无效参数错误
    pub fn invalid_parameters(msg: impl Into<String>) -> Self
}
```

### OpcResult 类型别名

```rust
/// OPC 操作结果类型
/// 这是 `Result<T, OpcError>` 的类型别名
pub type OpcResult<T> = Result<T, OpcError>;
```

### OpcValueError 枚举

值转换错误类型。

```rust
pub enum OpcValueError {
    /// 类型不匹配错误
    TypeMismatch {
        expected: String,
        actual: String,
    },
    
    /// 空指针错误
    NullPointer,
    
    /// 不支持的类型错误
    UnsupportedType(u32),
}
```

## 重新导出的类型

库主模块重新导出了所有主要类型：

```rust
// 客户端相关
pub use client::OpcClient;
pub use server::OpcServer;
pub use group::OpcGroup;
pub use item::OpcItem;

// 核心类型
pub use types::{OpcValue, OpcQuality, OpcDataCallback};

// 错误处理
pub use error::{OpcError, OpcResult};
```

## 使用示例

### 基本使用

```rust
use opc_da_client::{OpcClient, OpcValue};

fn main() -> OpcResult<()> {
    // 创建客户端
    let client = OpcClient::new()?;
    
    // 连接服务器
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    
    // 创建组
    let group = server.create_group("TestGroup", true, 1000, 0.0)?;
    
    // 添加项
    let item = group.add_item("Bucket Brigade.UInt2")?;
    
    // 读写操作
    let (value, quality) = item.read_sync()?;
    item.write_sync(&OpcValue::Int32(100))?;
    
    Ok(())
}
```

### 异步使用

```rust
use opc_da_client::{OpcClient, OpcValue, OpcDataCallback, OpcQuality};
use std::sync::Arc;

struct DataLogger;

impl OpcDataCallback for DataLogger {
    fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality) {
        println!("[{}] {} = {:?} (质量: {:?})", group_name, item_name, value, quality);
    }
}

fn main() -> OpcResult<()> {
    let client = OpcClient::new()?;
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    let group = server.create_group("MonitorGroup", true, 500, 0.0)?;
    
    // 启用异步订阅
    group.enable_async_subscription(Box::new(DataLogger))?;
    
    // 添加监控项
    group.add_item("Random.Int2")?;
    group.add_item("Triangle Waves.Real4")?;
    
    // 手动刷新触发数据更新
    group.refresh()?;
    
    // 等待数据回调
    std::thread::sleep(std::time::Duration::from_secs(10));
    
    Ok(())
}
```

### 错误处理

```rust
use opc_da_client::{OpcClient, OpcError};

fn connect_with_retry(server_name: &str, max_retries: u32) -> OpcResult<()> {
    let client = OpcClient::new()?;
    
    for attempt in 1..=max_retries {
        match client.connect_to_local_server(server_name) {
            Ok(server) => {
                println!("连接成功!");
                return Ok(());
            }
            Err(OpcError::ConnectionFailed(msg)) => {
                println!("尝试 {} 失败: {}", attempt, msg);
                if attempt < max_retries {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(OpcError::connection_failed("达到最大重试次数"))
}
```

## 平台限制

### 支持的平台
- **操作系统**: Windows 7/8/10/11, Windows Server 2008R2+
- **架构**: x86 (32位), x86_64 (64位)

### 编译时检查
库包含编译时平台检查，在非 Windows 平台会编译失败（除非使用文档生成模式）。

### 运行时要求
- Windows COM 系统
- OPC DA Core Components
- 网络访问权限（远程服务器时）

## 性能注意事项

### 最佳实践
1. **重用连接**: 尽可能重用 `OpcClient` 实例
2. **批量操作**: 使用组操作减少单独项操作
3. **异步订阅**: 对于实时监控使用异步订阅
4. **适当超时**: 根据网络状况设置合理的超时

### 资源管理
所有资源类型都实现 `Drop` trait，但建议显式释放：
- 及时释放不再需要的项和组
- 避免长时间持有服务器连接
- 使用作用域限制资源生命周期

## 版本兼容性

### API 稳定性
- 主要版本（1.x）保持 API 向后兼容
- 次要版本（x.1）可能添加新功能
- 修订版本（x.x.1）仅修复错误

### 依赖关系
- Rust: 1.70+
- OPC-Client-X64: 预编译库包含在项目中
- Windows: 特定版本要求见平台限制