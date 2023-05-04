use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use crate::service::{Service, ServiceScript};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
}

#[derive(Debug)]
pub struct ServiceWorker {
    service: Service,
    start_process: Option<Child>,
    stop_process: Option<Child>,

    status: ServiceStatus,
    status_last_change: Instant,
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceWorkerError {
    #[error("Service is already running")]
    AlreadyRunning,

    #[error("Service is not running")]
    NotRunning,

    #[error("Service is busy")]
    Busy,

    #[error("Script {0} not specified")]
    ScriptNotSpecified(String),

    #[error("IO error {0}")]
    IoError(#[from] std::io::Error),
}

impl ServiceWorker {
    pub fn new(service: Service) -> Self {
        Self {
            service,
            start_process: None,
            stop_process: None,

            status: ServiceStatus::Stopped,
            status_last_change: Instant::now(),
        }
    }

    pub async fn start(&mut self) -> Result<(), ServiceWorkerError> {
        // State check
        match self.status {
            ServiceStatus::Stopped => Ok(()),
            ServiceStatus::Starting => Err(ServiceWorkerError::Busy),
            ServiceStatus::Running => Err(ServiceWorkerError::AlreadyRunning),
            ServiceStatus::Stopping => Err(ServiceWorkerError::Busy)
        }?;

        // Just in case, we don't want any zombie processes hanging around
        if self.start_process.is_some() {
            return Err(ServiceWorkerError::AlreadyRunning);
        }

        // Start the service
        match &self.service.scripts.start {
            None => Err(ServiceWorkerError::ScriptNotSpecified("start".to_string())),

            Some(ServiceScript::Exec { command, arguments }) => {
                let child = Command::new(command)
                    .args(arguments)
                    .spawn()?;

                self.start_process = Some(child);

                Ok(())
            }

            #[allow(unreachable_patterns)]
            _ => todo!() // Kept around just in case we add more script types
        }?;

        // Update status
        self.status = ServiceStatus::Starting;

        Ok(())
    }

    pub async fn stop(&mut self, timeout: Duration, force: bool) -> Result<(), ServiceWorkerError> {
        // State check
        match self.status {
            ServiceStatus::Stopped => Err(ServiceWorkerError::NotRunning),
            ServiceStatus::Starting => Err(ServiceWorkerError::Busy),
            ServiceStatus::Running => Ok(()),
            ServiceStatus::Stopping => Err(ServiceWorkerError::Busy)
        }?;

        // Stop the service
        match &self.service.scripts.stop {
            None => {
                let begin = Instant::now();

                while self.start_process.is_some() {
                    if begin.elapsed() > timeout {
                        if force {
                            self.start_process.as_mut().take().unwrap().kill().await?;
                            self.stop_process.as_mut().take().unwrap().kill().await?;
                        } else {
                            // Timeout reached, but we're not forcing
                            self.stop_process.as_mut().take().unwrap().kill().await?;
                            return Err(ServiceWorkerError::Busy);
                        }
                    }

                    // Wait 100ms before checking again so we don't fry the CPU
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },

            Some(ServiceScript::Exec { command, arguments }) => {
                let child = Command::new(command)
                    .args(arguments)
                    .spawn()?;

                self.stop_process = Some(child);
            }

            #[allow(unreachable_patterns)]
            _ => todo!() // Kept around just in case we add more script types
        };

        Ok(())
    }
}
