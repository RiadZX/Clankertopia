//! Procedurally-generated textures so we ship no extra asset files.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Resource)]
pub struct ProcTextures {
    pub wood_floor: Handle<Image>,
    pub carpet: Handle<Image>,
    pub plaster: Handle<Image>,
    pub ceiling_tile: Handle<Image>,
    pub desk_wood: Handle<Image>,
    pub poster_grid: Handle<Image>,
    pub rug_pattern: Handle<Image>,
    pub metal: Handle<Image>,
}

impl ProcTextures {
    pub fn generate(images: &mut Assets<Image>) -> Self {
        Self {
            wood_floor: images.add(make_wood_planks(512, 512, (140, 95, 55), (105, 70, 38))),
            carpet: images.add(make_noise(256, 256, (60, 45, 70), 18)),
            plaster: images.add(make_noise(256, 256, (235, 225, 205), 6)),
            ceiling_tile: images.add(make_tiles(256, 256, (245, 245, 245), (210, 210, 210), 64)),
            desk_wood: images.add(make_wood_planks(256, 256, (130, 85, 55), (95, 60, 35))),
            poster_grid: images.add(make_poster(256, 256)),
            rug_pattern: images.add(make_rug(256, 256)),
            metal: images.add(make_noise(128, 128, (170, 175, 180), 5)),
        }
    }
}

fn rng_xy(x: u32, y: u32, salt: u32) -> u32 {
    let mut s = x
        .wrapping_mul(0x9E3779B1)
        .wrapping_add(y.wrapping_mul(0x85EBCA77))
        .wrapping_add(salt.wrapping_mul(0xC2B2AE3D));
    s ^= s >> 13;
    s = s.wrapping_mul(0x27D4EB2D);
    s ^ (s >> 15)
}

fn clamp_u8(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

fn build_image(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    let mut img = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::default()
    });
    img
}

fn make_noise(w: u32, h: u32, base: (u8, u8, u8), amp: i32) -> Image {
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let n = (rng_xy(x, y, 1) as i32 % (amp * 2 + 1)) - amp;
            data.push(clamp_u8(base.0 as i32 + n));
            data.push(clamp_u8(base.1 as i32 + n));
            data.push(clamp_u8(base.2 as i32 + n));
            data.push(255);
        }
    }
    build_image(w, h, data)
}

fn make_wood_planks(w: u32, h: u32, light: (u8, u8, u8), dark: (u8, u8, u8)) -> Image {
    let plank_h = 48u32; // planks running horizontally
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let plank_idx = y / plank_h;
        let offset_in_plank = y % plank_h;
        let plank_seed = plank_idx.wrapping_mul(31);
        let plank_dark = (rng_xy(0, plank_idx, 7) % 100) < 35;
        let base = if plank_dark { dark } else { light };
        for x in 0..w {
            // Grain stripes
            let grain = (rng_xy(x / 4, plank_idx, plank_seed) as i32 % 30) - 15;
            // Plank boundary darkening
            let near_edge = offset_in_plank == 0 || offset_in_plank == plank_h - 1;
            let edge_pen: i32 = if near_edge { -45 } else { 0 };
            let r = clamp_u8(base.0 as i32 + grain + edge_pen);
            let g = clamp_u8(base.1 as i32 + grain + edge_pen);
            let b = clamp_u8(base.2 as i32 + grain + edge_pen);
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(255);
        }
    }
    build_image(w, h, data)
}

fn make_tiles(w: u32, h: u32, base: (u8, u8, u8), seam: (u8, u8, u8), tile: u32) -> Image {
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let in_seam = (x % tile) == 0 || (y % tile) == 0;
            let c = if in_seam { seam } else { base };
            let n = (rng_xy(x, y, 3) as i32 % 12) - 6;
            data.push(clamp_u8(c.0 as i32 + n));
            data.push(clamp_u8(c.1 as i32 + n));
            data.push(clamp_u8(c.2 as i32 + n));
            data.push(255);
        }
    }
    build_image(w, h, data)
}

fn make_poster(w: u32, h: u32) -> Image {
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let in_border = x < 6 || y < 6 || x >= w - 6 || y >= h - 6;
            let stripe = ((x / 32) + (y / 32)) % 2 == 0;
            let (r, g, b) = if in_border {
                (40, 30, 60)
            } else if stripe {
                (245, 180, 90)
            } else {
                (60, 110, 200)
            };
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(255);
        }
    }
    build_image(w, h, data)
}

fn make_rug(w: u32, h: u32) -> Image {
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    let cx = w as i32 / 2;
    let cy = h as i32 / 2;
    for y in 0..h {
        for x in 0..w {
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            let ring = ((dx * dx + dy * dy) as f32).sqrt() as i32;
            let band = (ring / 16) % 3;
            let (r, g, b) = match band {
                0 => (160, 60, 60),
                1 => (190, 150, 80),
                _ => (90, 50, 50),
            };
            let n = (rng_xy(x, y, 11) as i32 % 14) - 7;
            data.push(clamp_u8(r as i32 + n));
            data.push(clamp_u8(g as i32 + n));
            data.push(clamp_u8(b as i32 + n));
            data.push(255);
        }
    }
    build_image(w, h, data)
}
