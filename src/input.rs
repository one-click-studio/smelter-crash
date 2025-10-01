use anyhow::Result;
use compositor_pipeline::pipeline::input::{mp4::*, InputOptions};
use compositor_pipeline::pipeline::{RegisterInputOptions, VideoDecoder};
use compositor_pipeline::queue::QueueInputOptions;
use compositor_pipeline::Pipeline;
use compositor_render::scene::*;
use compositor_render::web_renderer::{WebEmbeddingMethod, WebRendererSpec};
use compositor_render::{InputId, RendererId, RendererSpec, Resolution};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::info;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const WEB_URL: &str = "https://google.com";
const MP4_INPUT: &str = "test.mp4";

pub fn setup_mp4_input(pipeline: &Arc<Mutex<Pipeline>>) -> Result<Component> {
    let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let video_path = assets_path.join(MP4_INPUT);
    let video_input_id = InputId(Arc::from("video_input"));

    let input_options = InputOptions::Mp4(Mp4Options {
        source: Source::File(video_path.clone()),
        should_loop: true,
        video_decoder: VideoDecoder::FFmpegH264,
    });

    Pipeline::register_input(
        pipeline,
        video_input_id.clone(),
        RegisterInputOptions {
            input_options,
            queue_options: QueueInputOptions {
                required: false,
                offset: None,
                buffer_duration: Some(Duration::ZERO),
            },
        },
    )?;
    info!("Registered MP4 input: {}", video_path.display());

    // Create scene with MP4 input wrapped in a Rescaler
    Ok(Component::Rescaler(RescalerComponent {
        id: None,
        child: Box::new(Component::InputStream(InputStreamComponent {
            id: None,
            input_id: video_input_id.clone(),
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
