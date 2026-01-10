# 快速开始

本指南将帮助您快速开始使用 OPCDaclientRs 库。

## 安装

### 通过 Cargo.toml 添加依赖

在您的 `Cargo.toml` 文件中添加依赖：

```toml
[dependencies]
opc_da_client = { git = "https://github.com/dushibaiyu/OPCDaclientRs.git" }
```

### 平台要求

**重要**: OPC DA 是基于 Windows COM 技术的标准，因此本库仅在 Windows 平台上可用。

- **操作系统**: Windows 7/8/10/11, Windows Server 2008R2 及以上
- **架构**: x86 (32位) 或 x86_64 (64位)
- **运行时**: 需要安装 OPC DA Core Components

## 第一个示例

创建一个简单的 OPC DA 客户端应用程序：

```rust
use opc_da_client::{OpcClient, OpcValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 OPC 客户端
    let client = OpcClient::new()?;
    println!("OPC 客户端初始化成功");
    
    // 2. 连接到本地 OPC 服务器
    //    常见的仿真服务器: "Matrikon.OPC.Simulation.1", "OPCSim.KEPServerEX.V6"
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    println!("连接到 OPC 服务器成功");
    
    // 3. 获取服务器状态
    let (state, vendor_info) = server.get_status()?;
    println!("服务器状态: {}, 厂商: {}", state, vendor_info);
    
    // 4. 创建 OPC 组
    //    参数: 组名, 是否激活, 请求更新速率(ms), 死区值
    let group = server.create_group("TestGroup", true, 1000, 0.0)?;
    println!("创建 OPC 组成功");
    
    // 5. 添加 OPC 项
    //    项名格式通常为: "设备名.变量名" 或 "命名空间.变量名"
    let item = group.add_item("Bucket Brigade.UInt2")?;
    println!("添加 OPC 项成功");
    
    // 6. 同步读取值
    let (value, quality) = item.read_sync()?;
    println!("读取值: {:?}, 质量: {:?}", value, quality);
    
    // 7. 同步写入值
    item.write_sync(&OpcValue::Int32(12345))?;
    println!("写入值成功");
    
    // 8. 再次读取验证
    let (updated_value, updated_quality) = item.read_sync()?;
    println!("更新后的值: {:?}, 质量: {:?}", updated_value, updated_quality);
    
    println!("示例程序执行成功!");
    Ok(())
}
```

## 运行示例

1. **确保 OPC 服务器运行**:
   - 安装 OPC 仿真服务器（如 MatrikonOPC Simulation Server）
   - 启动仿真服务器

2. **运行程序**:
   ```bash
   cargo run --example basic_example
   ```

3. **预期输出**:
   ```
   OPC 客户端初始化成功
   连接到 OPC 服务器成功
   服务器状态: 1, 厂商: MatrikonOPC
   创建 OPC 组成功
   添加 OPC 项成功
   读取值: Int32(42), 质量: Good
   写入值成功
   更新后的值: Int32(12345), 质量: Good
   示例程序执行成功!
   ```

## 下一步

- 查看 [高级示例](./examples/advanced.md) 了解复杂用法
- 学习 [异步编程](./examples/async.md) 实现实时数据监控
- 阅读 [API 参考](./api/client.md) 了解所有可用功能
- 查看 [最佳实践](./best-practices/) 获取性能优化建议

## 故障排除

如果遇到问题：

1. **连接失败**:
   - 确保 OPC 服务器正在运行
   - 检查服务器名称是否正确
   - 验证 DCOM 配置

2. **编译错误**:
   - 确保在 Windows 平台上编译
   - 检查 Rust 版本（需要 1.70+）

3. **运行时错误**:
   - 查看错误信息中的详细描述
   - 参考 [错误处理指南](./examples/error-handling.md)

## 获取帮助

- 查看 [常见问题](./troubleshooting/faq.md)
- 在 [GitHub Issues](https://github.com/dushibaiyu/OPCDaclientRs/issues) 提交问题
- 查看示例代码获取更多使用示例