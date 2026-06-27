use egui::Color32;

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub danger: Color32,
    pub warning: Color32,
    pub success: Color32,
    pub info: Color32,
}

impl ThemeColors {
    pub fn new(dark_bg: bool) -> Self {
        if dark_bg {
            Self {
                danger: Color32::from_rgb(255, 80, 80),
                warning: Color32::from_rgb(255, 200, 50),
                success: Color32::from_rgb(80, 220, 80),
                info: Color32::from_rgb(80, 180, 255),
            }
        } else {
            Self {
                danger: Color32::from_rgb(200, 0, 0),
                warning: Color32::from_rgb(180, 120, 0),
                success: Color32::from_rgb(0, 140, 0),
                info: Color32::from_rgb(0, 100, 180),
            }
        }
    }
}
