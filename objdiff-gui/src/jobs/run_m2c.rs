use std::{process::Command, str::from_utf8, sync::mpsc::Receiver};

use anyhow::{anyhow, Context, Result};

use crate::{
    app::AppConfig,
    jobs::{objdiff::BuildConfig, start_job, update_status, Job, JobContext, JobResult, JobState},
};

#[derive(Debug, Clone)]
pub struct RunM2CConfig {
    pub build_config: BuildConfig,
    pub function_name: String,
}

impl RunM2CConfig {
    pub(crate) fn from_config(config: &AppConfig, function_name: String) -> Result<Self> {
        Ok(Self { build_config: BuildConfig::from_config(config), function_name })
    }

    pub fn is_available(config: &AppConfig) -> bool {
        let Some(selected_obj) = &config.selected_obj else {
            return false;
        };
        selected_obj.target_path.is_some()
    }
}

#[derive(Default, Debug, Clone)]
pub struct RunM2CResult {}

fn run_m2c(
    status: &JobContext,
    cancel: Receiver<()>,
    config: RunM2CConfig,
) -> Result<Box<RunM2CResult>> {
    let project_dir =
        config.build_config.project_dir.as_ref().ok_or_else(|| anyhow!("Missing project dir"))?;

    update_status(status, "Running M2C".to_string(), 1, 2, &cancel)?;

    use std::os::windows::process::CommandExt;

    let mut command = Command::new("python");
    command
        .current_dir(project_dir)
        .arg("tools/decomp.py")
        .arg(config.function_name)
        .arg("--valid-syntax")
        .arg("--no-casts")
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);
    let output = command.output().context("Failed to execute m2c")?;
    let stdout = from_utf8(&output.stdout).context("Failed to process stdout")?;
    let stderr = from_utf8(&output.stderr).context("Failed to process stderr")?;
    log::info!("{stdout}");
    log::info!("{stderr}");
    update_status(status, "Complete".to_string(), 2, 2, &cancel)?;
    Ok(Box::from(RunM2CResult {}))
}

pub fn start_run_m2c(ctx: &egui::Context, config: RunM2CConfig) -> JobState {
    start_job(ctx, "Run M2C", Job::RunM2C, move |context, cancel| {
        run_m2c(&context, cancel, config).map(|result| JobResult::RunM2C(Some(result)))
    })
}
