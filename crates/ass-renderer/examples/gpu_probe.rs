//! Probe whether a usable wgpu adapter/device is available in this environment
//! (feasibility check for headless GPU-backend verification). Dev-only.

fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    println!("== enumerated adapters ==");
    for a in instance.enumerate_adapters(wgpu::Backends::all()) {
        let i = a.get_info();
        println!(
            "  {:?}  {}  type={:?}  driver={}",
            i.backend, i.name, i.device_type, i.driver
        );
    }

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }));

    match adapter {
        Some(a) => {
            let i = a.get_info();
            println!(
                "SELECTED: {} backend={:?} type={:?}",
                i.name, i.backend, i.device_type
            );
            match pollster::block_on(a.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("probe"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )) {
                Ok(_) => println!("DEVICE_OK: yes -> headless GPU verification is FEASIBLE"),
                Err(e) => println!("DEVICE_OK: no ({e})"),
            }
        }
        None => println!("NO_ADAPTER -> request_adapter returned None"),
    }
}
