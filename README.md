# OPC DA Client Rust 封装库

一个用于 OPC DA Client Toolkit 的安全、符合 Rust 习惯的封装库，为工业自动化应用程序提供同步和异步 API。

## 特性

- **安全的 Rust API**：围绕 C++ OPC DA 客户端库的内存安全封装
- **同步操作**：用于简单用例的阻塞读写操作
- **订阅支持**：通过回调实现实时数据变更通知
- **类型安全**：具有适当错误处理的强类型值系统
- **跨平台**：仅限 Windows（由于 OPC DA COM 依赖）


## 先决条件

- Windows 操作系统（OPC DA 仅限 Windows）
- Rust 工具链（稳定版）

## 依赖库：
依赖 https://github.com/dushibaiyu/OPC-Client-X64 作为动态库依赖。代码库中以及基于vs2022预编译好一个版本，如有特殊需求，请自行编译。


## 快速入门

### 基本同步用法

```rust
use opc_da_client::{OpcClient, OpcValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let client = OpcClient::new()?;
    
    // 连接到服务器
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    
    // 创建一个组
    let group = server.create_group("TestGroup", true, 1000, 0.0)?;
    
    // 添加一个项目
    let item = group.add_item("Bucket Brigade.UInt2")?;
    
    // 读取值
    let (value, quality) = item.read_sync()?;
    println!("值: {:?}, 质量: {:?}", value, quality);
    
    // 写入值
    item.write_sync(&OpcValue::Int32(12345))?;
    
    Ok(())
}
```


### 订阅示例

```rust
use opc_da_client::{OpcClient, OpcDataCallback, OpcValue, OpcQuality};
use std::sync::Arc;

struct MyCallback;

impl OpcDataCallback for MyCallback {
    fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality) {
        println!("数据已更改: {}:{} = {:?} ({:?})", group_name, item_name, value, quality);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpcClient::new()?;
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    let group = server.create_group("SubGroup", true, 1000, 0.0)?;
    
    let item = group.add_item("Bucket Brigade.UInt2")?;
    let callback = Arc::new(MyCallback);
    
    // 启用订阅
    group.enable_async_subscription(callback)?;
    
    // 保持程序运行以接收回调
    std::thread::sleep(std::time::Duration::from_secs(30));
    
    Ok(())
}
```

## API 参考

### 核心类型

#### `OpcClient` - OPC 客户端
用于管理 OPC 连接的主客户端。负责初始化和清理 OPC 库资源。

**主要方法**:
- `new() -> OpcResult<OpcClient>` - 创建新的 OPC 客户端
- `connect_to_server(hostname, server_name) -> OpcResult<OpcServer>` - 连接到服务器
- `connect_to_local_server(server_name) -> OpcResult<OpcServer>` - 连接到本地服务器
- `is_initialized() -> bool` - 检查客户端是否已初始化

#### `OpcServer` - OPC 服务器
表示到 OPC DA 服务器的活动连接。

**主要方法**:
- `get_status() -> OpcResult<(u32, String)>` - 获取服务器状态和厂商信息
- `create_group(name, active, update_rate, deadband) -> OpcResult<OpcGroup>` - 创建 OPC 组
- `get_item_names() -> OpcResult<Vec<String>>` - 获取所有可用项名

#### `OpcGroup` - OPC 组
OPC 项的容器，具有共享的属性。

**主要方法**:
- `add_item(name) -> OpcResult<OpcItem>` - 向组中添加项
- `enable_async_subscription(callback) -> OpcResult<()>` - 启用异步订阅
- `refresh() -> OpcResult<()>` - 刷新组中的所有项
- `read_sync(item) -> OpcResult<(OpcValue, OpcQuality)>` - 同步读取项值
- `write_sync(item, value) -> OpcResult<()>` - 同步写入项值

#### `OpcItem` - OPC 项
表示单个可读写的数据点。

**主要方法**:
- `read_sync() -> OpcResult<(OpcValue, OpcQuality)>` - 同步读取值
- `write_sync(value) -> OpcResult<()>` - 同步写入值
- `read_async() -> OpcResult<()>` - 异步读取值
- `write_async(value) -> OpcResult<()>` - 异步写入值

#### `OpcValue` - OPC 值类型
支持的数据类型枚举。

**变体**:
- `Int16(i16)` - 16位有符号整数
- `Int32(i32)` - 32位有符号整数
- `Float(f32)` - 32位单精度浮点数
- `Double(f64)` - 64位双精度浮点数
- `String(String)` - UTF-8 字符串

**转换方法**:
- `type_name() -> &'static str` - 获取类型名称
- `raw_type() -> u32` - 获取原始类型代码
- `from_raw(value, value_type) -> Result<OpcValue, OpcValueError>` - 从原始值创建

