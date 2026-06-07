use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_render::renderer::ViewQuery;
use bevy_render::view::ViewTarget;
use bevy_render::{
    Extract,
    renderer::RenderContext,
};
use bevy_window::prelude::*;
use iced_core::Size;
use iced_wgpu::wgpu::TextureFormat;
use iced_widget::graphics::Viewport;

use crate::systems::IcedCamera;
use crate::{DidDraw, IcedResource, IcedSettings};

pub const TEXTURE_FMT: TextureFormat = TextureFormat::Rgba8UnormSrgb; // must be equal to the format of the iced 2D camera

#[derive(Resource, Deref, DerefMut, Clone)]
pub struct IcedViewport(pub Viewport);

pub fn update_viewport(
    windows: Query<&Window>,
    iced_settings: Res<IcedSettings>,
    mut commands: Commands,
) {
    let window = windows.single().unwrap();
    let scale_factor = if let Some(scale_factor) = iced_settings.scale_factor {
        scale_factor as f32
    } else {
        window.scale_factor()
    };
    let viewport = Viewport::with_physical_size(
        Size::new(window.physical_width(), window.physical_height()),
        scale_factor,
    );
    commands.insert_resource(IcedViewport(viewport));
}

// Same as DidDraw, but as a regular bool instead of an atomic.
#[derive(Resource, Deref, DerefMut)]
pub struct DidDrawBasic(bool);

pub fn extract_iced_data(
    mut commands: Commands,
    viewport: Extract<Res<IcedViewport>>,
    did_draw: Extract<Res<DidDraw>>,
) {
    commands.insert_resource(viewport.clone());
    commands.insert_resource(DidDrawBasic(
        did_draw.swap(false, std::sync::atomic::Ordering::Relaxed),
    ));
}

pub fn recall_staging_belt(
    #[cfg(target_arch = "wasm32")] iced: NonSend<IcedResource>,
    #[cfg(not(target_arch = "wasm32"))] iced: Res<IcedResource>,
) {
    iced.lock().renderer.recall();
}


pub fn iced_render_pass(
    mut render_context: RenderContext,
    view: ViewQuery<(&ViewTarget, Has<IcedCamera>)>,
    did_draw_basic: Option<Res<DidDrawBasic>>,
    viewport: Res<IcedViewport>,
    #[cfg(target_arch = "wasm32")] iced_resource: NonSendMut<IcedResource>,
    #[cfg(not(target_arch = "wasm32"))] iced_resource: ResMut<IcedResource>,
) {
    let (view_target, is_iced) = view.into_inner();
    if !is_iced {
        return;
    }

    if !did_draw_basic.is_some_and(|x| x.0) {
        return;
    }

    let texture_view = view_target.main_texture_view();

    let renderer = &mut iced_resource.lock().renderer;

    let encoder = renderer.draw(None, texture_view, &viewport);
    render_context.add_command_buffer(encoder.finish());
    renderer.finish();
}
