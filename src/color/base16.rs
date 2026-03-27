use color_eyre::{eyre::WrapErr, Report};
use colorsys::{Hsl, Rgb};
use image::{ImageReader, RgbImage};
use indexmap::IndexMap;
use material_colors::{
    color::Argb,
    hct::Hct,
    utils::math::{difference_degrees, rotate_direction, sanitize_degrees_double},
    dynamic_color::{DynamicColor, DynamicScheme, ContrastCurve, ToneDeltaPair, TonePolarity},
};

use crate::{
    color::{
        backend::wal::WalBackend,
        color::{get_source_color_from_color, ColorFormat, Source},
        format::{argb_from_rgb, rgb_from_argb},
        math::{luminance, saturation},
    },
    scheme::Schemes,
};

const GRAY_NAMES: [&str; 8] = [
    "base00", "base01", "base02", "base03", "base04", "base05", "base06", "base07",
];

const ACCENT_NAMES: [&str; 8] = [
    "base08", "base09", "base0a", "base0b", "base0c", "base0d", "base0e", "base0f",
];

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Backend {
    Wal,
}

impl Backend {
    pub fn create(&self) -> Box<dyn PaletteBackend> {
        match self {
            Backend::Wal => Box::new(WalBackend::default()),
        }
    }
}

pub trait PaletteBackend {
    fn extract(&self, image: &RgbImage) -> Vec<Rgb>;
}

pub trait DynamicBase16Colors {
    fn base00(&self) -> Argb;
    fn base01(&self) -> Argb;
    fn base02(&self) -> Argb;
    fn base03(&self) -> Argb;
    fn base04(&self) -> Argb;
    fn base05(&self) -> Argb;
    fn base06(&self) -> Argb;
    fn base07(&self) -> Argb;
    fn base08(&self) -> Argb;
    fn base09(&self) -> Argb;
    fn base0a(&self) -> Argb;
    fn base0b(&self) -> Argb;
    fn base0c(&self) -> Argb;
    fn base0d(&self) -> Argb;
    fn base0e(&self) -> Argb;
    fn base0f(&self) -> Argb;
}