#### `OpcQuality` - OPC 质量指示器
数据质量状态枚举。

**变体**:
- `Good` - 良好质量，数据可靠
- `Uncertain` - 不确定质量，数据可能有问题
- `Bad` - 不良质量，数据不可靠

**转换方法**:
- `from_raw(quality) -> OpcQuality` - 从原始质量值创建
- `to_raw() -> i32` - 转换为原始质量值

#### `OpcDataCallback` - 异步数据变化回调
异步数据变化通知的回调接口。

**需要实现的方法**:
- `on_data_change(group_name, item_name, value, quality)` - 数据变化时调用

### 错误处理

所有操作都返回 `OpcResult<T>`（`Result<T, OpcError>` 的别名）。

#### 错误类型 (`OpcError`)

- `OperationFailed(String)` - 常规 OPC 操作失败
- `ConnectionFailed(String)` - 连接相关错误
- `InvalidParameters(String)` - 无效参数错误
- `ValueConversionError(OpcValueError)` - 值转换错误
- `ComInitializationFailed(String)` - COM 初始化失败
- `ServerNotFound(String)` - 服务器未找到
- `ItemNotFound(String)` - 项未找到
- `GroupCreationFailed(String)` - 组创建失败
- `AsyncSubscriptionFailed(String)` - 异步订阅失败
- `Timeout(String)` - 操作超时

#### 便捷错误创建方法

- `OpcError::operation_failed(msg)` - 创建操作失败错误
- `OpcError::connection_failed(msg)` - 创建连接失败错误
- `OpcError::invalid_parameters(msg)` - 创建无效参数错误

### 值转换

`OpcValue` 支持 `TryFrom` 转换为 Rust 原生类型：

```rust
use opc_da_client::OpcValue;
use std::convert::TryFrom;

let value = OpcValue::Int32(42);

// 使用 TryFrom trait
let int_value: i32 = i32::try_from(value.clone())?;

// 使用 try_into 方法
let int_value: i32 = value.try_into()?;

// 模式匹配
match value {
    OpcValue::Int32(v) => println!("整数值: {}", v),
    OpcValue::Float(v) => println!("浮点值: {}", v),
    OpcValue::String(v) => println!("字符串值: {}", v),
    _ => println!("其他类型"),
}
```

### 工具函数

- `to_wide_string(s: &str) -> Vec<u16>` - 将 Rust 字符串转换为 UTF-16 宽字符串
- `from_wide_string(ptr: *const u16) -> String` - 将 UTF-16 宽字符串转换为 Rust 字符串
- `connect_to_server(server_name) -> OpcResult<OpcServer>` - 便捷函数：连接到本地服务器
- `connect_to_server_on_host(hostname, server_name) -> OpcResult<OpcServer>` - 便捷函数：连接到远程服务器

## 从源代码构建

1. 克隆仓库：
   ```bash
   git clone https://github.com/yourusername/OPC-Client-X64.git
   cd OPC-Client-X64/rust-wrapper
   ```

2. 构建库：
   ```bash
   cargo build --release
   ```

3. 运行示例：
   ```bash
   cargo run --example basic_example
   cargo run --example subscription_example
   ```

## 测试

测试套件包括单元测试和集成测试：

```bash
# 运行单元测试
cargo test

# 运行所有测试（包括需要 OPC 服务器的集成测试）
cargo test -- --include-ignored
```

注意：集成测试需要运行 OPC 服务器（如 MatrikonOPC Simulation Server）。

## 架构设计

### 模块结构

```
src/
├── lib.rs              # 库主入口，FFI 绑定和工具函数
├── client.rs           # OPC 客户端，连接管理
├── server.rs           # OPC 服务器，状态和组管理
├── group.rs            # OPC 组，项管理和订阅
├── item.rs             # OPC 项，读写操作
├── types.rs            # 核心类型（值、质量、回调）
├── error.rs            # 错误类型和处理
└── utils.rs            # 字符串转换工具（内部）
```

### 设计原则

1. **RAII (Resource Acquisition Is Initialization)**
   - 所有资源（客户端、服务器、组、项）都使用 RAII 模式管理
   - 对象离开作用域时自动清理资源
   - 确保没有资源泄漏

2. **类型安全**
   - 使用 Rust 的强类型系统确保类型安全
   - 将原始指针包装在安全的 Rust 类型中
   - 提供类型转换和验证

3. **错误处理**
   - 所有可能失败的操作都返回 `Result` 类型
   - 提供详细的错误信息和错误链
   - 支持 `?` 操作符进行错误传播

4. **线程安全**
   - 明确标记线程安全的类型
   - 回调对象需要实现 `Send + Sync`
   - 文档说明线程限制

