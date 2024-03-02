use iced::{
    color, theme,
    widget::{svg, Svg},
};
pub const IconSize: f32 = 24.;

pub enum Icon {
    Trash,
    Play,
    Plus,
    Lock,
    Unlock,
    Check,
    ArrowLeft,
}
impl Icon {
    pub fn svg(&self) -> Svg {
        let handle = svg::Handle::from_memory(self.data());

        svg(handle).style(theme::Svg::custom_fn(|_theme| svg::Appearance {
            color: Some(color!(0xffffff)),
        }))
    }
    const fn data(&self) -> &'static [u8] {
        match self {
            Icon::Trash => include_bytes!("../assets/feather/trash.svg"),
            Icon::Play => include_bytes!("../assets/feather/play.svg"),
            Icon::Plus => include_bytes!("../assets/feather/plus.svg"),
            Icon::Lock => include_bytes!("../assets/feather/lock.svg"),
            Icon::Unlock => include_bytes!("../assets/feather/unlock.svg"),
            Icon::Check => include_bytes!("../assets/feather/check.svg"),
            Icon::ArrowLeft => include_bytes!("../assets/feather/arrow-left.svg"),
        }
    }
}
