# 架构设计

本文档描述了 OPCDaclientRs 库的架构设计。

## 总体架构

OPCDaclientRs 采用分层架构设计，将底层的 C++ OPC 库封装为安全、符合 Rust 习惯的 API。

```text
┌─────────────────────────────────────────┐
│           Rust 应用程序层                │
│  (使用安全、符合 Rust 习惯的 API)        │
└─────────────────────────────────────────┘
                   │
┌─────────────────────────────────────────┐
│           安全包装层                     │
│  (RAII 模式、错误处理、类型安全)         │
└─────────────────────────────────────────┘
                   │
┌─────────────────────────────────────────┐
│           FFI 绑定层                     │
│  (extern "C" 函数、原始指针处理)         │
└─────────────────────────────────────────┘
                   │
┌─────────────────────────────────────────┐
│        OPC-Client-X64 C++ 库            │
│  (预编译的 Windows COM OPC DA 库)       │
└─────────────────────────────────────────┘
```

## 核心组件

### 1. FFI 绑定层 (`src/lib.rs` 中的 `ffi` 模块)

**职责**:
- 声明与 C++ 库交互的 `extern "C"` 函数
- 处理原始指针和内存管理
- 提供字符串转换工具函数

**关键特性**:
- 使用 `#[link]` 属性链接预编译的 OPC 库
- 提供宽字符串（UTF-16）转换工具
- 实现安全的资源释放函数

### 2. 安全包装层

#### 客户端 (`src/client.rs`)
```rust
pub struct OpcClient {
    // 私有字段确保 RAII
}
```

**职责**:
- 管理 OPC 客户端生命周期
- 提供服务器连接功能
- 实现 `Drop` trait 确保资源清理

#### 服务器 (`src/server.rs`)
```rust
pub struct OpcServer {
    ptr: *mut c_void,
    host_ptr: *mut c_void,
}
```

**职责**:
- 封装 OPC 服务器连接
- 提供组创建和项浏览功能
- 管理服务器状态信息

#### 组 (`src/group.rs`)
```rust
pub struct OpcGroup {
    ptr: *mut c_void,
    server_ptr: *mut c_void,
}
```

**职责**:
- 管理 OPC 组生命周期
- 提供项添加和异步订阅功能
- 实现组属性设置

#### 项 (`src/item.rs`)
```rust
pub struct OpcItem {
    ptr: *mut c_void,
    group_ptr: *mut c_void,
}
```

**职责**:
- 封装 OPC 数据项
- 提供同步/异步读写操作
- 管理项值类型转换

### 3. 核心类型系统 (`src/types.rs`)

#### OpcValue 枚举
```rust
pub enum OpcValue {
    Int16(i16),
    Int32(i32),
    Float(f32),
    Double(f64),
    String(String),
}
```

**特性**:
- 类型安全的 OPC 值表示
- 支持 `TryFrom` trait 进行类型转换
- 提供原始值转换方法

#### OpcQuality 枚举
```rust
pub enum OpcQuality {
    Good,        // 良好质量
    Uncertain,   // 不确定质量
    Bad,         // 不良质量
}
```

**特性**:
- 表示 OPC 数据质量
- 支持原始值转换
- 实现 `Display` trait

#### OpcDataCallback trait
```rust
pub trait OpcDataCallback: Send + Sync {
    fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality, timestamp: u64);
}
```

**特性**:
- 异步数据变化回调接口
- 要求 `Send + Sync` 以支持多线程
- 提供类型安全的回调机制

### 4. 错误处理系统 (`src/error.rs`)

#### OpcError 枚举
```rust
pub enum OpcError {
    OperationFailed(String),
    ConnectionFailed(String),
    InvalidParameters(String),
    ValueConversionError(OpcValueError),
    // ... 其他错误变体
}
```

**特性**:
- 详细的错误分类
- 支持错误链（通过 `thiserror`）
- 提供便捷的错误创建方法

#### OpcResult 类型别名
```rust
pub type OpcResult<T> = Result<T, OpcError>;
```

**特性**:
- 统一的返回类型
- 支持 `?` 操作符错误传播
- 与 Rust 标准错误处理兼容

## 内存安全设计

### RAII 模式
所有资源管理类型都实现 `Drop` trait，确保资源自动释放：

```rust
impl Drop for OpcServer {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                crate::ffi::opc_server_free(self.ptr);
            }
            if !self.host_ptr.is_null() {
                crate::ffi::opc_host_free(self.host_ptr);
            }
        }
    }
}
```

### 所有权系统
- **客户端** 拥有 **服务器** 的生命周期
- **服务器** 拥有 **组** 的生命周期  
- **组** 拥有 **项** 的生命周期
- 防止悬垂指针和双重释放

### 线程安全
- 所有公共类型都实现 `Send` trait
- 回调 trait 要求 `Send + Sync`
- 内部使用 `Arc` 和 `Mutex` 进行线程安全的数据共享

## 性能考虑

### 1. 零成本抽象
- FFI 调用直接映射到 C++ 函数
- 类型转换在编译时检查
- 避免不必要的内存分配

### 2. 批量操作支持
- 支持批量读取/写入操作
- 减少 FFI 调用次数
- 优化网络通信

### 3. 异步支持
- 提供异步数据订阅
- 支持回调机制
- 避免轮询开销

## 扩展性设计

### 1. 模块化结构
每个组件都是独立的模块，便于：
- 单独测试
- 功能扩展
- 代码维护

### 2. trait 系统
使用 trait 提供扩展点：
- `OpcDataCallback` 支持自定义回调
- `TryFrom` 支持类型转换扩展
- `Display` 支持自定义格式化

### 3. 配置选项
通过结构体字段提供配置：
- 组更新速率
- 死区值设置
- 异步订阅参数

## 平台兼容性

### Windows 特定实现
```rust
#[cfg(windows)]
mod ffi {
    // Windows 特定的 FFI 绑定
}
```

### 跨平台编译支持
```rust
#[cfg(all(not(windows), test))]
mod ffi {
    // 测试专用的桩实现
}
```

### 构建系统
- `build.rs` 处理平台检测
- 条件编译支持非 Windows 平台编译
- 预编译库包含在项目中

## 设计原则

### 1. 安全性优先
- 所有不安全的代码都封装在安全接口后面
- 使用 Rust 的类型系统防止常见错误
- 实现完整的错误处理

### 2. 符合 Rust 习惯
- 使用 `Result` 类型进行错误处理
- 实现 `Drop` trait 管理资源
- 提供符合 Rust 习惯的 API 设计

### 3. 文档完整性
- 所有公共 API 都有完整的中文文档
- 提供使用示例和最佳实践
- 包含平台限制和注意事项

### 4. 测试覆盖
- 单元测试验证核心逻辑
- 集成测试验证 FFI 交互
- 示例代码作为功能测试

## 未来扩展

### 计划的功能
1. **更多 OPC 规范支持**
   - OPC UA 支持
   - OPC HDA 支持
   - OPC A&E 支持

2. **性能优化**
   - 连接池支持
   - 批量操作优化
   - 内存使用优化

3. **工具集成**
   - CLI 工具
   - Web 界面
   - 监控仪表板

### 架构演进
- 插件系统支持
- 配置驱动设计
- 可扩展的传输层