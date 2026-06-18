//! Per-cell color extraction from prepared source images.

use std::collections::HashMap;

use image::{DynamicImage, GenericImageView};

use super::types::ColorMethod;

pub fn sample_cell_color(
    img: &DynamicImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    method: ColorMethod,
) -> [u8; 3] {
    match method {
        ColorMethod::Average => sample_color_average(img, x, y, w, h),
        ColorMethod::Absolute => sample_color_absolute(img, x, y, w, h),
        ColorMethod::Sharp => sample_color_sharp(img, x, y, w, h),
    }
}

pub fn sample_color_average(img: &DynamicImage, x: u32, y: u32, w: u32, h: u32) -> [u8; 3] {
    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;
    let mut count = 0u64;
    let x_end = (x + w).min(img.width());
    let y_end = (y + h).min(img.height());

    for py in y..y_end {
        for px in x..x_end {
            let pixel = img.get_pixel(px, py);
            r_sum += pixel.0[0] as u64;
            g_sum += pixel.0[1] as u64;
            b_sum += pixel.0[2] as u64;
            count += 1;
        }
    }

    if count == 0 {
        [0, 0, 0]
    } else {
        [
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
        ]
    }
}

/// Quantize a channel to 16 levels so nearby pixels bucket together for mode detection.
fn quantize_channel(value: u8) -> u8 {
    (value / 16) * 16
}

pub fn sample_color_absolute(img: &DynamicImage, x: u32, y: u32, w: u32, h: u32) -> [u8; 3] {
    let x_end = (x + w).min(img.width());
    let y_end = (y + h).min(img.height());

    if x >= x_end || y >= y_end {
        return [0, 0, 0];
    }

    let mut counts: HashMap<[u8; 3], u32> = HashMap::new();
    for py in y..y_end {
        for px in x..x_end {
            let pixel = img.get_pixel(px, py);
            let key = [
                quantize_channel(pixel.0[0]),
                quantize_channel(pixel.0[1]),
                quantize_channel(pixel.0[2]),
            ];
            *counts.entry(key).or_insert(0) += 1;
        }
    }

    let mut best_key = [0u8; 3];
    let mut best_count = 0u32;
    for (key, count) in counts {
        if count > best_count || (count == best_count && key < best_key) {
            best_count = count;
            best_key = key;
        }
    }

    best_key
}

pub fn sample_color_sharp(img: &DynamicImage, x: u32, y: u32, w: u32, h: u32) -> [u8; 3] {
    let img_w = img.width();
    let img_h = img.height();

    if img_w == 0 || img_h == 0 {
        return [0, 0, 0];
    }

    let px = if w == 0 { x } else { x + w / 2 };
    let py = if h == 0 { y } else { y + h / 2 };
    let px = px.min(img_w.saturating_sub(1));
    let py = py.min(img_h.saturating_sub(1));

    let pixel = img.get_pixel(px, py);
    [pixel.0[0], pixel.0[1], pixel.0[2]]
}
