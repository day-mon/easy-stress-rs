use sysinfo::{ComponentExt, System, SystemExt};

pub fn cpu_temp(
    system: &mut System,
    refresh: bool
) -> Option<f32> {
    if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
       return None
    }

    if refresh {
        system.refresh_components_list();
    }

   system.components()
       .iter()
       .find(|c| c.label().contains("CPU"))
       .map(|component| component.temperature())
}