// Copyright (c) Savchenko Ivan "Aiving" 2023-2024
// https://github.com/Aiving/material-colors/blob/0.4.2/src/dynamic_color/material_dynamic_colors.rs#L15
// Used under MIT License
macro_rules! define_key {
    ($name:ident => $palette:ident; [tone, $scheme_argument:ident] => $tone:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$scheme_argument| $tone,
                false,
                None,
                None,
                None,
                None,
            )
        }
    };

    ($name:ident => $palette:ident; [tone, $tone_scheme_argument:ident] => $tone:expr; [background, $background_scheme_argument:ident] => $background:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$tone_scheme_argument| $tone,
                false,
                Some(|$background_scheme_argument| $background),
                None,
                None,
                None,
            )
        }
    };

    ($name:ident => $palette:ident; [tone, $tone_scheme_argument:ident] => $tone:expr; [background, $background_scheme_argument:ident] => $background:expr; [contrast_curve] => $contrast_curve:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$tone_scheme_argument| $tone,
                false,
                Some(|$background_scheme_argument| $background),
                None,
                Some($contrast_curve),
                None,
            )
        }
    };

    ($name:ident => $palette:ident; [tone, $tone_scheme_argument:ident] => $tone:expr; [background, $background_scheme_argument:ident] => $background:expr; [second_background, $second_background_scheme_argument:ident] => $second_background:expr; [contrast_curve] => $contrast_curve:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$tone_scheme_argument| $tone,
                false,
                Some(|$background_scheme_argument| $background),
                Some(|$second_background_scheme_argument| $second_background),
                Some($contrast_curve),
                None,
            )
        }
    };

    ($name:ident => $palette:ident; [tone, $tone_scheme_argument:ident] => $tone:expr; [background, $background_scheme_argument:ident] => $background:expr; [contrast_curve] => $contrast_curve:expr; [tone_delta_pair, $tone_delta_pair_argument:ident] => $tone_delta_pair:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$tone_scheme_argument| $tone,
                false,
                Some(|$background_scheme_argument| $background),
                None,
                Some($contrast_curve),
                Some(|$tone_delta_pair_argument| $tone_delta_pair),
            )
        }
    };

    (background $name:ident => $palette:ident; [tone, $tone_scheme_argument:ident] => $tone:expr; [background, $background_scheme_argument:ident] => $background:expr; [contrast_curve] => $contrast_curve:expr; [tone_delta_pair, $tone_delta_pair_argument:ident] => $tone_delta_pair:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$tone_scheme_argument| $tone,
                true,
                Some(|$background_scheme_argument| $background),
                None,
                Some($contrast_curve),
                Some(|$tone_delta_pair_argument| $tone_delta_pair),
            )
        }
    };

    (background $name:ident => $palette:ident; [tone, $scheme_argument:ident] => $tone:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$scheme_argument| $tone,
                true,
                None,
                None,
                None,
                None,
            )
        }
    };

    // own additions
    ($name:ident => $palette:ident; [tone chroma, $tone_scheme_argument:ident] => $tone:expr; [background, $background_scheme_argument:ident] => $background:expr; [contrast_curve] => $contrast_curve:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$tone_scheme_argument| {
                    Self::_find_desired_chroma_by_tone(
                        $tone_scheme_argument.$palette.hue(),
                        $tone_scheme_argument.$palette.chroma(),
                        $tone,
                        $tone_scheme_argument.is_dark,
                    )
                },
                false,
                Some(|$background_scheme_argument| $background),
                None,
                Some($contrast_curve),
                None,
            )
        }
    };

    ($name:ident => $palette:ident; [tone chroma, $tone_scheme_argument:ident] => $tone:expr; [background, $background_scheme_argument:ident] => $background:expr; [contrast_curve] => $contrast_curve:expr; [tone_delta_pair, $tone_delta_pair_argument:ident] => $tone_delta_pair:expr;) => {
        pub fn $name() -> DynamicColor {
            DynamicColor::new(
                stringify!($name),
                |scheme| &scheme.$palette,
                |$tone_scheme_argument| {
                    Self::_find_desired_chroma_by_tone(
                        $tone_scheme_argument.$palette.hue(),
                        $tone_scheme_argument.$palette.chroma(),
                        $tone,
                        $tone_scheme_argument.is_dark,
                    )
                },
                false,
                Some(|$background_scheme_argument| $background),
                None,
                Some($contrast_curve),
                Some(|$tone_delta_pair_argument| $tone_delta_pair),
            )
        }
    };
}

// Based off of material_colors::dynamic_color::material_dynamic_colors::MaterialDynamicColors
// See https://docs.rs/material-colors/0.4.2/src/material_colors/dynamic_color/material_dynamic_colors.rs.html
// Semantics: See https://github.com/chriskempson/base16/blob/main/styling.md
pub struct Base16Colors;

