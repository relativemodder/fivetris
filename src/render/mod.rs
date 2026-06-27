mod board_painter;
mod settings_ui;
mod sidebar;
mod texture_atlas;
pub mod theme;

use egui::{Color32, Vec2};

use crate::app::ui_state::UiState;
use crate::config::{AppConfig, BlockStyle, find_skin_palette, load_skin_palettes};
use crate::core::piece::{Cell, PieceState};
use crate::core::{Board, GameState};

pub use board_painter::{board_rect, draw_board};
pub use settings_ui::draw_settings_window;
pub use sidebar::{draw_next_panel, draw_sidebar};
pub use texture_atlas::{TextureAtlas, TextureError, load_texture_atlas, load_texture_atlas_bytes};
pub use theme::ThemeColors;

pub fn btn(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let saved = ui.style().spacing.button_padding;
    ui.style_mut().spacing.button_padding = egui::vec2(20.0, 5.0);
    let r = ui.button(text);
    ui.style_mut().spacing.button_padding = saved;
    r
}

pub fn fa_btn(ui: &mut egui::Ui, icon: char, text: &str) -> egui::Response {
    let saved = ui.style().spacing.button_padding;
    ui.style_mut().spacing.button_padding = egui::vec2(15.0, 5.0);
    let r = ui.add(egui::Button::new(format!("{icon}  {text}")));
    ui.style_mut().spacing.button_padding = saved;
    r
}

pub const CELL_SIZE: f32 = 28.0;
pub const GAP: f32 = 1.0;

pub struct GameRenderView<'a> {
    pub game: &'a GameState,
    pub ghost: Option<PieceState>,
    pub mode_name: &'static str,
    pub paused: bool,
    pub style: RenderStyle,
    pub texture_atlas: Option<&'a TextureAtlas>,
    pub texture_handle: Option<&'a egui::TextureHandle>,
    pub board_cell_size: f32,
    pub preview_cell_size: f32,
    pub ui_state: &'a UiState,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderStyle {
    pub colored_ghost: bool,
    pub ghost_alpha: u8,
    pub i_cell: Color32,
    pub j_cell: Color32,
    pub s_cell: Color32,
    pub o_cell: Color32,
    pub z_cell: Color32,
    pub l_cell: Color32,
    pub t_cell: Color32,
    pub empty_cell: Color32,
    pub gray_cell: Color32,
    pub ghost_cell: Color32,
    pub background: Color32,
    pub box_color: Color32,
    pub text_color: Color32,
    pub corner_radius: u8,
    pub filled_stroke: Color32,
    pub empty_stroke: Color32,
    pub theme_colors: ThemeColors,
}

impl RenderStyle {
    pub fn from_config(config: &AppConfig) -> Self {
        let palettes = AppConfig::skin_search_dirs()
            .into_iter()
            .filter_map(|dir| load_skin_palettes(&dir).ok())
            .flatten()
            .collect::<Vec<_>>();
        let palette = find_skin_palette(&palettes, &config.selected_skin);

        let default_style = BlockStyle::Inset;
        let corner_radius = match palette
            .map(|palette| palette.style)
            .unwrap_or(default_style)
        {
            BlockStyle::Flat => 1,
            BlockStyle::Inset => 6,
        };

        let texture_corner_radius = match config.selected_texture.as_str() {
            "rounded-square.png" => 6,
            "template.png" => 1,
            _ => corner_radius,
        };

        let color =
            |rgba: [u8; 4]| Color32::from_rgba_premultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);

        let empty_cell = palette
            .map(|palette| color(palette.empty))
            .unwrap_or(Color32::from_gray(16));
        let gray_cell = palette
            .map(|palette| color(palette.gray))
            .unwrap_or(Color32::from_gray(80));
        let background = palette
            .map(|palette| color(palette.background))
            .unwrap_or(Color32::from_gray(12));
        let box_color = palette
            .map(|palette| color(palette.box_color))
            .unwrap_or(Color32::from_gray(24));
        let text_color = palette
            .map(|palette| color(palette.text))
            .unwrap_or(Color32::WHITE);
        let colored_ghost = config.colored_ghost;
        let ghost_alpha = (config.ghost_opacity_percent as u32 * 255 / 100) as u8;
        let ghost_cell = Color32::from_rgba_premultiplied(
            text_color.r(),
            text_color.g(),
            text_color.b(),
            ghost_alpha,
        );

        Self {
            colored_ghost,
            ghost_alpha,
            i_cell: palette
                .map(|palette| color(palette.i))
                .unwrap_or(Color32::from_rgb(0, 240, 240)),
            j_cell: palette
                .map(|palette| color(palette.j))
                .unwrap_or(Color32::from_rgb(0, 80, 255)),
            s_cell: palette
                .map(|palette| color(palette.s))
                .unwrap_or(Color32::from_rgb(0, 220, 0)),
            o_cell: palette
                .map(|palette| color(palette.o))
                .unwrap_or(Color32::from_rgb(255, 240, 0)),
            z_cell: palette
                .map(|palette| color(palette.z))
                .unwrap_or(Color32::from_rgb(255, 40, 0)),
            l_cell: palette
                .map(|palette| color(palette.l))
                .unwrap_or(Color32::from_rgb(255, 160, 0)),
            t_cell: palette
                .map(|palette| color(palette.t))
                .unwrap_or(Color32::from_rgb(180, 0, 255)),
            empty_cell,
            gray_cell,
            ghost_cell,
            background,
            box_color,
            text_color,
            corner_radius: texture_corner_radius,
            filled_stroke: box_color,
            empty_stroke: grid_stroke_color(background, config.grid_brightness),
            theme_colors: ThemeColors::new(
                (background.r() as u32 + background.g() as u32 + background.b() as u32) / 3 < 128,
            ),
        }
    }
}

pub(crate) fn textured_cell_tint(cell: Cell, fallback_tint: Color32) -> Color32 {
    match cell {
        Cell::Ghost => fallback_tint,
        _ => Color32::WHITE,
    }
}

fn grid_stroke_color(background: Color32, brightness: u8) -> Color32 {
    if brightness == 0 {
        return Color32::TRANSPARENT;
    }
    let t = brightness.min(100) as u32;
    Color32::from_rgba_premultiplied(
        ((background.r() as u32 * (100 - t) + 255u32 * t) / 100) as u8,
        ((background.g() as u32 * (100 - t) + 255u32 * t) / 100) as u8,
        ((background.b() as u32 * (100 - t) + 255u32 * t) / 100) as u8,
        background.a(),
    )
}

pub fn cell_color(cell: Cell, style: RenderStyle) -> Color32 {
    match cell {
        Cell::Empty => style.empty_cell,
        Cell::I => style.i_cell,
        Cell::J => style.j_cell,
        Cell::S => style.s_cell,
        Cell::O => style.o_cell,
        Cell::Z => style.z_cell,
        Cell::L => style.l_cell,
        Cell::T => style.t_cell,
        Cell::Gray => style.gray_cell,
        Cell::Ghost => style.ghost_cell,
    }
}

pub fn cell_texture_index(cell: Cell) -> usize {
    match cell {
        Cell::Empty => 0,
        Cell::I => 1,
        Cell::J => 2,
        Cell::S => 3,
        Cell::O => 4,
        Cell::Z => 5,
        Cell::L => 6,
        Cell::T => 7,
        Cell::Gray => 8,
        Cell::Ghost => 9,
    }
}

pub fn board_size(board: &Board, cell_size: f32) -> Vec2 {
    let step = cell_size + GAP;
    Vec2::new(
        board.width as f32 * step,
        board.visible_height as f32 * step,
    )
}
