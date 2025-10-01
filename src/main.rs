use anyhow::{anyhow, Context, Result};
use compositor_pipeline::pipeline::encoder::*;
use compositor_pipeline::pipeline::input::{mp4::*, InputOptions};
use compositor_pipeline::pipeline::output::*;
use compositor_pipeline::pipeline::*;
use compositor_pipeline::queue::QueueInputOptions;
use compositor_pipeline::Pipeline;
use compositor_render::scene::*;
use compositor_render::web_renderer::{WebEmbeddingMethod, WebRendererSpec};
use compositor_render::{Framerate, InputId, OutputId, RendererId, RendererSpec, Resolution};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::info;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const WEB_URL: &str = "https://google.com";
const MP4_INPUT: &str = "test.mp4";
const OUTPUT_VIDEO: &str = "output.mp4";

enum InputSource {
    Mp4,
    Web,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    let mut use_web = false;
    let mut duration_arg = None;
    let mut allocate_ram: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--web" {
            use_web = true;
            i += 1;
        } else if arg == "--ram" {
            if i + 1 >= args.len() {
                return Err(anyhow!("--ram requires a value (e.g., 100M, 2G)"));
            }
            allocate_ram = Some(args[i + 1].clone());
            i += 2;
        } else if duration_arg.is_none() {
            duration_arg = Some(arg.clone());
            i += 1;
        } else {
            return Err(anyhow!("Unknown argument: {}", arg));
        }
    }

    let duration = match duration_arg {
        Some(d) => parse_duration(&d)?,
        None => {
            eprintln!("Usage: {} [--web] [--ram <size>] <duration>", args[0]);
            eprintln!("");
            eprintln!("Arguments:");
            eprintln!("  <duration>        Duration to record (e.g., 5s, 10m, 2h, 6h30m)");
            eprintln!("  --web             Use web renderer instead of MP4 input (default: MP4)");
            eprintln!("  --ram <size>      Allocate memory before starting (e.g., 100M, 2G)");
            eprintln!("");
            eprintln!("Examples:");
            eprintln!("  {} 5s                - Record MP4 for 5 seconds", args[0]);
            eprintln!("  {} --web 10m         - Record web page for 10 minutes", args[0]);
            eprintln!("  {} --ram 500M 5s     - Allocate 500MB RAM and record MP4", args[0]);
            eprintln!("  {} --ram 2G --web 1h - Allocate 2GB RAM and record web page", args[0]);
            return Err(anyhow!("Missing duration argument"));
        }
    };

    // Initialize logging early
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_env_filter("smelter_crash=info,compositor_pipeline=warn,compositor_render=warn,compositor_chromium=info")
        .init();

    info!("Starting minimal smelter compositor");

    // Allocate RAM if requested (before initializing compositor)
    let _allocated_memory = if let Some(ram_size) = allocate_ram {
        let bytes = parse_memory_size(&ram_size)?;
        info!("Allocating {} bytes ({}) of RAM...", bytes, ram_size);
        let memory: Vec<u8> = vec![0; bytes];
        info!("Successfully allocated {} of RAM", ram_size);
        Some(memory)
    } else {
        None
    };

    let input_source = if use_web {
        InputSource::Web
    } else {
        InputSource::Mp4
    };

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
            enable: matches!(input_source, InputSource::Web),
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

    // Create scene based on input source
    let scene = match input_source {
        InputSource::Web => {
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
            Component::Rescaler(RescalerComponent {
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
            })
        }
        InputSource::Mp4 => {
            // Register MP4 input
            let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
            let video_path = assets_path.join(MP4_INPUT);
            let video_input_id = InputId(Arc::from("video_input"));

            let input_options = InputOptions::Mp4(Mp4Options {
                source: Source::File(video_path.clone()),
                should_loop: true,
                video_decoder: VideoDecoder::FFmpegH264,
            });

            Pipeline::register_input(
                &pipeline,
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
            Component::Rescaler(RescalerComponent {
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
            })
        }
    };

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

    // Handle event loop based on input source
    match input_source {
        InputSource::Web => {
            // Web rendering requires the event loop to run on the main thread
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
        }
        InputSource::Mp4 => {
            // MP4 input doesn't need event loop, just wait for duration
            std::thread::sleep(duration);

            // Stop recording
            let mut pipeline_lock = pipeline.lock().unwrap();
            Pipeline::unregister_output(&mut *pipeline_lock, &output_id)?;
            drop(pipeline_lock);

            // Give it a moment to finalize
            std::thread::sleep(Duration::from_secs(1));

            info!("Recording complete: {}", output_path.display());
        }
    }

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

fn parse_memory_size(input: &str) -> Result<usize> {
    let input = input.trim().to_uppercase();

    // Find where the number ends and the unit begins
    let split_pos = input
        .chars()
        .position(|c| !c.is_ascii_digit())
        .unwrap_or(input.len());

    let (num_str, unit_str) = input.split_at(split_pos);

    if num_str.is_empty() {
        return Err(anyhow!("Invalid memory size format: missing number"));
    }

    let num: usize = num_str
        .parse()
        .context(format!("Failed to parse number: {}", num_str))?;

    let multiplier: usize = match unit_str.trim() {
        "" | "B" => 1,                          // Bytes
        "K" | "KB" => 1024,                     // Kilobytes
        "M" | "MB" => 1024 * 1024,              // Megabytes
        "G" | "GB" => 1024 * 1024 * 1024,       // Gigabytes
        _ => return Err(anyhow!("Invalid memory unit: '{}'. Use B, K/KB, M/MB, or G/GB", unit_str)),
    };

    num.checked_mul(multiplier)
        .ok_or_else(|| anyhow!("Memory size too large: {} would overflow", input))
}
