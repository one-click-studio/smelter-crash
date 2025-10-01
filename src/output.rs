use anyhow::Result;
use compositor_pipeline::pipeline::output::*;
use compositor_pipeline::pipeline::{OutputVideoOptions, PipelineOutputEndCondition, RegisterOutputOptions};
use compositor_pipeline::Pipeline;
use compositor_render::scene::Component;
use compositor_render::{OutputId, Resolution};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::info;

pub fn setup_raw_output(
    pipeline: &Arc<Mutex<Pipeline>>,
    scene: Component,
    resolution: Resolution,
) -> Result<OutputId> {
    let output_id = OutputId(Arc::from("output"));
    let receiver = Pipeline::register_raw_data_output(
        pipeline,
        output_id.clone(),
        RegisterOutputOptions {
            output_options: RawDataOutputOptions {
                video: Some(RawVideoOptions { resolution }),
                audio: None,
            },
            video: Some(OutputVideoOptions {
                initial: scene,
                end_condition: PipelineOutputEndCondition::Never,
            }),
            audio: None,
        },
    )?;

    // Spawn thread to consume frames as fast as possible
    if let Some(video_receiver) = receiver.video {
        std::thread::Builder::new()
            .name("frame_consumer".to_string())
            .spawn(move || {
                let mut consecutive_errors = 0u64;

                // Simply receive and let frames drop immediately - no storage, no batching
                loop {
                    match video_receiver.recv() {
                        Ok(_frame) => {
                            consecutive_errors = 0;
                        }
                        Err(e) => {
                            consecutive_errors += 1;
                            info!("Frame consumer recv error #{}: {:?}", consecutive_errors, e);
                            if consecutive_errors > 10 {
                                info!("Too many consecutive errors, exiting consumer thread");
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    }
                }
            })
            .expect("Failed to spawn frame consumer thread");
    } else {
        info!("Warning: No video receiver available for raw output");
    }

    info!("Started raw output mode (running indefinitely)");

    Ok(output_id)
}
