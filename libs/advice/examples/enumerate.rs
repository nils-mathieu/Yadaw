fn main() {
    match print_host_info() {
        Ok(_) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}

/// Print host data.
fn print_host_info() -> Result<(), advice::Error> {
    let Some(host) = advice::default_host()? else {
        println!("No host found.");
        return Ok(());
    };

    let devices = host.devices()?;

    for device in devices {
        print_device_info(device.as_ref())?;
        println!();
    }

    Ok(())
}

/// Prints information about a device.
fn print_device_info(device: &dyn advice::Device) -> Result<(), advice::Error> {
    if let Some(name) = device.name()? {
        println!(" - {name}");
    } else {
        println!(" - <Unknown>");
    }

    let shared_output_format = device.output_formats(advice::ShareMode::Share)?;
    let exclusive_output_format = device.output_formats(advice::ShareMode::Exclusive)?;
    let shared_input_format = device.input_formats(advice::ShareMode::Share)?;
    let exclusive_input_format = device.input_formats(advice::ShareMode::Exclusive)?;

    if shared_output_format.is_some() || exclusive_output_format.is_some() {
        if shared_output_format == exclusive_input_format {
            if let Some(shared) = shared_output_format.as_ref() {
                println!("    - Output formats:");
                print_device_formats(shared);
            }
        } else {
            if let Some(shared) = shared_output_format.as_ref() {
                println!("    - Output formats (shared):");
                print_device_formats(shared);
            }

            if let Some(exclusive) = exclusive_output_format.as_ref() {
                println!("    - Output formats (exclusive):");
                print_device_formats(exclusive);
            }
        }
    }

    if shared_input_format.is_some() || exclusive_input_format.is_some() {
        if shared_input_format == exclusive_input_format {
            if let Some(shared) = shared_input_format.as_ref() {
                println!("    - Input formats:");
                print_device_formats(shared);
            }
        } else {
            if let Some(shared) = shared_input_format.as_ref() {
                println!("    - Input formats (shared):");
                print_device_formats(shared);
            }

            if let Some(exclusive) = exclusive_input_format.as_ref() {
                println!("    - Input formats (exclusive):");
                print_device_formats(exclusive);
            }
        }
    }

    Ok(())
}

/// Print the formats available for a device.
#[rustfmt::skip]
fn print_device_formats(formats: &advice::DeviceFormats) {
    println!("       - Max channel count: {}", formats.max_channel_count);
    println!("       - Sample formats: {:?}", formats.formats);
    println!("       - Frame rates: {:?}", formats.frame_rates);
    println!("       - Buffer size range: {} - {}", formats.min_buffer_size, formats.max_buffer_size);
    println!("       - Channel layout: {:?}", formats.channel_layouts);
}