### FFI 层

库通过 FFI (Foreign Function Interface) 调用底层的 C++ OPC 库：

1. **安全包装**: 所有 FFI 调用都包装在 `unsafe` 块中
2. **类型转换**: 处理 Rust 类型和 C 类型之间的转换
3. **资源管理**: 确保 FFI 分配的资源正确释放
4. **错误转换**: 将 C 错误码转换为 Rust 错误类型

## 最佳实践

### 1. 连接管理

```rust
// 正确：使用 RAII 管理连接
{
    let client = OpcClient::new()?;
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    // 使用 server...
} // 自动清理资源

// 避免：手动管理资源（容易泄漏）
```

### 2. 错误处理

```rust
// 正确：使用 ? 操作符传播错误
fn read_value() -> OpcResult<i32> {
    let client = OpcClient::new()?;
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    let group = server.create_group("Test", true, 1000, 0.0)?;
    let item = group.add_item("Bucket Brigade.UInt2")?;
    let (value, quality) = item.read_sync()?;
    
    match value {
        OpcValue::Int32(v) => Ok(v),
        _ => Err(OpcError::operation_failed("类型不匹配")),
    }
}

// 避免：忽略错误或使用 unwrap()
```

### 3. 异步订阅

```rust
use std::sync::Arc;
use opc_da_client::{OpcDataCallback, OpcValue, OpcQuality};

struct DataLogger;

impl OpcDataCallback for DataLogger {
    fn on_data_change(&self, group_name: &str, item_name: &str, 
                      value: OpcValue, quality: OpcQuality) {
        // 注意：回调可能在后台线程中调用
        println!("[{}/{}] {:?} ({:?})", group_name, item_name, value, quality);
    }
}

// 启用订阅
let callback = Arc::new(DataLogger);
group.enable_async_subscription(callback)?;
```

### 4. 性能优化

```rust
// 批量操作：使用组进行批量读写
let group = server.create_group("BatchGroup", false, 0, 0.0)?;
let items: Vec<OpcItem> = item_names.iter()
    .filter_map(|name| group.add_item(name).ok())
    .collect();

// 定期刷新而不是频繁读取
group.refresh()?;

// 使用合适的更新速率
let monitoring_group = server.create_group("Monitor", true, 1000, 0.1)?; // 1秒更新，10%死区
```

### 5. 资源清理

```rust
// 正确：依赖 RAII 自动清理
fn process_data() -> OpcResult<()> {
    let client = OpcClient::new()?;
    let server = client.connect_to_server("192.168.1.100", "MyOPCServer")?;
    
    // 创建临时组
    {
        let temp_group = server.create_group("Temp", true, 500, 0.0)?;
        // 使用 temp_group...
    } // temp_group 自动销毁
    
    Ok(())
}

// 手动提前释放（如果需要）
let group = server.create_group("MyGroup", true, 1000, 0.0)?;
// 使用 group...
drop(group); // 显式释放
```

## 常见问题

### Q: 为什么只能在 Windows 上使用？
A: OPC DA (Data Access) 是基于 Windows COM 技术的标准，因此仅支持 Windows 平台。

### Q: 如何连接到远程服务器？
A: 使用 `connect_to_server("hostname", "server_name")`，需要配置 DCOM 权限。

### Q: 异步回调在哪个线程中调用？
A: 回调可能在 OPC 库的后台线程中调用，确保回调函数是线程安全的。

### Q: 如何处理连接中断？
A: 库会返回 `ConnectionFailed` 错误，应用程序需要处理重连逻辑。

### Q: 支持哪些数据类型？
A: 支持 Int16、Int32、Float、Double、String 等基本类型。

### Q: 如何获取服务器中的所有项？
A: 使用 `server.get_item_names()` 方法浏览服务器命名空间。

## 故障排除

### 连接问题
1. 确保 OPC 服务器已安装并运行
2. 检查服务器名称是否正确
3. 验证 DCOM 配置和防火墙设置
4. 确认有足够的权限

### 性能问题
1. 调整组的更新速率和死区值
2. 使用批量操作减少网络往返
3. 避免频繁创建和销毁对象
4. 使用合适的缓存策略

### 内存问题
1. 确保及时释放不再使用的对象
2. 监控回调函数的内存使用
3. 避免在回调中执行耗时操作
4. 定期检查资源泄漏

## 限制

- **仅限 Windows**：OPC DA 是 Windows COM 技术
- **字符串支持**：当前实现中的字符串读写操作有限
- **异步实现**：当前的异步 API 是同步操作的封装

## 贡献

1. Fork 仓库
2. 创建一个功能分支
3. 进行您的更改
4. 添加测试
5. 运行 `cargo test`
6. 提交 pull request

