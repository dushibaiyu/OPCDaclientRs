//! Example demonstrating OPC DA Client subscription functionality
//! This example connects to an OPC server and subscribes to data changes.

use OPCDaclientRs::{OpcClient, OpcValue, OpcQuality, OpcDataCallback};
use std::sync::Arc;
use std::time::Duration;
use std::thread;

/// A simple callback implementation that prints received data
struct DataChangeCallback;

impl OpcDataCallback for DataChangeCallback {
    fn on_data_change(&self, group_name: &str, item_name: &str, value: OpcValue, quality: OpcQuality, timestamp: u64) {
        println!(
            "Data Change - Group: '{}', Item: '{}', Value: {:?}, Quality: {:?}, Timestamp: {} ms",
            group_name, item_name, value, quality, timestamp
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = OpcClient::new()?;
    println!("OPC client initialized");
    
    // Connect to an OPC server
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    println!("Connected to OPC server");
    
    // Get server status
    let (state, vendor_info) = server.get_status()?;
    println!("Server state: {}, Vendor: {}", state, vendor_info);
    
    // Get available item names
    let item_names = match server.get_item_names() {
        Ok(names) => {
            println!("Available items: {:?}", names);
            names
        }
        Err(e) => {
            println!("Note: Could not get item names: {}", e);
            Vec::new()
        }
    };
    
    // Create an OPC group for subscriptions
    let group = server.create_group("RustSubscriptionGroup", true, 1000, 0.0)?;
    println!("Created OPC group");
    
    // Add an item to the group
    let item_name = if item_names.contains(&"Bucket Brigade.Int2".to_string()) {
        "Bucket Brigade.UInt2"
    } else if !item_names.is_empty() {
        &item_names[0]
    } else {
        "Random.Int2" // Fallback for simulation servers
    };
    
    let item = group.add_item(item_name)?;
    println!("Added item '{}' to group", item_name);
    
    // Create and set up the callback for data changes
    let callback = Arc::new(DataChangeCallback {});
    group.enable_async_subscription(callback)?;
    println!("Enabled async subscription");
    
    // Write initial values to trigger changes
    println!("Writing initial values to trigger data changes...");
    
    for i in 0..5 {
        let write_value = OpcValue::Int32(1000 + i);
        
        match item.write_sync(&write_value) {
            Ok(_) => println!("Wrote value: {:?}", write_value),
            Err(e) => println!("Failed to write value: {}", e),
        }
        
        // Read back to verify
        match item.read_sync() {
            Ok((value, quality, timestamp)) => {
                println!("Read value: {:?}, Quality: {:?}, Timestamp: {} ms", value, quality, timestamp);
            }
            Err(e) => println!("Failed to read: {}", e),
        }
        
        thread::sleep(Duration::from_secs(1));
    }
    
    // Keep the program running to receive data changes
    println!("Waiting for data changes... Press Ctrl+C to exit");
    
    // In a real application, you'd use a proper event loop or async runtime
    for i in 0..30 {
        thread::sleep(Duration::from_secs(1));
        if i % 5 == 0 {
            println!("Waiting... {}s", i);
        }
    }
    
    println!("Subscription example completed");
    Ok(())
}