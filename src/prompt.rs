use inquire::{CustomUserError};
use inquire::list_option::ListOption;
use inquire::ui::{Attributes, Color, RenderConfig, Styled, StyleSheet};
use inquire::validator::Validation;
use ocl::{Device, DeviceType, Platform};


pub fn get_nice_render_config_new() -> RenderConfig {
    let mut render_config = RenderConfig::default();
    render_config.prompt_prefix = Styled::new("❯").with_fg(Color::LightBlue);
    render_config.answered_prompt_prefix =
        Styled::new("✔").with_fg(Color::LightGreen);
    render_config.canceled_prompt_indicator =
        Styled::new("✘").with_fg(Color::LightRed)
            .with_attr(Attributes::BOLD);
    render_config.highlighted_option_prefix =
        Styled::new("▶").with_fg(Color::LightBlue);
    render_config.selected_checkbox =
        Styled::new("✔").with_fg(Color::LightGreen);
    render_config.scroll_up_prefix =
        Styled::new("▲").with_fg(Color::LightBlue);
    render_config.scroll_down_prefix =
        Styled::new("▼")
            .with_fg(Color::LightBlue);
    render_config.unselected_checkbox = Styled::new("✔").with_fg(Color::LightRed);

    render_config.error_message = render_config
        .error_message
        .with_prefix(Styled::new("✘").with_fg(Color::LightRed));

    render_config.answer = StyleSheet::new()
        .with_attr(Attributes::BOLD)
        .with_fg(Color::LightBlue);

    render_config.help_message = StyleSheet::new().with_fg(Color::White);

    render_config
}


pub fn termination_method_validator(options: &[ListOption<&&str>]) -> Result<Validation, CustomUserError> {
    if options.is_empty() {
        return Ok(Validation::Invalid("This list is too small!".into()))
    }
    Ok(Validation::Valid)
}

pub fn duration_validator(option: &u16)  -> Result<Validation, CustomUserError>  {
    if *option == 0 {
        return Ok(Validation::Invalid("Test cannot be 0 minutes".into()));
    }
    Ok(Validation::Valid)
}

pub fn platform_formatter(list_option: ListOption<&Platform>) -> String {
    let name = list_option.value.name().unwrap_or("Unknown".to_string());
    let devices = Device::list(list_option.value, Some(DeviceType::GPU))
        .unwrap_or(vec![]);
    let mut device_string = String::new();
    for device in devices {
        device_string.push_str(&format!("{} | ", device.name().unwrap_or("Unknown".to_string())));
    }
    format!("{name} | {device_string}")
}

pub fn temperature_validator(option: &u8, temp: Option<f32>)  -> Result<Validation, CustomUserError>  {
    let value = *option as f32;
    if value == 0.0 {
        return Ok(Validation::Invalid("Temperature must not be 0".into()));
    }

    let Some(current_temp)  = temp else {
        return Ok(Validation::Valid)
    };

    if current_temp > value {
        return Ok(Validation::Invalid(format!("The current temperature is {current_temp}C, which is higher than the temperature you want to stop at!").into()));
    }

    Ok(Validation::Valid)
}



