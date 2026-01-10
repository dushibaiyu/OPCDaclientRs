//! Basic example demonstrating OPC DA Client usage
//! This example connects to an OPC server and reads/writes values.

use opc_da_client::{OpcClient, OpcValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let client = OpcClient::new()?;
    println!("OPC client initialized");
    
    // Connect to an OPC server
    // Common simulation server names: "Matrikon.OPC.Simulation.1", "OPCSim.KEPServerEX.V6"
    let server = client.connect_to_local_server("Matrikon.OPC.Simulation.1")?;
    println!("Connected to OPC server");
    
    // Get server status
    let (state, vendor_info) = server.get_status()?;
    println!("Server state: {}, Vendor: {}", state, vendor_info);
    
    // Get available item names
    match server.get_item_names() {
        Ok(items) => println!("Available items: {:?}", items),
        Err(e) => println!("Note: Could not get item names: {}", e),
    }
    
    // Create an OPC group
    let group = server.create_group("RustTestGroup", true, 1000, 0.0)?;
    println!("Created OPC group");
    
    // Add an item to the group
    // Example item names from simulation servers: "Bucket Brigade.UInt2", "Random.Int2", etc.
    let item = group.add_item("Bucket Brigade.UInt2")?;
    println!("Added item to group");
    
    // Read current value
    match item.read_sync() {
        Ok((value, quality)) => {
            println!("Current value: {:?}, Quality: {:?}", value, quality);
            
            // Write a new value (if it's an integer type)
            if let OpcValue::Int32(current) = value {
                let new_value = OpcValue::Int32(current + 1);
                item.write_sync(&new_value)?;
                println!("Wrote new value: {:?}", new_value);
                
                // Read back to verify
                let (updated_value, updated_quality) = item.read_sync()?;
                println!("Updated value: {:?}, Quality: {:?}", updated_value, updated_quality);
            }
        }
        Err(e) => println!("Failed to read item: {}", e),
    }
    
    println!("Basic OPC example completed successfully");
    Ok(())
}