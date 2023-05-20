use std::error::Error;
use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::Manager;

// const TEMPERATURE_CHARACTERISTIC_UUID: &str = "8edfffef-3d1b-9c37-4623-ad7265f14076";
const TEMPERATURE_CHARACTERISTIC_UUID: &str = "00001f1f-0000-1000-8000-00805f9b34fb";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // pretty_env_logger::init();

    let manager: Manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        let a = adapter.adapter_info().await?;
        println!("Starting scan on {:?}...", a);
        adapter
            .start_scan(ScanFilter::default()) //
            .await
            .expect("Can't scan BLE adapter for connected devices...");
        time::sleep(Duration::from_secs(10)).await;
        let peripherals = adapter.peripherals().await?;
        if peripherals.is_empty() {
            eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
        } else {
            // All peripheral devices in range
            for peripheral in peripherals.iter() {
                let properties = peripheral.properties().await?;
                let is_connected = peripheral.is_connected().await?;
                let local_name = properties
                    .unwrap()
                    .local_name
                    .unwrap_or(String::from("(peripheral name unknown)"));
                let mac_address = peripheral.address();
                if !local_name.starts_with("ATC") {
                    // println!("Skipping {:?}", local_name);
                    continue;
                }
                println!(
                    "Peripheral {:?} is connected: {:?}  [{:?}]",
                    local_name, is_connected, mac_address
                );
                if !is_connected {
                    println!("Connecting to peripheral {:?}...", &local_name);
                    if let Err(err) = peripheral.connect().await {
                        eprintln!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                }
                let is_connected = peripheral.is_connected().await?;
                println!(
                    "Now connected ({:?}) to peripheral {:?}...",
                    is_connected, &local_name
                );
                peripheral.discover_services().await?;
                let chars = peripheral.characteristics();
                let cmd_char = chars
                    .iter()
                    .find(|c| c.uuid.to_string() == TEMPERATURE_CHARACTERISTIC_UUID)
                    .expect("Unable to find characterics");
                // let ch = peripheral.read(&cmd_char).await?;
                // let temperature = u16::from(ch[0]) << 0
                //                      | u16::from(ch[1]) << 8;
                // let humidity = u8::from(ch[2]);
                // let battery_voltage = u16::from(ch[3]) << 0
                //                          | u16::from(ch[4]) << 8;
                // let battery_voltage_f: f32 = f32::from(battery_voltage) / 1000.0;
                // let temperature_f : f32 = f32::from(temperature) / 100.0;
                // println!("{}: {} C, {} %, {} V", mac_address, temperature_f, humidity, battery_voltage_f);

                let temp_cmd = vec![0xFA, 0x10];
                peripheral
                    .write(&cmd_char, &temp_cmd, WriteType::WithoutResponse)
                    .await?;

                if is_connected {
                    println!("Disconnecting from peripheral {:?}...", &local_name);
                    peripheral
                        .disconnect()
                        .await
                        .expect("Error disconnecting from BLE peripheral");
                }
            }
        }
    }
    Ok(())
}
