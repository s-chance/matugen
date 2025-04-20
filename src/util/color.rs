use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};
use material_colors::color::Argb;

use material_colors::palette::TonalPalette;
use material_colors::theme::Palettes;

use colorsys::Rgb;

use crate::Schemes;

#[cfg(feature = "dump-json")]
use super::arguments::Format;

use matugen::color::format::rgb_from_argb;

const DEFAULT_TONES: [i32; 18] = [
    0, 5, 10, 15, 20, 25, 30, 35, 40, 50, 60, 70, 80, 90, 95, 98, 99, 100,
];

pub fn show_color(schemes: &Schemes, source_color: &Argb) {
    let mut table = Table::new();
    table
        .load_preset("││──├─┼┤│ │││┬┴┌┐└┘")
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("NAME").set_alignment(CellAlignment::Center),
            Cell::new("LIGHT").set_alignment(CellAlignment::Center),
            Cell::new("DARK").set_alignment(CellAlignment::Center),
        ]);

    for ((field, color_light), (_, color_dark)) in std::iter::zip(&schemes.light, &schemes.dark) {
        let color_light: Rgb = rgb_from_argb(*color_light);
        let color_dark: Rgb = rgb_from_argb(*color_dark);

        generate_table_rows(&mut table, field, color_light, color_dark);
    }

    generate_table_rows(
        &mut table,
        "source_color",
        rgb_from_argb(*source_color),
        rgb_from_argb(*source_color),
    );

    println!("{table}")
}

#[cfg(feature = "dump-json")]
pub fn dump_json(schemes: &Schemes, source_color: &Argb, format: &Format, palettes: &Palettes) {
    use std::collections::HashMap;

    let mut colors_normal_light: HashMap<&str, String> = HashMap::new();
    let mut colors_normal_dark: HashMap<&str, String> = HashMap::new();

    for ((field, color_light), (_, color_dark)) in std::iter::zip(&schemes.light, &schemes.dark) {
        let color_light: Rgb = rgb_from_argb(*color_light);
        let color_dark: Rgb = rgb_from_argb(*color_dark);

        colors_normal_light.insert(field, format_single_color(color_light, format));
        colors_normal_dark.insert(field, format_single_color(color_dark, format));
    }

    colors_normal_light.insert(
        "source_color",
        format_single_color(rgb_from_argb(*source_color), format),
    );

    println!(
        "{}",
        serde_json::json!({
            "colors": {
                "light": colors_normal_light,
                "dark": colors_normal_dark,
            },
            "palettes": format_palettes(palettes, format),
        })
    );
}

#[cfg(feature = "dump-json")]
fn format_palettes(palettes: &Palettes, format: &Format) -> serde_json::Value {
    let primary = format_single_palette(palettes.primary, format);
    let secondary = format_single_palette(palettes.secondary, format);
    let tertiary = format_single_palette(palettes.tertiary, format);
    let neutral = format_single_palette(palettes.neutral, format);
    let neutral_variant = format_single_palette(palettes.neutral_variant, format);
    let error = format_single_palette(palettes.error, format);
    serde_json::json!({
        "primary": primary,
        "secondary": secondary,
        "tertiary": tertiary,
        "neutral": neutral,
        "neutral_variant": neutral_variant,
        "error": error,
    })
}

#[cfg(feature = "dump-json")]
fn format_single_palette(palette: TonalPalette, format: &Format) -> serde_json::Value {
    let mut tones: String = "".to_string();

    for (i, tone) in DEFAULT_TONES.into_iter().enumerate() {
        if i == 0 {
            tones.push_str("{\n");
        }

        tones.push_str(&format!(
            "\"{}\": \"{}\"",
            &format!("{}", tone),
            format_single_color(rgb_from_argb(palette.tone(tone)), format),
        ));

        if i != DEFAULT_TONES.len() - 1 {
            tones.push_str(",\n");
        } else {
            tones.push_str("\n}");
        }
    }

    serde_json::from_str(&tones).unwrap()
}

#[cfg(feature = "dump-json")]
fn format_single_color(color: Rgb, format: &Format) -> String {
    use matugen::color::format::{
        format_hex, format_hex_stripped, format_hsl, format_hsla, format_rgb, format_rgba,
        hsl_from_rgb,
    };

    let fmt = match format {
        Format::Rgb => |c: Rgb| format_rgb(&c),
        Format::Rgba => |c: Rgb| format_rgba(&c, true),
        Format::Hsl => |c: Rgb| format_hsl(&hsl_from_rgb(c)),
        Format::Hsla => |c: Rgb| format_hsla(&hsl_from_rgb(c), true),
        Format::Hex => |c: Rgb| format_hex(&c),
        Format::Strip => |c: Rgb| format_hex_stripped(&c),
    };
    fmt(color)
}

fn generate_table_rows(table: &mut Table, field: &str, color_light: Rgb, color_dark: Rgb) {
    table.add_row(vec![
        Cell::new(field),
        Cell::new(color_light.to_hex_string().to_uppercase())
            .bg(Color::Rgb {
                r: color_light.red() as u8,
                g: color_light.green() as u8,
                b: color_light.blue() as u8,
            })
            .fg(generate_wcag_fg(&color_light)),
        Cell::new(color_dark.to_hex_string().to_uppercase())
            .bg(Color::Rgb {
                r: color_dark.red() as u8,
                g: color_dark.green() as u8,
                b: color_dark.blue() as u8,
            })
            .fg(generate_wcag_fg(&color_dark)),
    ]);
}

fn generate_wcag_fg(color: &Rgb) -> Color {
    let r = color.red() / 255.0;
    let g = color.green() / 255.0;
    let b = color.blue() / 255.0;

    let r = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let g = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let b = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };

    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;

    if luma > 0.179 {
        Color::Black
    } else {
        Color::White
    }
}
