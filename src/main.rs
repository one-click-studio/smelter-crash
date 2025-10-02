mod args;
mod input;
mod memory_monitor;
mod output;
mod ram;

use anyhow::{Context, Result};
use compositor_pipeline::pipeline::GraphicsContext;
use compositor_pipeline::Pipeline;
use compositor_render::{EventLoop, Framerate, OutputId};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::info;

fn main() -> Result<()> {
    // Parse command line arguments
    let args = args::Args::parse()?;

    // Initialize logging early
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_env_filter("smelter_crash=info,compositor_pipeline=warn,compositor_render=warn,compositor_chromium=info")
        .init();

    info!("Starting minimal smelter compositor");

    // Start memory monitor
    memory_monitor::start_memory_monitor();

    // Allocate and hold RAM if requested
    if let Some(ram_size) = args.allocate_ram {
        ram::allocate_and_hold(ram_size)?;
    }

    // Initialize graphics context
    let graphics_context = GraphicsContext::new(compositor_pipeline::pipeline::GraphicsContextOptions {
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
            never_drop_output_frames: true, // Never drop frames - use blocking send instead of send_deadline
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

    // Setup web input
    let scene = input::setup_web_input(&pipeline)?;

    // Setup raw output
    let output_id = output::setup_raw_output(&pipeline, scene, input::resolution())?;

    // Run with event loop (required for web rendering)
    run_with_event_loop(event_loop, pipeline, output_id)?;

    Ok(())
}

fn run_with_event_loop(
    event_loop: Arc<dyn EventLoop>,
    _pipeline: Arc<Mutex<Pipeline>>,
    _output_id: OutputId,
) -> Result<()> {
    // Raw output mode: run indefinitely
    info!("Running in raw output mode (press Ctrl+C to exit)");

    // Run the CEF event loop on the main thread
    event_loop.run().context("Failed to run event loop")?;

    Ok(())
}