impl Base16Colors {
    // Default Background
    define_key! {
        background base00 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 4.0 } else { 98.0 };
    }

    // Lighter Background
    define_key! {
        background base01 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 14.0 } else { 88.0 };
    }

    // Selection Background
    define_key! {
        background base02 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 20.0 } else { 80.0 };
    }

    // Comments, Invisibles
    define_key! {
        base03 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 60.0 } else { 40.0 };
    }

    // Dark Foreground
    define_key! {
        base04 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 80.0 } else { 30.0 };
    }

    // Default Foreground
    define_key! {
        base05 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 90.0 } else { 10.0 };
    }

    // Light Foreground
    define_key! {
        base06 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 94.0 } else { 6.0 };
    }

    // Lightest Foreground
    define_key! {
        base07 => neutral_palette;
        [tone, scheme] => if scheme.is_dark { 96.0 } else { 4.0 };
    }

    // Variables, Diff Deleted
    define_key! {
        base08 => error_palette;
        [tone chroma, scheme] => if scheme.is_dark { 70.0 } else { 30.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
    }

    // Literals
    define_key! {
        base09 => tertiary_palette;
        [tone chroma, scheme] => if scheme.is_dark { 80.0 } else { 30.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
        [tone_delta_pair, _scheme] => ToneDeltaPair::new(Self::base0c(), Self::base09(), 10.0, TonePolarity::Nearer, true);
    }

    // Classes
    define_key! {
        base0a => primary_palette;
        [tone chroma, scheme] => if scheme.is_dark { 70.0 } else { 40.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
        [tone_delta_pair, _scheme] => ToneDeltaPair::new(Self::base0a(), Self::base0d(), 10.0, TonePolarity::Nearer, true);
    }

    // Strings, Diff Inserted
    define_key! {
        base0b => neutral_variant_palette;
        [tone, scheme] => if scheme.is_dark { 80.0 } else { 30.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
        [tone_delta_pair, _scheme] => ToneDeltaPair::new(Self::base0f(), Self::base0b(), 10.0, TonePolarity::Nearer, true);
    }

    // Escape Characters
    define_key! {
        base0c => tertiary_palette;
        [tone chroma, scheme] => if scheme.is_dark { 70.0 } else { 50.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
        [tone_delta_pair, _scheme] => ToneDeltaPair::new(Self::base0c(), Self::base09(), 10.0, TonePolarity::Nearer, true);
    }

    // Functions
    define_key! {
        base0d => primary_palette;
        [tone chroma, scheme] => if scheme.is_dark { 80.0 } else { 30.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
        [tone_delta_pair, _scheme] => ToneDeltaPair::new(Self::base0a(), Self::base0d(), 10.0, TonePolarity::Nearer, true);
    }

    // Keywords, Diff Changed
    define_key! {
        base0e => secondary_palette;
        [tone chroma, scheme] => if scheme.is_dark { 80.0 } else { 30.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
    }

    // Deprecated
    define_key! {
        base0f => neutral_variant_palette;
        [tone, scheme] => if scheme.is_dark { 70.0 } else { 50.0 };
        [background, _scheme] => Self::base00();
        [contrast_curve] => ContrastCurve { low: 3.0, normal: 4.5, medium: 7.0, high: 11.0 };
        [tone_delta_pair, _scheme] => ToneDeltaPair::new(Self::base0f(), Self::base0b(), 10.0, TonePolarity::Nearer, true);
    }

    // Copyright (c) Savchenko Ivan "Aiving" 2023-2024
    // https://github.com/Aiving/material-colors/blob/0.4.2/src/dynamic_color/material_dynamic_colors.rs#L508
    // Used under MIT License
    fn _find_desired_chroma_by_tone(
        hue: f64,
        chroma: f64,
        tone: f64,
        by_decreasing_tone: bool,
    ) -> f64 {
        let mut answer = tone;

        let mut closest_to_chroma = Hct::from(hue, chroma, tone);

        if closest_to_chroma.get_chroma() < chroma {
            let mut chroma_peak = closest_to_chroma.get_chroma();

            while closest_to_chroma.get_chroma() < chroma {
                answer += if by_decreasing_tone { -1.0 } else { 1.0 };

                let potential_solution = Hct::from(hue, chroma, answer);

                if chroma_peak > potential_solution.get_chroma() {
                    break;
                }

                if (potential_solution.get_chroma() - chroma).abs() < 0.4 {
                    break;
                }

                let (potential_delta, current_delta) = (
                    (potential_solution.get_chroma() - chroma).abs(),
                    (closest_to_chroma.get_chroma() - chroma).abs(),
                );

                if potential_delta < current_delta {
                    closest_to_chroma = potential_solution;
                }

                chroma_peak = chroma_peak.max(potential_solution.get_chroma());
            }
        }

        answer
    }
}

macro_rules! from_helper {
    ($name:ident) => {
        fn $name(&self) -> Argb {
            Base16Colors::$name().get_argb(self)
        }
    };
}
impl DynamicBase16Colors for DynamicScheme {
    from_helper!{base00}
    from_helper!{base01}
    from_helper!{base02}
    from_helper!{base03}
    from_helper!{base04}
    from_helper!{base05}
    from_helper!{base06}
    from_helper!{base07}
    from_helper!{base08}
    from_helper!{base09}
    from_helper!{base0a}
    from_helper!{base0b}
    from_helper!{base0c}
    from_helper!{base0d}
    from_helper!{base0e}
    from_helper!{base0f}
}

fn drag_hue(source_hue: f64, target_hue: f64, amount: f64) -> f64 {
    let rot_deg = difference_degrees(source_hue, target_hue);
    let rot_dir = rotate_direction(source_hue, target_hue) * amount;
    sanitize_degrees_double(rot_deg.mul_add(rot_dir, source_hue))
}

pub fn generate_base16_scheme_from_palette(
    palette: &[Rgb],
    dark: bool,
) -> Result<IndexMap<String, Argb>, Report> {
    let mut scheme = IndexMap::new();

    let mut sorted = palette.to_vec();
    sorted.sort_by(|a, b| luminance(b).partial_cmp(&luminance(a)).unwrap());

    let base00 = sorted.first().unwrap();
    let base05 = sorted.last().unwrap();

    scheme.insert("base00".to_string(), argb_from_rgb(base00));
    scheme.insert("base05".to_string(), argb_from_rgb(base05));

    let gray_ramp = interpolate_grays(base00, base05, dark);
    for (i, &name) in GRAY_NAMES.iter().enumerate() {
        scheme.insert(name.to_string(), gray_ramp[i]);
    }

    let mut accents: Vec<&Rgb> = sorted.iter().collect();
    accents.sort_by(|a, b| saturation(b).partial_cmp(&saturation(a)).unwrap());

    for (i, &name) in ACCENT_NAMES.iter().enumerate() {
        scheme.insert(name.to_string(), argb_from_rgb(accents[i % accents.len()]));
    }

    Ok(scheme)
}

pub fn generate_base16_scheme_from_color(
    color: &Rgb,
    dark: bool,
) -> Result<IndexMap<String, Argb>, Report> {
    let mut scheme = IndexMap::new();

    let hsl: Hsl = color.into();
    let (source_hue, source_sat, source_lit) = (hsl.hue(), hsl.saturation(), hsl.lightness());
    let base00: Rgb = Hsl::new(source_hue, source_sat * 0.3, source_lit * 1.5, None).into();
    let base05: Rgb = Hsl::new(source_hue, source_sat * 0.7, source_lit * 0.2, None).into();

    let gray_ramp = interpolate_grays(&base00, &base05, dark);
    for (i, &name) in GRAY_NAMES.iter().enumerate() {
        scheme.insert(name.to_string(), gray_ramp[i]);
    }

    let hct: Hct = argb_from_rgb(color).into();
    let source_chroma = hct.get_chroma();
    let source_tone = hct.get_tone();
    let pri_hue = hct.get_hue();
    let acc_hue = pri_hue + 60.0;
    let red_hue = drag_hue(pri_hue, 25.0, 0.8);
    let grn_hue = drag_hue(pri_hue, 118.0, 0.8);
    let off_hue = 10.0_f64.mul_add(rotate_direction(red_hue, pri_hue), red_hue);
    let main_chroma = source_chroma.max(80.0);
    let mute_chroma = main_chroma / 2.0;
    let depr_chroma = (source_chroma / 6.0).min(10.0);
    let main_tone = source_tone.mul_add(0.3, 50.0);
    let depr_tone = source_tone.mul_add(0.5, 20.0);
    let accent_parameters = [
        (red_hue, main_chroma, main_tone), // Semantics: Variables, Diff Deleted
        (off_hue, mute_chroma, main_tone), // Semantics: Literals
        (pri_hue, mute_chroma, main_tone), // Semantics: Classes
        (grn_hue, main_chroma, main_tone), // Semantics: Strings, Diff Inserted
        (acc_hue, mute_chroma, main_tone), // Semantics: Escape Characters
        (pri_hue, main_chroma, main_tone), // Semantics: Functions
        (acc_hue, main_chroma, main_tone), // Semantics: Keywords, Diff Changed
        (pri_hue, depr_chroma, depr_tone), // Semantics: Deprecated
    ];

    for (i, &name) in ACCENT_NAMES.iter().enumerate() {
        let (hue, chroma, tone) = accent_parameters[i];
        scheme.insert(name.to_string(), Hct::from(hue, chroma, tone).into());
    }

    Ok(scheme)
}

fn interpolate_grays(base00: &Rgb, base05: &Rgb, dark: bool) -> Vec<Argb> {
    let mut grays = Vec::new();
    let n = GRAY_NAMES.len();

    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let r = base00.red() as f32 + t * (base05.red() as f32 - base00.red() as f32);
        let g = base00.green() as f32 + t * (base05.green() as f32 - base00.green() as f32);
        let b = base00.blue() as f32 + t * (base05.blue() as f32 - base00.blue() as f32);
        grays.push(Argb::new(
            255,
            r.round() as u8,
            g.round() as u8,
            b.round() as u8,
        ));
    }

    if dark {
        grays.reverse();
    }

    grays
}

fn generate_base16_from_scheme(scheme: &DynamicScheme) -> IndexMap<String, Argb> {
    let mut ret = IndexMap::new();
    ret.insert("base00".into(), scheme.base00());
    ret.insert("base01".into(), scheme.base01());
    ret.insert("base02".into(), scheme.base02());
    ret.insert("base03".into(), scheme.base03());
    ret.insert("base04".into(), scheme.base04());
    ret.insert("base05".into(), scheme.base05());
    ret.insert("base06".into(), scheme.base06());
    ret.insert("base07".into(), scheme.base07());
    ret.insert("base08".into(), scheme.base08());
    ret.insert("base09".into(), scheme.base09());
    ret.insert("base0a".into(), scheme.base0a());
    ret.insert("base0b".into(), scheme.base0b());
    ret.insert("base0c".into(), scheme.base0c());
    ret.insert("base0d".into(), scheme.base0d());
    ret.insert("base0e".into(), scheme.base0e());
    ret.insert("base0f".into(), scheme.base0f());
    ret
}

pub fn generate_base16(scheme_dark: &DynamicScheme, scheme_light: &DynamicScheme) -> Schemes {
    Schemes {
        dark: generate_base16_from_scheme(scheme_dark),
        light: generate_base16_from_scheme(scheme_light),
    }
}

pub fn generate_base16_schemes(source: &Source, backend: Backend) -> Result<Schemes, Report> {
    let schemes = match source {
        Source::Json { path: _ } => unreachable!(),
        Source::Image { path } => {
            let image = ImageReader::open(path)?
                .with_guessed_format()?
                .decode()?
                .to_rgb8();
            generate_base16_schemes_from_image(&image, backend).wrap_err(format!(
                "Could not generate base16 scheme from image: {}",
                path
            ))?
        }
        Source::Color(color) => generate_base16_schemes_from_color(color).wrap_err(format!(
            "Could not generate base16 scheme from color: {}",
            color.get_string()
        ))?,
        #[cfg(feature = "web-image")]
        Source::WebImage { url } => {
            let bytes = reqwest::blocking::get(url)?.bytes()?;
            let image = image::load_from_memory(&bytes)?.to_rgb8();
            generate_base16_schemes_from_image(&image, backend).wrap_err(format!(
                "Could not generate base16 scheme from image: {}",
                url
            ))?
        }
    };
    Ok(schemes)
}

pub fn generate_base16_schemes_from_image(
    image: &RgbImage,
    backend: Backend,
) -> Result<Schemes, Report> {
    let palette = backend.create().extract(&image);

    let dark_scheme = generate_base16_scheme_from_palette(&palette, true)?;
    let light_scheme = generate_base16_scheme_from_palette(&palette, false)?;

    Ok(Schemes {
        dark: dark_scheme,
        light: light_scheme,
    })
}

pub fn generate_base16_schemes_from_color(color: &ColorFormat) -> Result<Schemes, Report> {
    let source_color = rgb_from_argb(get_source_color_from_color(color)?);

    let dark_scheme = generate_base16_scheme_from_color(&source_color, true)?;
    let light_scheme = generate_base16_scheme_from_color(&source_color, false)?;

    Ok(Schemes {
        dark: dark_scheme,
        light: light_scheme,
    })
}
