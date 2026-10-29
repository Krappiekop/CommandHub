use std::{
    io,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{OwnedSemaphorePermit, Semaphore},
};

use crate::{config::ToolDefinition, registry::ToolRegistry};

use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};

#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;

#[cfg(windows)]
use process_wrap::tokio::JobObject;

#[derive(Debug)]
// Representeert het resultaat van een uitgevoerde tool, inclusief successtatus, uitvoer, foutuitvoer, exitcode en duur van de uitvoering.
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration: Duration,
}

#[derive(Debug)]
// Representeert mogelijke fouten die kunnen optreden tijdens de uitvoering van een tool, zoals onbekende tools, overschrijding van capaciteit of timeouts.
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
    ProcessKillFailed(io::Error),
}

// Definieert de maximale gelijktijdige uitvoeringen en de maximale uitvoerbytes voor de ExecutionService.
const MAX_CONCURRENT_EXECUTIONS: usize = 4;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

// Structuur die de uitvoer van een proces vastlegt, inclusief de gelezen bytes en de bijbehorende budgetpermissies.
struct CapturedOutput {
    bytes: Vec<u8>,
    _budget_permits: Vec<OwnedSemaphorePermit>,
}

// Service die verantwoordelijk is voor het uitvoeren van tools, het beheren van gelijktijdige uitvoeringen en het afhandelen van uitvoerbeperkingen.
pub struct ExecutionService {
    permits: Arc<Semaphore>,
}

// Implementeert de ExecutionService, inclusief methoden voor het starten van processen, het lezen van uitvoer en het afhandelen van fouten.
impl ExecutionService {
    // Creëert een nieuwe instantie van de ExecutionService met een gelimiteerd aantal gelijktijdige uitvoeringen.
    pub fn new() -> Self {
        Self {
            permits: Arc::new(Semaphore::new(MAX_CONCURRENT_EXECUTIONS)),
        }
    }

    // Probeert een permit te verkrijgen voor het starten van een nieuwe uitvoering. Als de capaciteit is bereikt, wordt een fout geretourneerd.
    fn try_acquire(&self) -> Result<OwnedSemaphorePermit, ExecutionError> {
        Arc::clone(&self.permits)
            .try_acquire_owned()
            .map_err(|_| ExecutionError::CapacityExceeded)
    }

    // Start een proces op basis van de opgegeven tooldefinitie en verkrijgt een permit voor gelijktijdige uitvoering. Als het starten van het proces mislukt, wordt een fout geretourneerd.
    fn start_process(
        &self,
        tool: &ToolDefinition,
    ) -> Result<(Box<dyn ChildWrapper>, OwnedSemaphorePermit), ExecutionError> {
        let permit = self.try_acquire()?;
        let child = create_wrapped_command(tool)
            .spawn()
            .map_err(ExecutionError::ProcessStartFailed)?;

        Ok((child, permit))
    }

    // Voert een parameterloze tool uit op basis van de opgegeven naam en het geregistreerde tools. Het leest de uitvoer, wacht op voltooiing en retourneert het resultaat of een fout.
    pub(crate) async fn execute_parameterless(
        &self,
        registry: &ToolRegistry,
        name: &str,
    ) -> Result<ExecutionResult, ExecutionError> {
        // Zoek de tooldefinitie op basis van de naam en controleer of deze geen parameters heeft.
        let tool = find_parameterless_tool(registry, name)?;
        let started_at = Instant::now();
        let (mut child, _permit) = self.start_process(tool)?;

        // Bepaal de timeout voor de uitvoering, standaard 10 seconden als geen specifieke timeout is opgegeven.
        let timeout = Duration::from_millis(tool.timeout.unwrap_or(10_000));
        let budget = Arc::new(Semaphore::new(MAX_OUTPUT_BYTES));

        // Start een asynchrone taak om de uitvoer van het proces te lezen en te wachten op voltooiing.
        let uitvoering = async {
            let (stdout, stderr) = read_process_output(child.as_mut(), budget).await?;
            let status = child
                .wait()
                .await
                .map_err(ExecutionError::ProcessWaitFailed)?;

            Ok::<_, ExecutionError>((stdout, stderr, status))
        };

        // Wacht op de uitvoering met een timeout. Als de uitvoering succesvol is, retourneer het resultaat. Als er een fout optreedt of de timeout wordt overschreden, stop het proces en retourneer de fout.
        let (stdout, stderr, status) = match tokio::time::timeout(timeout, uitvoering).await {
            Ok(Ok(resultaat)) => resultaat,
            Ok(Err(fout)) => {
                stop_child(child.as_mut()).await?;
                return Err(fout);
            }
            Err(_) => {
                stop_child(child.as_mut()).await?;
                return Err(ExecutionError::TimedOut);
            }
        };

        // Bouw het ExecutionResult op basis van de verkregen uitvoer, status en duur van de uitvoering.
        Ok(ExecutionResult {
            success: status.success(),
            stdout: String::from_utf8_lossy(&stdout.bytes).into_owned(),
            stderr: String::from_utf8_lossy(&stderr.bytes).into_owned(),
            exit_code: status.code(),
            duration: started_at.elapsed(),
        })
    }
}

