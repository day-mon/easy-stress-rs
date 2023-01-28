use std::fmt::{Display, Formatter};
use colored::Colorize;
use ocl::{Device, DeviceType, Platform};
use ocl::core::DeviceInfo;
use sysinfo::{CpuExt, System, SystemExt};

pub fn get_system_gpus() -> Vec<Device> {
    let platform = Platform::default();
    let devices =  Device::list(platform, Some(DeviceType::GPU))
        .unwrap();
    devices
}



pub struct GreetingValues {
    pub host_name: String,
    pub os: String,
    pub memory: u64,
    pub cpu_information: CPUInformation,
    pub gpu_information: Vec<GPUInformation>

}

pub struct CPUInformation {
    pub name: String,
    // if intel hyper-threading or AMD SMT enabled on chip logical cores != physical cores
    pub logical_cores: usize,
    pub physical_cores: usize,
}

#[derive(Default)]
pub struct GPUInformation {
    pub name: String,
    pub mem: Option<usize>
}

impl CPUInformation {
    pub fn new(system: &System) -> Self {
        if let Some(cpu) = system.cpus().first() {
            let name = cpu.brand().to_string();
            let logical_cores = system.cpus().len();
            let physical_cores = system.physical_core_count().unwrap_or_else(|| {
                println!("{}", "No physical core count found, Falling back to logical core count".red());
                logical_cores
            });
            CPUInformation { name, logical_cores, physical_cores }
        } else {
            panic!("No CPU found")
        }
    }
}

impl GPUInformation {
    pub fn new(device: &Device) -> Option<Self> {
        // get the name of the gpu or return None if it fails
        let name = device.info(DeviceInfo::Name).ok()?
            .to_string();

        let mem = device.info(DeviceInfo::GlobalMemSize);
        if let Ok(mem) = mem {
            let mem = mem.to_string().parse::<usize>().unwrap();
            return Some(GPUInformation { name, mem: Some(mem) })

        }

        Some(GPUInformation { name, mem: None })
    }


}

impl Display for CPUInformation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // pretty print the cpu information
        writeln!(f, "{}", "CPU Information".bold())?;
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Logical Cores: {}", self.logical_cores)?;
        write!(f, "Physical Cores: {}", self.physical_cores)?;
        Ok(())
    }
}

impl Display for GPUInformation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // pretty print the gpu information
        writeln!(f, "{}", "GPU Information".bold())?;
        writeln!(f, "Name: {}", self.name)?;
        if let Some(mem) = self.mem {
            writeln!(f, "Memory: {} MB", mem / 1024 / 1024)?;
        }
        Ok(())
    }
}

impl Display for GreetingValues {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // pretty print the greeting values
        writeln!(f, "Hello {}!", self.host_name.bold())?;
        writeln!(f, "OS: {}", self.os)?;
        writeln!(f, "Memory: {} MB", self.memory / 1024 / 1024)?;
        writeln!(f, "{}", self.cpu_information)?;
        for gpu in &self.gpu_information {
            write!(f, "{gpu}")?;
        }
        Ok(())
    }
}

impl GreetingValues {
    pub fn new(system: &System) -> Self {
        let host_name = system.host_name().unwrap_or("User".to_string());
        let os_long = system.long_os_version().unwrap_or_else(|| "N/A".to_string());
        let kernel_version = system.kernel_version();
        let os = if let Some(kernel_version) = kernel_version {
            format!("{os_long} v{kernel_version}")
        } else {
            os_long
        };
        let memory = system.total_memory();
        let cpu_information = CPUInformation::new(system);
        let gpu_information = get_system_gpus().iter().filter_map(GPUInformation::new).collect();
        GreetingValues { host_name, os, memory, cpu_information, gpu_information }
    }
}