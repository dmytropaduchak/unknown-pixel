use super::BIG;
use super::MEDIUM;
use super::SMALL;
use miniquad::conf::Conf;
use miniquad::conf::Icon;

pub fn default() -> Conf {
    let icon = Icon {
        big: BIG,
        small: SMALL,
        medium: MEDIUM,
    };
    let conf = Conf {
        window_title: "unkNOWn Pixel".to_string(),
        icon: Some(icon),
        ..Default::default()
    };
    conf
}
