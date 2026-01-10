//! 错误处理模块
//! 
//! 这个模块定义了 OPC DA 客户端库的错误类型和结果类型。
//! 提供了统一的错误处理机制和详细的错误信息。
//! 
//! ## 主要类型
//! 
//! - `OpcResult<T>`: OPC 操作的结果类型，`Result<T, OpcError>` 的别名
//! - `OpcError`: OPC 操作错误枚举，包含所有可能的错误类型
//! 
//! ## 错误处理原则
//! 
//! 1. **早期失败**: 在可能失败的地方尽早返回错误
//! 2. **详细错误信息**: 提供足够的信息帮助调试
//! 3. **错误转换**: 将底层错误转换为用户友好的错误
//! 4. **错误链**: 保留原始错误信息

use crate::types::OpcValueError;

/// OPC 操作结果类型
/// 
/// 这是 `Result<T, OpcError>` 的类型别名，用于所有 OPC 操作。
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::{OpcResult, OpcClient};
/// 
/// fn connect_to_server() -> OpcResult<()> {
///     let client = OpcClient::new()?;  // 使用 ? 操作符传播错误
///     let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
///     Ok(())
/// }
/// ```
pub type OpcResult<T> = Result<T, OpcError>;

/// OPC 操作错误类型
/// 
/// 这个枚举包含了 OPC DA 客户端库可能遇到的所有错误。
/// 每个错误变体都提供了详细的错误信息。
/// 
/// ## 错误分类
/// 
/// 错误分为以下几类：
/// 1. **操作错误**: 常规 OPC 操作失败
/// 2. **连接错误**: 网络和服务器连接问题
/// 3. **参数错误**: 无效的函数参数
/// 4. **转换错误**: 数据类型转换失败
/// 5. **初始化错误**: COM 和库初始化问题
/// 6. **资源错误**: 服务器、组、项找不到
/// 7. **订阅错误**: 异步订阅失败
/// 8. **超时错误**: 操作超时
/// 
/// ## 示例
/// 
/// ```
/// use opc_da_client::OpcError;
/// 
/// // 创建错误
/// let err = OpcError::connection_failed("无法连接到服务器");
/// 
/// // 匹配错误类型
/// match err {
///     OpcError::ConnectionFailed(msg) => println!("连接失败: {}", msg),
///     _ => println!("其他错误"),
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum OpcError {
    /// 常规 OPC 操作错误
    /// 
    /// 表示一般的 OPC 操作失败，如读取、写入失败。
    /// 
    /// # 可能的原因
    /// - 服务器内部错误
    /// - 权限不足
    /// - 资源限制
    #[error("OPC operation failed: {0}")]
    OperationFailed(String),
    
    /// 连接相关错误
    /// 
    /// 表示连接到服务器或主机失败。
    /// 
    /// # 可能的原因
    /// - 服务器未运行
    /// - 网络问题
    /// - DCOM 配置错误
    /// - 防火墙阻止
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    /// 无效参数错误
    /// 
    /// 表示传递给函数的参数无效。
    /// 
    /// # 可能的原因
    /// - 空字符串参数
    /// - 超出范围的数值
    /// - 无效的服务器名称
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    /// 值转换错误
    /// 
    /// 表示 OPC 值类型转换失败。
    /// 
    /// # 可能的原因
    /// - 类型不匹配
    /// - 无效的原始值
    /// - 不支持的值类型
    #[error("Value conversion error: {0}")]
    ValueConversionError(#[from] OpcValueError),
    
    /// COM 初始化错误
    /// 
    /// 表示 Windows COM 系统初始化失败。
    /// 
    /// # 可能的原因
    /// - 非 Windows 平台
    /// - COM 库未安装
    /// - 权限不足
    #[error("COM initialization failed: {0}")]
    ComInitializationFailed(String),
    
    /// 服务器未找到错误
    /// 
    /// 表示指定的 OPC 服务器不存在或不可访问。
    #[error("Server not found: {0}")]
    ServerNotFound(String),
    
    /// 项未找到错误
    /// 
    /// 表示指定的 OPC 项不存在或不可访问。
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    
    /// 组创建错误
    /// 
    /// 表示创建 OPC 组失败。
    /// 
    /// # 可能的原因
    /// - 组名已存在
    /// - 服务器资源不足
    /// - 无效的组参数
    #[error("Failed to create group: {0}")]
    GroupCreationFailed(String),
    
    /// 异步订阅错误
    /// 
    /// 表示启用异步数据变化订阅失败。
    /// 
    /// # 可能的原因
    /// - 组未激活
    /// - 回调设置失败
    /// - 服务器不支持异步
    #[error("Failed to enable async subscription: {0}")]
    AsyncSubscriptionFailed(String),
    
    /// 超时错误
    /// 
    /// 表示操作在指定时间内未完成。
    #[error("Operation timed out: {0}")]
    Timeout(String),
}

impl OpcError {
    /// 创建新的操作失败错误
    /// 
    /// # 参数
    /// - `msg`: 错误消息
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::OpcError;
    /// 
    /// let err = OpcError::operation_failed("读取操作失败");
    /// ```
    pub fn operation_failed(msg: impl Into<String>) -> Self {
        OpcError::OperationFailed(msg.into())
    }
    
    /// 创建新的连接失败错误
    /// 
    /// # 参数
    /// - `msg`: 错误消息
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::OpcError;
    /// 
    /// let err = OpcError::connection_failed("无法连接到服务器");
    /// ```
    pub fn connection_failed(msg: impl Into<String>) -> Self {
        OpcError::ConnectionFailed(msg.into())
    }
    
    /// 创建新的无效参数错误
    /// 
    /// # 参数
    /// - `msg`: 错误消息
    /// 
    /// # 示例
    /// ```
    /// use opc_da_client::OpcError;
    /// 
    /// let err = OpcError::invalid_parameters("组名不能为空");
    /// ```
    pub fn invalid_parameters(msg: impl Into<String>) -> Self {
        OpcError::InvalidParameters(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_opc_error_creation() {
        // Test creation of all error types
        let op_failed = OpcError::operation_failed("test operation");
        let conn_failed = OpcError::connection_failed("test connection");
        let invalid_params = OpcError::invalid_parameters("test params");
        let value_error = OpcError::ValueConversionError(OpcValueError::type_mismatch("Int32", "String"));
        let com_error = OpcError::ComInitializationFailed("test com".to_string());
        let server_not_found = OpcError::ServerNotFound("test server".to_string());
        let item_not_found = OpcError::ItemNotFound("test item".to_string());
        let group_error = OpcError::GroupCreationFailed("test group".to_string());
        let async_error = OpcError::AsyncSubscriptionFailed("test async".to_string());
        let timeout_error = OpcError::Timeout("test timeout".to_string());
        
        // Test display formatting
        assert!(op_failed.to_string().contains("OPC operation failed"));
        assert!(conn_failed.to_string().contains("Connection failed"));
        assert!(invalid_params.to_string().contains("Invalid parameters"));
        assert!(value_error.to_string().contains("Value conversion error"));
        assert!(com_error.to_string().contains("COM initialization failed"));
        assert!(server_not_found.to_string().contains("Server not found"));
        assert!(item_not_found.to_string().contains("Item not found"));
        assert!(group_error.to_string().contains("Failed to create group"));
        assert!(async_error.to_string().contains("Failed to enable async subscription"));
        assert!(timeout_error.to_string().contains("Operation timed out"));
    }
    
    #[test]
    fn test_opc_error_convenience_methods() {
        // Test convenience methods
        let err1 = OpcError::operation_failed("test");
        let err2 = OpcError::connection_failed("test");
        let err3 = OpcError::invalid_parameters("test");
        
        match err1 {
            OpcError::OperationFailed(msg) => assert_eq!(msg, "test"),
            _ => panic!("Wrong error type"),
        }
        
        match err2 {
            OpcError::ConnectionFailed(msg) => assert_eq!(msg, "test"),
            _ => panic!("Wrong error type"),
        }
        
        match err3 {
            OpcError::InvalidParameters(msg) => assert_eq!(msg, "test"),
            _ => panic!("Wrong error type"),
        }
    }
    
    #[test]
    fn test_opc_result_type() {
        // Test OpcResult type alias
        let success: OpcResult<i32> = Ok(42);
        let error: OpcResult<i32> = Err(OpcError::operation_failed("test"));
        
        assert_eq!(success.unwrap(), 42);
        assert!(error.is_err());
        
        // Test ? operator with OpcResult
        fn test_function() -> OpcResult<i32> {
            let result: OpcResult<i32> = Ok(100);
            let value = result?;
            Ok(value * 2)
        }
        
        assert_eq!(test_function().unwrap(), 200);
    }
    
    #[test]
    fn test_error_conversion() {
        // Test conversion from OpcValueError
        let value_error = OpcValueError::type_mismatch("Int32", "String");
        let opc_error: OpcError = value_error.into();
        
        match opc_error {
            OpcError::ValueConversionError(_) => assert!(true),
            _ => panic!("Expected ValueConversionError"),
        }
    }
}