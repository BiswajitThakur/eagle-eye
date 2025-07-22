use std::io;

use eagle_eye_device::{Device, DeviceManager};

fn main() -> io::Result<()> {
    let mut my_devices = DeviceManager::<512>::new();
    my_devices.push_device(Device::new().id(123).key([33; 32]));
    my_devices.scan()?;
    println!(
        "No of online Devices: {}/{}",
        my_devices.total_online(),
        my_devices.total_device()
    );
    std::thread::sleep(std::time::Duration::from_secs(30));
    Ok(())
}
