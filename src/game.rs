use crate::{
    asset::{Object, Sprite},
    atlas::AtlasBuilder,
    instance::Instance,
    instancelist::InstanceList,
    render::{opengl::OpenGLRenderer, Renderer, RendererOptions},
};
use gm8exe::{rsrc::WindowsIcon, GameAssets};

use std::convert::identity;

/// Resolves icon closest to preferred_width and converts it from a WindowsIcon to proper RGBA pixels.
fn get_icon(icons: &[WindowsIcon], preferred_width: i32) -> Option<(Vec<u8>, u32, u32)> {
    fn closest<'a, I: Iterator<Item = &'a WindowsIcon>>(preferred_width: i32, i: I) -> Option<&'a WindowsIcon> {
        i.min_by(|a, b| {
            (a.width as i32 - preferred_width)
                .abs()
                .cmp(&(b.width as i32 - preferred_width).abs())
        })
    }

    fn icon_from_win32(raw: &[u8], width: usize) -> Option<(Vec<u8>, u32, u32)> {
        let mut rgba = Vec::with_capacity(raw.len());
        for chunk in raw.rchunks_exact(width * 4) {
            rgba.extend_from_slice(chunk);
            let vlen = rgba.len();
            crate::util::bgra2rgba(rgba.get_mut(vlen - (width * 4)..)?);
        }
        Some((rgba, width as u32, width as u32))
    }

    closest(
        preferred_width,
        icons.iter().filter(|i| i.original_bpp == 24 || i.original_bpp == 32),
    )
    .or_else(|| closest(preferred_width, icons.iter()))
    .and_then(|i| icon_from_win32(&i.bgra_data, i.width as usize))
}

pub fn launch(assets: GameAssets) {
    // destructure assets
    let GameAssets {
        room_order,
        rooms,
        sprites,
        objects,
        ..
    } = assets;

    // If there are no rooms, you can't build a GM8 game. Fatal error.
    // We need a lot of the initialization info from the first room,
    // the window size, and title, etc. is based on it.
    let room1 = room_order
        .first() // first index
        .map(|x| rooms.get(*x as usize))
        .and_then(identity) // Option<Option<T>> -> Option<T>
        .and_then(|x| x.as_ref()) // Option<&Option<T>> -> Option<&T>
        .map(|r| r.as_ref()) // Option<&Box<T>> -> Option<&T>
        .unwrap();

    let options = RendererOptions {
        title: &room1.caption,
        size: (room1.width, room1.height),
        icon: get_icon(&assets.icon_data, 32),
        resizable: assets.settings.allow_resize,
        on_top: assets.settings.window_on_top,
        decorations: !assets.settings.dont_draw_border,
        fullscreen: assets.settings.fullscreen,
        vsync: assets.settings.vsync, // TODO: Overrideable
    };

    let mut renderer = OpenGLRenderer::new(options).unwrap();
    let mut atlases = AtlasBuilder::new(renderer.max_gpu_texture_size() as _);

    //println!("GPU Max Texture Size: {}", renderer.max_gpu_texture_size());

    let sprites = sprites
        .into_iter()
        .map(|o| {
            o.map(|b| {
                let (w, h) = b.frames.first().map_or((0, 0), |f| (f.width, f.height));
                Box::new(Sprite {
                    name: b.name,
                    frames: b
                        .frames
                        .into_iter()
                        .map(|f| atlases.texture(f.width as _, f.height as _, f.data).unwrap())
                        .collect(),
                    width: w,
                    height: h,
                    origin_x: b.origin_x,
                    origin_y: b.origin_y,
                })
            })
        })
        .collect::<Vec<_>>();

    let objects = objects
        .into_iter()
        .map(|o| {
            o.map(|b| {
                Box::new(Object {
                    name: b.name,
                    solid: b.solid,
                    visible: b.visible,
                    persistent: b.persistent,
                    depth: b.depth,
                    sprite_index: b.sprite_index,
                    mask_index: b.mask_index,
                })
            })
        })
        .collect::<Vec<_>>();

    renderer.upload_atlases(atlases).unwrap();

    let mut instance_list = InstanceList::new();

    for instance in &room1.instances {
        let object = match objects.get(instance.object as usize) {
            Some(&Some(ref o)) => o.as_ref(),
            _ => panic!("Instance of invalid Object in room {}", room1.name),
        };
        instance_list.insert(Instance::new(
            instance.id as _,
            f64::from(instance.x),
            f64::from(instance.y),
            instance.object,
            object,
        ));
    }

    while !renderer.should_close() {
        for (_, instance) in instance_list.iter() {
            if let Some(Some(sprite)) = sprites.get(instance.sprite_index as usize) {
                renderer.draw_sprite(
                    sprite.frames.first().unwrap(),
                    instance.x,
                    instance.y,
                    instance.image_xscale,
                    instance.image_yscale,
                    instance.image_angle,
                    instance.image_blend,
                    instance.image_alpha,
                )
            }
        }
        renderer.draw();
    }

    // renderer.dump_atlases(|i| std::path::PathBuf::from(format!("./atl{}.png", i))).unwrap();
}
