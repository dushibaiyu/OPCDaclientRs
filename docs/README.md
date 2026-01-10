# OPCDaclientRs 文档

这是 OPCDaclientRs 库的完整文档。OPCDaclientRs 是一个为 OPC DA Client Toolkit 提供的安全、符合 Rust 习惯的封装库。

## 文档目录

### 1. 入门指南
- [快速开始](./getting-started.md) - 如何安装和使用库
- [安装指南](./installation.md) - 系统要求和安装步骤
- [平台支持](./platform-support.md) - 支持的平台和架构

### 2. 核心概念
- [OPC DA 简介](./opc-da-intro.md) - OPC DA 技术概述
- [架构设计](./architecture.md) - 库的架构设计
- [核心类型](./core-types.md) - 主要数据类型和结构

### 3. API 参考
- [客户端 API](./api/client.md) - OpcClient 相关 API
- [服务器 API](./api/server.md) - OpcServer 相关 API  
- [组 API](./api/group.md) - OpcGroup 相关 API
- [项 API](./api/item.md) - OpcItem 相关 API
- [类型 API](./api/types.md) - 核心类型 API
- [错误处理](./api/error.md) - 错误类型和处理

### 4. 使用示例
- [基础示例](./examples/basic.md) - 基本用法示例
- [高级示例](./examples/advanced.md) - 复杂场景示例
- [异步编程](./examples/async.md) - 异步操作示例
- [错误处理](./examples/error-handling.md) - 错误处理最佳实践

### 5. 最佳实践
- [性能优化](./best-practices/performance.md) - 性能调优建议
- [内存安全](./best-practices/memory-safety.md) - 内存安全指南
- [错误处理](./best-practices/error-handling.md) - 错误处理模式
- [测试策略](./best-practices/testing.md) - 测试方法

### 6. 开发指南
- [构建说明](./development/build.md) - 如何构建项目
- [贡献指南](./development/contributing.md) - 如何贡献代码
- [发布流程](./development/release.md) - 发布新版本

### 7. 故障排除
- [常见问题](./troubleshooting/faq.md) - 常见问题解答
- [错误代码](./troubleshooting/error-codes.md) - 错误代码参考
- [调试技巧](./troubleshooting/debugging.md) - 调试方法

## 快速链接

- [GitHub 仓库](https://github.com/dushibaiyu/OPCDaclientRs)
- [API 文档](https://docs.rs/opc_da_client) (发布后可用)
- [问题追踪](https://github.com/dushibaiyu/OPCDaclientRs/issues)
- [变更日志](./CHANGELOG.md)

## 许可证

本项目采用 Apache 2.0 许可证。详见 [LICENSE](../LICENSE) 文件。

## 支持

如果您遇到问题或有建议：
1. 查看 [常见问题](./troubleshooting/faq.md)
2. 在 [GitHub Issues](https://github.com/dushibaiyu/OPCDaclientRs/issues) 提交问题
3. 查看示例代码获取使用帮助