// Zoekt een parameterloze tool op basis van de naam in het opgegeven ToolRegistry. Als de tool niet bestaat of parameters heeft, wordt een fout geretourneerd.
pub fn find_parameterless_tool<'a>(
    registry: &'a ToolRegistry,
    name: &str,
) -> Result<&'a ToolDefinition, ExecutionError> {
    // Zoek de tooldefinitie op basis van de naam in het ToolRegistry.
    let tool = registry
        .tools()
        .iter()
        .find(|tool| tool.name == name)
        .ok_or(ExecutionError::UnknownTool)?;

    // Controleer of de tool parameters heeft of placeholders in de argumenten bevat. Als dat het geval is, retourneer een fout.
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

// Creëert een Command-object op basis van de opgegeven ToolDefinition, inclusief het instellen van de commandoregel, argumenten en standaardinvoer/-uitvoer/-fout.
fn create_command(tool: &ToolDefinition) -> Command {
    let mut command = Command::new(tool.command.as_str());

    // Voeg de argumenten van de tool toe aan het
    command
        .args(tool.arguments.iter().map(String::as_str))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    command
}

// Creëert een CommandWrap-object dat het proces op een platformonafhankelijke manier kan beheren, inclusief het instellen van procesgroepen of jobobjecten en het automatisch beëindigen van het proces bij het loslaten van de wrapper.
fn create_wrapped_command(tool: &ToolDefinition) -> CommandWrap {
    let mut command = CommandWrap::from(create_command(tool));

    // Voeg platformafhankelijke wrappers toe om het proces te beheren en ervoor te zorgen dat het correct wordt beëindigd bij het loslaten van de wrapper.
    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());

    #[cfg(windows)]
    command.wrap(JobObject);

    command.wrap(KillOnDrop);
    command
}

// Leest de uitvoer van een asynchrone reader met een opgegeven budget voor het aantal bytes. Als de uitvoer boven het budget uitkomt, wordt een fout geretourneerd.
async fn read_capped<R>(
    mut reader: R,
    budget: Arc<Semaphore>,
) -> Result<CapturedOutput, ExecutionError>
where
    // De functie read_capped leest de uitvoer van een asynchrone reader (zoals stdout of stderr van een proces) en beperkt de hoeveelheid gelezen bytes op basis van een opgegeven budget. Als de uitvoer boven het budget uitkomt, wordt een fout geretourneerd.
    R: tokio::io::AsyncRead + Unpin,
{
    // Initialiseer een buffer om de gelezen bytes op te slaan en een vector om de verkregen budgetpermissies bij te houden.
    let mut bytes = Vec::new();
    let mut budget_permits = Vec::new();
    let mut buffer = [0_u8; 8192];

    // Lees de uitvoer in een lus totdat er geen bytes meer beschikbaar zijn of totdat het budget is overschreden.
    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .map_err(ExecutionError::OutputReadFailed)?;

        if bytes_read == 0 {
            break;
        }

        // Probeer het aantal gelezen bytes te verkrijgen van het budget. Als het budget is overschreden, retourneer een fout.
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

// Leest de uitvoer van een proces (zowel stdout als stderr) met een gedeeld budget voor het aantal bytes. Als de gecombineerde uitvoer boven het budget uitkomt, wordt een fout geretourneerd.
async fn read_process_output(
    child: &mut dyn ChildWrapper,
    budget: Arc<Semaphore>,
) -> Result<(CapturedOutput, CapturedOutput), ExecutionError> {
    let stdout = child.stdout().take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stdout-pipe ontbreekt"))
    })?;
    let stderr = child.stderr().take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stderr-pipe ontbreekt"))
    })?;

    tokio::try_join!(
        read_capped(stdout, Arc::clone(&budget)),
        read_capped(stderr, budget)
    )
}

// Stopt een lopend proces door te controleren of het al is voltooid. Als het proces nog actief is, wordt het beëindigd. Eventuele fouten bij het wachten of beëindigen van het proces worden geretourneerd als ExecutionError.
async fn stop_child(child: &mut dyn ChildWrapper) -> Result<(), ExecutionError> {
    let status = child
        .try_wait()
        .map_err(ExecutionError::ProcessWaitFailed)?;

    let kill_future = child.kill();
    std::pin::Pin::from(kill_future)
        .await
        .map_err(ExecutionError::ProcessKillFailed)?;

    Ok(())
}

// Implementeert de Display-trait voor ExecutionError om een leesbare foutmelding te genereren op basis van het type fout.
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
            Self::ProcessKillFailed(_) => write!(formatter, "Proces beëindigen mislukt"),
        }
    }
}

// Implementeert de Error-trait voor ExecutionError om de bron van de fout te kunnen achterhalen, indien beschikbaar.
impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ProcessStartFailed(error)
            | Self::OutputReadFailed(error)
            | Self::ProcessWaitFailed(error)
            | Self::ProcessKillFailed(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
