use std::{io, process::Stdio, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    sync::{OwnedSemaphorePermit, Semaphore},
};

use crate::{config::ToolDefinition, registry::ToolRegistry};

#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration: Duration,
}

#[derive(Debug)]
pub enum ExecutionError {
    UnknownTool,
    ParametersNotSupported,
    CapacityExceeded,
    TimedOut,
    OutputLimitExceeded,
    Cancelled,
    ProcessStartFailed(io::Error),
    OutputReadFailed(io::Error),
    ProcessWaitFailed(io::Error),
}

const MAX_CONCURRENT_EXECUTIONS: usize = 4;

const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

struct CapturedOutput {
    bytes: Vec<u8>,
    _budget_permits: Vec<OwnedSemaphorePermit>,
}

pub struct ExecutionService {
    permits: Arc<Semaphore>,
}

impl ExecutionService {
    pub fn new() -> Self {
        Self {
            permits: Arc::new(Semaphore::new(MAX_CONCURRENT_EXECUTIONS)),
        }
    }

    fn try_acquire(&self) -> Result<OwnedSemaphorePermit, ExecutionError> {
        Arc::clone(&self.permits)
            .try_acquire_owned()
            .map_err(|_| ExecutionError::CapacityExceeded)
    }

    fn start_process(
        &self,
        tool: &ToolDefinition,
    ) -> Result<(tokio::process::Child, OwnedSemaphorePermit), ExecutionError> {
        let permit = self.try_acquire()?;
        let child = create_command(tool)
            .spawn()
            .map_err(ExecutionError::ProcessStartFailed)?;

        Ok((child, permit))
    }
}

pub fn find_parameterless_tool<'a>(
    registry: &'a ToolRegistry,
    name: &str,
) -> Result<&'a ToolDefinition, ExecutionError> {
    let tool = registry
        .tools()
        .iter()
        .find(|tool| tool.name == name)
        .ok_or(ExecutionError::UnknownTool)?;

    let has_parameters = !tool.parameters.is_empty();
    let has_placeholders = tool
        .arguments
        .iter()
        .any(|argument| argument.contains('{') || argument.contains('}'));

    if has_parameters || has_placeholders {
        return Err(ExecutionError::ParametersNotSupported);
    }

    Ok(tool)
}

fn create_command(tool: &ToolDefinition) -> Command {
    let mut command = Command::new(tool.command.as_str());

    command
        .args(tool.arguments.iter().map(String::as_str))
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    command
}

async fn read_capped<R>(
    mut reader: R,
    budget: Arc<Semaphore>,
) -> Result<CapturedOutput, ExecutionError>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut bytes = Vec::new();
    let mut budget_permits = Vec::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .map_err(ExecutionError::OutputReadFailed)?;

        if bytes_read == 0 {
            break;
        }

        let permit = Arc::clone(&budget)
            .try_acquire_many_owned(bytes_read as u32)
            .map_err(|_| ExecutionError::OutputLimitExceeded)?;

        bytes.extend_from_slice(&buffer[..bytes_read]);
        budget_permits.push(permit);
    }

    Ok(CapturedOutput {
        bytes,
        _budget_permits: budget_permits,
    })
}

async fn read_process_output(
    child: &mut tokio::process::Child,
    budget: Arc<Semaphore>,
) -> Result<(CapturedOutput, CapturedOutput), ExecutionError> {
    let stdout = child.stdout.take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stdout-pipe ontbreekt"))
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stderr-pipe ontbreekt"))
    })?;

    tokio::try_join!(
        read_capped(stdout, Arc::clone(&budget)),
        read_capped(stderr, budget)
    )
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTool => write!(formatter, "Onbekende tool"),
            Self::ParametersNotSupported => {
                write!(formatter, "Deze tool ondersteunt nog geen parameters")
            }
            Self::CapacityExceeded => write!(formatter, "Uitvoercapaciteit bereikt"),
            Self::TimedOut => write!(formatter, "Uitvoering verlopen"),
            Self::OutputLimitExceeded => write!(formatter, "Uitvoerlimiet overschreden"),
            Self::Cancelled => write!(formatter, "Uitvoering geannuleerd"),
            Self::ProcessStartFailed(_) => write!(formatter, "Proces starten mislukt"),
            Self::OutputReadFailed(_) => write!(formatter, "Procesuitvoer lezen mislukt"),
            Self::ProcessWaitFailed(_) => write!(formatter, "Wachten op proces mislukt"),
        }
    }
}

impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProcessStartFailed(error)
            | Self::OutputReadFailed(error)
            | Self::ProcessWaitFailed(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConfigFile, ParameterDefinition, ParameterType};

    fn test_registry(parameters: Vec<ParameterDefinition>) -> ToolRegistry {
        let config = ConfigFile {
            tools: vec![ToolDefinition {
                name: "git_versie".to_owned(),
                description: "Toon de geinstalleerde Git-versie".to_owned(),
                command: "git".to_owned(),
                arguments: vec!["--version".to_owned()],
                parameters,
                timeout: None,
            }],
        };

        ToolRegistry::new(config)
    }

    #[test]
    fn vindt_parameterloze_tool() {
        let registry = test_registry(Vec::new());

        let tool = find_parameterless_tool(&registry, "git_versie")
            .expect("de ingestelde tool zonder parameters moet worden gevonden");

        assert_eq!(tool.command, "git");
    }

    #[test]
    fn weigert_onbekende_tool() {
        let registry = test_registry(Vec::new());

        let resultaat = find_parameterless_tool(&registry, "onbekende_tool");

        assert!(matches!(resultaat, Err(ExecutionError::UnknownTool)));
    }

    #[test]
    fn weigert_tool_met_parameters() {
        let registry = test_registry(vec![ParameterDefinition {
            name: "pad".to_owned(),
            kind: ParameterType::String,
            required: true,
        }]);

        let resultaat = find_parameterless_tool(&registry, "git_versie");

        assert!(matches!(
            resultaat,
            Err(ExecutionError::ParametersNotSupported)
        ));
    }

    #[test]
    fn weigert_extra_uitvoering_bij_volledige_capaciteit() {
        let service = ExecutionService::new();
        let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
            .map(|_| {
                service
                    .try_acquire()
                    .expect("capaciteit moet beschikbaar zijn")
            })
            .collect();

        assert!(matches!(
            service.try_acquire(),
            Err(ExecutionError::CapacityExceeded)
        ));
    }

    #[test]
    fn geeft_capaciteit_vrij_als_starten_mislukt() {
        let service = ExecutionService::new();
        let tool = ToolDefinition {
            name: "ongeldig".to_owned(),
            description: "Test een mislukte processtart".to_owned(),
            command: "\0".to_owned(),
            arguments: Vec::new(),
            parameters: Vec::new(),
            timeout: None,
        };

        assert!(matches!(
            service.start_process(&tool),
            Err(ExecutionError::ProcessStartFailed(_))
        ));

        let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
            .map(|_| {
                service
                    .try_acquire()
                    .expect("capaciteit moet na de fout vrij zijn")
            })
            .collect();
    }

    #[tokio::test]
    async fn start_proces_en_wacht_op_succesvolle_afloop() {
        let service = ExecutionService::new();
        let registry = test_registry(Vec::new());
        let tool = find_parameterless_tool(&registry, "git_versie")
            .expect("de git-testtool moet gevonden worden");

        let (mut child, _permit) = service
            .start_process(tool)
            .expect("git moet gestart kunnen worden");

        let budget = Arc::new(Semaphore::new(MAX_OUTPUT_BYTES));
        let (stdout_output, _stderr_output) =
            read_process_output(&mut child, budget)
                .await
                .expect("stdout en stderr moeten binnen het gedeelde budget gelezen worden");

        assert!(!stdout_output.bytes.is_empty());

        let status = child
            .wait()
            .await
            .expect("er moet op het proces gewacht kunnen worden");

        assert!(status.success());
    }

    #[tokio::test]
    async fn leest_output_binnen_de_limiet() {
        let (mut schrijver, lezer) = tokio::io::duplex(16);
        schrijver
            .write_all(b"uitvoer")
            .await
            .expect("testuitvoer moet geschreven worden");
        drop(schrijver);

        let budget = Arc::new(Semaphore::new(MAX_OUTPUT_BYTES));
        let uitvoer = read_capped(lezer, budget)
            .await
            .expect("uitvoer onder de limiet moet worden gelezen");

        assert_eq!(uitvoer.bytes, b"uitvoer");
    }

    #[tokio::test]
    async fn begrenst_stdout_en_stderr_met_een_gedeeld_budget() {
        let (mut stdout_writer, stdout_reader) = tokio::io::duplex(16);
        let (mut stderr_writer, stderr_reader) = tokio::io::duplex(16);

        stdout_writer
            .write_all(b"123")
            .await
            .expect("stdout-testdata moet geschreven worden");
        stderr_writer
            .write_all(b"456")
            .await
            .expect("stderr-testdata moet geschreven worden");
        drop(stdout_writer);
        drop(stderr_writer);

        let budget = Arc::new(Semaphore::new(5));
        let resultaat = tokio::try_join!(
            read_capped(stdout_reader, Arc::clone(&budget)),
            read_capped(stderr_reader, budget)
        );

        assert!(matches!(
            resultaat,
            Err(ExecutionError::OutputLimitExceeded)
        ));
    }

    #[tokio::test]
    async fn weigert_output_boven_de_limiet() {
        let (mut schrijver, lezer) = tokio::io::duplex(16);
        schrijver
            .write_all(b"12345")
            .await
            .expect("testuitvoer moet geschreven worden");
        drop(schrijver);

        let budget = Arc::new(Semaphore::new(4));
        let resultaat = read_capped(lezer, budget).await;

        assert!(matches!(
            resultaat,
            Err(ExecutionError::OutputLimitExceeded)
        ));
    }
}
