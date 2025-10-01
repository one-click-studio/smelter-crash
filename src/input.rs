use anyhow::Result;
use compositor_pipeline::Pipeline;
use compositor_render::scene::*;
use compositor_render::web_renderer::{WebEmbeddingMethod, WebRendererSpec};
use compositor_render::{RendererId, RendererSpec, Resolution};
use std::sync::{Arc, Mutex};
use tracing::info;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const WEB_URL: &str = "https://google.com";

pub fn setup_web_input(pipeline: &Arc<Mutex<Pipeline>>) -> Result<Component> {
    let web_renderer_id = RendererId(Arc::from("web_renderer"));
    Pipeline::register_renderer(
        pipeline,
        web_renderer_id.clone(),
        RendererSpec::WebRenderer(WebRendererSpec {
            url: WEB_URL.to_string(),
            resolution: Resolution {
                width: WIDTH,
                height: HEIGHT,
            },
            embedding_method: WebEmbeddingMethod::NativeEmbeddingOverContent,
        }),
    )?;
    info!("Registered web renderer: {}", WEB_URL);

    // Create scene with web renderer wrapped in a Rescaler
    Ok(Component::Rescaler(RescalerComponent {
        id: None,
        child: Box::new(Component::WebView(WebViewComponent {
            id: None,
            children: vec![],
            instance_id: web_renderer_id.clone(),
        })),
        position: Position::Static {
            width: None,
            height: None,
        },
        transition: None,
        mode: RescaleMode::Fit,
        horizontal_align: HorizontalAlign::Center,
        vertical_align: VerticalAlign::Center,
        border_radius: BorderRadius::ZERO,
        border_width: 0.0,
        border_color: RGBAColor(0, 0, 0, 0),
        box_shadow: vec![],
    }))
}

pub fn resolution() -> Resolution {
    Resolution {
        width: WIDTH,
        height: HEIGHT,
    }
}
