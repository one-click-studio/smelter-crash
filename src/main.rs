use anyhow::{anyhow, Context, Result};
use compositor_pipeline::pipeline::encoder::*;
use compositor_pipeline::pipeline::output::*;
use compositor_pipeline::pipeline::*;
use compositor_pipeline::Pipeline;
use compositor_render::scene::*;
use compositor_render::web_renderer::{WebEmbeddingMethod, WebRendererSpec};
use compositor_render::{Framerate, OutputId, RendererId, RendererSpec, Resolution};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::info;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const WEB_URL: &str = "https://google.com";
const OUTPUT_VIDEO: &str = "output.mp4";

fn main() -> Result<()> {
    // Parse duration from command line argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <duration>", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} 5s      - Record for 5 seconds", args[0]);
        eprintln!("  {} 10m     - Record for 10 minutes", args[0]);
        eprintln!("  {} 2h      - Record for 2 hours", args[0]);
        eprintln!("  {} 6h30m   - Record for 6 hours and 30 minutes", args[0]);
        return Err(anyhow!("Invalid number of arguments"));
    }

    let duration = parse_duration(&args[1])?;
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_env_filter("smelter_crash=info,compositor_pipeline=warn,compositor_render=warn,compositor_chromium=info")
        .init();

    info!("Starting minimal smelter compositor");

    // Remove existing output file if it exists
    let output_path = PathBuf::from(OUTPUT_VIDEO);
    if output_path.exists() {
        std::fs::remove_file(&output_path)?;
        info!("Removed existing output file");
    }

    // Initialize graphics context
    let graphics_context = GraphicsContext::new(GraphicsContextOptions {
        force_gpu: false,
        features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::TEXTURE_BINDING_ARRAY,
        limits: wgpu::Limits::default(),
        compatible_surface: None,
        libvulkan_path: None,
    })
    .context("Failed to initialize WGPU")?;

    // Create pipeline
    let (pipeline, event_loop) = Pipeline::new(compositor_pipeline::pipeline::Options {
        queue_options: compositor_pipeline::queue::QueueOptions {
            default_buffer_duration: Duration::ZERO,
            ahead_of_time_processing: false,
            output_framerate: Framerate { num: 30, den: 1 },
            run_late_scheduled_events: true,
            never_drop_output_frames: false,
        },
        stream_fallback_timeout: Duration::from_millis(500),
        web_renderer: compositor_render::web_renderer::WebRendererInitOptions {
            enable: true,
            enable_gpu: false,
        },
        force_gpu: false,
        download_root: std::env::temp_dir(),
        mixing_sample_rate: 48000,
        wgpu_features: wgpu::Features::PUSH_CONSTANTS | wgpu::Features::TEXTURE_BINDING_ARRAY,
        load_system_fonts: None,
        wgpu_ctx: Some(graphics_context),
        stun_servers: Default::default(),
        whip_whep_server_port: 9000,
        start_whip_whep: false,
        tokio_rt: None,
        rendering_mode: compositor_render::RenderingMode::GpuOptimized,
    })
    .context("Failed to create compositor pipeline")?;

    let pipeline = Arc::new(Mutex::new(pipeline));
    Pipeline::start(&pipeline);
    info!("Pipeline started");

    // Register web renderer
    let web_renderer_id = RendererId(Arc::from("web_renderer"));
    Pipeline::register_renderer(
        &pipeline,
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
    let scene = Component::Rescaler(RescalerComponent {
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
    });

    // Register MP4 output
    let output_id = OutputId(Arc::from("mp4_output"));
    Pipeline::register_output(
        &pipeline,
        output_id.clone(),
        RegisterOutputOptions {
            output_options: OutputOptions::Mp4(mp4::Mp4OutputOptions {
                output_path: output_path.clone(),
                video: Some(VideoEncoderOptions::H264(ffmpeg_h264::Options {
                    preset: ffmpeg_h264::EncoderPreset::Medium,
                    resolution: Resolution {
                        width: WIDTH,
                        height: HEIGHT,
                    },
                    raw_options: vec![],
                    pixel_format: OutputPixelFormat::YUV420P,
                })),
                audio: None,
            }),
            video: Some(OutputVideoOptions {
                initial: scene,
                end_condition: PipelineOutputEndCondition::Never,
            }),
            audio: None,
        },
    )?;
    info!("Started recording to {} for {:?}", output_path.display(), duration);

    // Spawn a thread to handle the recording duration and stop
    let pipeline_clone = pipeline.clone();
    let output_id_clone = output_id.clone();
    let output_path_clone = output_path.clone();
    std::thread::spawn(move || {
        // Record for specified duration
        std::thread::sleep(duration);

        // Stop recording
        let mut pipeline_lock = pipeline_clone.lock().unwrap();
        if let Err(e) = Pipeline::unregister_output(&mut *pipeline_lock, &output_id_clone) {
            eprintln!("Error unregistering output: {}", e);
        }
        drop(pipeline_lock);

        // Give it a moment to finalize
        std::thread::sleep(Duration::from_secs(1));

        info!("Recording complete: {}", output_path_clone.display());
        std::process::exit(0);
    });

    // Run the event loop on the main thread (required for CEF/Chromium)
    info!("Starting event loop (required for web rendering)");
    event_loop.run().context("Failed to run event loop")?;

    Ok(())
}

fn parse_duration(input: &str) -> Result<Duration> {
    let input = input.trim();
    let mut total_secs = 0u64;
    let mut current_num = String::new();

    for ch in input.chars() {
        if ch.is_ascii_digit() {
            current_num.push(ch);
        } else if ch == 's' || ch == 'm' || ch == 'h' {
            if current_num.is_empty() {
                return Err(anyhow!("Invalid duration format: missing number before '{}'", ch));
            }
            let num: u64 = current_num.parse()
                .context(format!("Failed to parse number: {}", current_num))?;

            let multiplier = match ch {
                's' => 1,
                'm' => 60,
                'h' => 3600,
                _ => unreachable!(),
            };

            total_secs += num * multiplier;
            current_num.clear();
        } else if !ch.is_whitespace() {
            return Err(anyhow!("Invalid character '{}' in duration. Use only numbers and s/m/h", ch));
        }
    }

    if !current_num.is_empty() {
        return Err(anyhow!("Invalid duration format: trailing number without unit (s/m/h)"));
    }

    if total_secs == 0 {
        return Err(anyhow!("Duration must be greater than 0"));
    }

    Ok(Duration::from_secs(total_secs))
}
