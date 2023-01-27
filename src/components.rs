use std::fmt::{Display, Formatter};
use ocl::{Device, DeviceType, Platform};
use ocl::core::DeviceInfo;
use sysinfo::{CpuExt, System, SystemExt};

pub fn get_system_gpus() -> Vec<Device> {
    let platform = Platform::default();
    let devices =  Device::list(platform, Some(DeviceType::GPU))
        .unwrap();
    devices
}



// pub struct GreetingValues {
//     pub host_name: String,
//     pub os: String,
//     pub memory: u64,
//     pub cpu_information: CPUInformation,
//     pub gpu_information: Vec<GPUInformation>
//
// }
//
// pub struct CPUInformation {
//     pub name: String,
//     // if intel hyper-threading or AMD SMT enabled on chip logical cores != physical cores
//     pub logical_cores: Option<usize>,
//     pub physical_cores: usize,
// }
//
// #[derive(Default)]
// pub struct GPUInformation {
//     pub name: String,
//     pub gpu_type: String,
//     pub mem: Option<usize>
// }
//
// impl CPUInformation {
//     pub fn new(system: &System) -> Result<Self, String> {
//
//
//         Ok(()
//     }
// }
//
// impl GPUInformation {
//     pub fn new(device: &Device) -> Self {
//         let name = device.name().unwrap();
//         GPUInformation {
//             name,
//
//         }
//     }
//
//
// }
//
// impl Display for CPUInformation {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }
//
// impl GreetingValues {
//     pub fn new(system: &System) -> Self {
//         let platform = Platform::default();
//         let devices =  Device::list(platform, Some(DeviceType::GPU))
//             .unwrap();
//
//         let cpu_information = CPUInformation::new(system);
//         let d = if let Err(cpu_information) = CPUInformation::new(system) {
//
//         };
//         let host_name = system.host_name().unwrap_or_else(|| "User".to_string());
//         let os_long = system.long_os_version().unwrap_or_else(|| "Not found".to_string());
//         let kernel_version = system.kernel_version().map(|d| {
//            d.to_string()
//         });
//
//
//
//         GreetingValues {
//
//         }
//
//     }
// }