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

## API 概述

### 核心类型

- `OpcClient`：用于管理连接的主客户端
- `OpcServer`：表示到 OPC 服务器的连接
- `OpcGroup`：具有共享属性的 OPC 项目容器
- `OpcItem`：可读写的单个数据点
- `OpcValue`：表示支持的数据类型（Int16, Int32, Float, Double, String）的枚举
- `OpcQuality`：数据质量指示器（良好、不确定、错误）

### 错误处理

所有操作都返回 `OpcResult<T>`，它是 `Result<T, OpcError>`。常见的错误类型包括：

- `OperationFailed`：常规 OPC 操作失败
- `ConnectionFailed`：服务器连接问题
- `InvalidParameters`：传递给函数的参数无效
- `ValueConversionError`：类型转换失败

### 值转换

`OpcValue` 枚举支持 `TryFrom` 转换为 Rust 原始类型：

```rust
let value = OpcValue::Int32(42);
let int_value: i32 = value.try_into()?;
```

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

## 架构

该封装库组织为几个模块：

- `client.rs`：主客户端和连接管理
- `server.rs`：服务器操作和状态
- `group.rs`：组管理和订阅
- `item.rs`：项目读写操作
- `types.rs`：核心数据类型和转换
- `error.rs`：错误类型和处理
- `utils.rs`：字符串转换工具

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

