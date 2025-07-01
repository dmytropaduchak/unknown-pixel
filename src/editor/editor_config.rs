use miniquad::conf::Conf;
use miniquad::conf::Icon;
use std::error::Error;
use std::fs;

fn icon<const SIZE: usize>(path: &str) -> Result<[u8; SIZE], Box<dyn Error>> {
    let data = fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let len = data.len();
    data.try_into().map_err(|_| {
        format!(
            "{} has invalid size (expected {} bytes, got {})",
            path, SIZE, len
        )
        .into()
    })
}

fn files() -> Result<Icon, Box<dyn Error>> {
    let small = icon::<{ 16 * 16 * 4 }>("icon_16.rgba")?;
    let medium = icon::<{ 32 * 32 * 4 }>("icon_32.rgba")?;
    let big = icon::<{ 64 * 64 * 4 }>("icon_64.rgba")?;
    Ok(Icon { small, medium, big })
}

pub fn editor_config() -> Conf {
    let icon = match files() {
        Ok(icon) => Some(icon),
        Err(e) => {
            eprintln!("Failed to load icons: {e}");
            None
        }
    };
    let window_title = "unkNOWn Shape".to_string();

    Conf {
        window_title,
        icon,
        // window_width: 1920,
        // window_height: 1080,
        ..Default::default()
    }
}
