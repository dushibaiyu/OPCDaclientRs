//! 高级示例：演示 OPC DA 客户端的复杂使用场景
//! 
//! 这个示例展示了如何：
//! 1. 连接多个 OPC 服务器
//! 2. 创建多个 OPC 组
//! 3. 实现异步数据订阅
//! 4. 批量读写操作
//! 5. 错误处理和资源清理
//! 
//! ## 运行要求
//! 
//! 1. Windows 操作系统
//! 2. 安装 OPC 仿真服务器（如 MatrikonOPC Simulation Server）
//! 3. 服务器名称：通常为 "Matrikon.OPC.Simulation.1"
//! 
//! ## 运行命令
//! 
//! ```bash
//! cargo run --example advanced_example
//! ```

use opc_da_client::{OpcClient, OpcValue, OpcQuality, OpcDataCallback};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 自定义数据回调处理器
/// 实现 OpcDataCallback trait 来处理异步数据变化
#[derive(Debug)]
struct DataCallbackHandler {
    name: String,
    callback_count: Arc<Mutex<u32>>,
}

impl DataCallbackHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            callback_count: Arc::new(Mutex::new(0)),
        }
    }
    
    fn get_count(&self) -> u32 {
        *self.callback_count.lock().unwrap()
    }
}

impl OpcDataCallback for DataCallbackHandler {
    fn on_data_change(&self, item_name: &str, value: OpcValue, quality: OpcQuality, timestamp: u64) {
        let mut count = self.callback_count.lock().unwrap();
        *count += 1;
        
        println!(
            "[{}] 数据变化 #{:03}: 项名={}, 值={:?}, 质量={:?}, 时间戳={}",
            self.name, *count, item_name, value, quality, timestamp
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OPC DA 高级示例开始 ===");
    println!("演示：多服务器、多组、异步订阅、批量操作");
    
    let start_time = Instant::now();
    
    // 1. 创建 OPC 客户端
    let client = OpcClient::new()?;
    println!("✓ OPC 客户端初始化成功");
    
    // 2. 尝试连接多个服务器（演示错误处理）
    let server_names = vec![
        "Matrikon.OPC.Simulation.1",  // 仿真服务器
        "NonExistent.Server.1",       // 不存在的服务器（演示错误处理）
        "OPCSim.KEPServerEX.V6",      // 另一个仿真服务器
    ];
    
    let mut connected_servers = Vec::new();
    
    for server_name in server_names {
        println!("\n尝试连接服务器: {}", server_name);
        match client.connect_to_local_server(server_name) {
            Ok(server) => {
                println!("  ✓ 连接成功");
                
                // 获取服务器状态
                match server.get_status() {
                    Ok((state, vendor)) => {
                        println!("    状态: {}, 厂商: {}", state, vendor);
                    }
                    Err(e) => {
                        println!("    警告: 无法获取服务器状态: {}", e);
                    }
                }
                
                connected_servers.push((server_name.to_string(), server));
            }
            Err(e) => {
                println!("  ✗ 连接失败: {}", e);
                println!("    注意: 这是预期的错误处理演示");
            }
        }
    }
    
    if connected_servers.is_empty() {
        println!("\n⚠ 警告: 没有成功连接到任何服务器");
        println!("请确保至少有一个 OPC 仿真服务器正在运行");
        return Ok(());
    }
    
    println!("\n✓ 成功连接到 {} 个服务器", connected_servers.len());
    
    // 3. 为每个连接的服务器创建多个组
    let mut all_groups = Vec::new();
    let mut all_items = Vec::new();
    
    for (server_name, server) in &connected_servers {
        println!("\n在服务器 '{}' 上创建组:", server_name);
        
        // 创建快速更新组（500ms）
        match server.create_group("FastGroup", true, 500, 0.0) {
            Ok(fast_group) => {
                println!("  ✓ 创建快速更新组 (500ms)");
                
                // 添加多个项到快速组
                let fast_items = vec![
                    "Bucket Brigade.UInt2",
                    "Random.Int2",
                    "Triangle Waves.Real4",
                ];
                
                for item_name in fast_items {
                    match fast_group.add_item(item_name) {
                        Ok(item) => {
                            println!("    ✓ 添加项: {}", item_name);
                            all_items.push((format!("{}::{}", server_name, item_name), item));
                        }
                        Err(e) => {
                            println!("    ✗ 无法添加项 {}: {}", item_name, e);
                        }
                    }
                }
                
                all_groups.push((format!("{}::FastGroup", server_name), fast_group));
            }
            Err(e) => {
                println!("  ✗ 无法创建快速组: {}", e);
            }
        }
        
        // 创建慢速更新组（2000ms）
        match server.create_group("SlowGroup", true, 2000, 0.0) {
            Ok(slow_group) => {
                println!("  ✓ 创建慢速更新组 (2000ms)");
                
                // 添加项到慢速组
                match slow_group.add_item("Saw-toothed Waves.Int2") {
                    Ok(item) => {
                        println!("    ✓ 添加项: Saw-toothed Waves.Int2");
                        all_items.push((format!("{}::Saw-toothed Waves.Int2", server_name), item));
                    }
                    Err(e) => {
                        println!("    ✗ 无法添加项: {}", e);
                    }
                }
                
                all_groups.push((format!("{}::SlowGroup", server_name), slow_group));
            }
            Err(e) => {
                println!("  ✗ 无法创建慢速组: {}", e);
            }
        }
    }
    
    println!("\n✓ 创建了 {} 个组，添加了 {} 个项", all_groups.len(), all_items.len());
    
    // 4. 批量同步读取演示
    println!("\n=== 批量同步读取演示 ===");
    
    for (item_name, item) in &all_items {
        match item.read_sync() {
            Ok((value, quality)) => {
                println!("  {}: 值={:?}, 质量={:?}", item_name, value, quality);
            }
            Err(e) => {
                println!("  {}: 读取失败: {}", item_name, e);
            }
        }
    }
    
    // 5. 异步订阅演示
    println!("\n=== 异步订阅演示 ===");
    println!("启用异步订阅，等待数据变化...");
    
    // 创建回调处理器
    let callback_handler = DataCallbackHandler::new("MainCallback");
    
    // 为第一个组启用异步订阅
    if let Some((group_name, group)) = all_groups.first() {
        match group.enable_async_subscription(Box::new(callback_handler)) {
            Ok(_) => {
                println!("✓ 在组 '{}' 上启用异步订阅", group_name);
                
                // 手动刷新以触发数据更新
                println!("执行手动刷新...");
                match group.refresh() {
                    Ok(_) => println!("  ✓ 刷新成功"),
                    Err(e) => println!("  ✗ 刷新失败: {}", e),
                }
                
                // 等待一段时间接收异步回调
                println!("等待 3 秒接收异步数据...");
                std::thread::sleep(Duration::from_secs(3));
                
                // 再次刷新
                println!("再次刷新...");
                match group.refresh() {
                    Ok(_) => println!("  ✓ 刷新成功"),
                    Err(e) => println!("  ✗ 刷新失败: {}", e),
                }
                
                // 再等待一段时间
                println!("再等待 2 秒...");
                std::thread::sleep(Duration::from_secs(2));
            }
            Err(e) => {
                println!("✗ 无法启用异步订阅: {}", e);
            }
        }
    }
    
    // 6. 批量写入演示
    println!("\n=== 批量写入演示 ===");
    
    // 只对部分项进行写入操作
    let items_to_write = all_items.iter().take(2).collect::<Vec<_>>();
    
    for (i, (item_name, item)) in items_to_write.iter().enumerate() {
        let write_value = match i % 3 {
            0 => OpcValue::Int32(100 + i as i32),
            1 => OpcValue::Float(50.5 + i as f32),
            2 => OpcValue::Double(75.25 + i as f64),
            _ => OpcValue::Int32(0),
        };
        
        println!("写入 {}: {:?}", item_name, write_value);
        match item.write_sync(&write_value) {
            Ok(_) => println!("  ✓ 写入成功"),
            Err(e) => println!("  ✗ 写入失败: {}", e),
        }
        
        // 短暂延迟
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // 7. 资源清理演示
    println!("\n=== 资源清理演示 ===");
    
    // 显式清理组（虽然 Drop trait 会自动清理）
    println!("显式清理 {} 个组...", all_groups.len());
    for (group_name, _) in all_groups {
        println!("  清理组: {}", group_name);
        // 组会在离开作用域时自动清理
    }
    
    println!("显式清理 {} 个项...", all_items.len());
    for (item_name, _) in all_items {
        println!("  清理项: {}", item_name);
        // 项会在离开作用域时自动清理
    }
    
    // 服务器会在离开作用域时自动断开连接
    println!("服务器连接将在离开作用域时自动断开");
    
    let elapsed = start_time.elapsed();
    println!("\n=== OPC DA 高级示例完成 ===");
    println!("总运行时间: {:.2} 秒", elapsed.as_secs_f32());
    println!("演示了:");
    println!("  - 多服务器连接和错误处理");
    println!("  - 多组创建和管理");
    println!("  - 异步数据订阅");
    println!("  - 批量读写操作");
    println!("  - RAII 资源自动清理");
    
    Ok(())
}