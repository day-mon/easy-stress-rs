use sysinfo::{ComponentExt, CpuExt, System, SystemExt};

pub fn cpu_temp(
    system: &mut System,
    refresh: bool
) -> Option<f32> {
    if refresh {
        system.refresh_components_list();
    }

   system.components()
       .iter()
       .find(|c| c.label().contains("CPU"))
       .map(|component| component.temperature())
}