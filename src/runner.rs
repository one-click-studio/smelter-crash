use anyhow::{Context, Result};
use compositor_pipeline::Pipeline;
use compositor_render::{EventLoop, OutputId};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::info;

pub fn run_with_event_loop(
    event_loop: Arc<dyn EventLoop>,
    pipeline: Arc<Mutex<Pipeline>>,
    output_id: OutputId,
    duration: Option<Duration>,
) -> Result<()> {
    // Web rendering requires the event loop to run on the main thread
    if let Some(duration) = duration {
        // Recording mode: spawn a thread to handle the recording duration and exit
        thread::spawn(move || {
            // Record for specified duration
            thread::sleep(duration);

            // Stop recording
            let mut pipeline_lock = pipeline.lock().unwrap();
            if let Err(e) = Pipeline::unregister_output(&mut *pipeline_lock, &output_id) {
                eprintln!("Error unregistering output: {}", e);
            }
            drop(pipeline_lock);

            // Give it a moment to finalize
            thread::sleep(Duration::from_secs(1));

            info!("Recording complete");
            std::process::exit(0);
        });
    } else {
        // Raw output mode: log that we're running indefinitely
        info!("Running in raw output mode (press Ctrl+C to exit)");
    }

    // Run the event loop on the main thread (required for CEF/Chromium)
    info!("Starting event loop (required for web rendering)");
    event_loop.run().context("Failed to run event loop")?;

    Ok(())
}

pub fn run_without_event_loop(
    pipeline: Arc<Mutex<Pipeline>>,
    output_id: OutputId,
    duration: Option<Duration>,
) -> Result<()> {
    if let Some(duration) = duration {
        // Recording mode: wait for duration, then stop recording and exit
        thread::sleep(duration);

        // Stop recording
        let mut pipeline_lock = pipeline.lock().unwrap();
        Pipeline::unregister_output(&mut *pipeline_lock, &output_id)?;
        drop(pipeline_lock);

        // Give it a moment to finalize
        thread::sleep(Duration::from_secs(1));

        info!("Recording complete");
    } else {
        // Raw output mode: run indefinitely
        info!("Running in raw output mode (press Ctrl+C to exit)");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }

    Ok(())
}
