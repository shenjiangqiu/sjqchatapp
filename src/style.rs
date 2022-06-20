use iced::{container, Background, Color};

pub struct ContainerSend;
pub struct ContainerRecv;

impl container::StyleSheet for ContainerSend {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 30.,
            border_width: 3.,
            border_color: Color::from_rgb8(163, 238, 245),
            ..Default::default()
        }
    }
}

impl container::StyleSheet for ContainerRecv {
    fn style(&self) -> container::Style {
        container::Style {
            border_radius: 30.,
            border_width: 3.,
            border_color: Color::from_rgb8(223, 171, 173),
            ..Default::default()
        }
    }
}
