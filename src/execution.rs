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

use process_wrap::tokio::{CommandWrap, KillOnDrop};

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
    ) -> Result<(tokio::process::Child, OwnedSemaphorePermit), ExecutionError> {
        let permit = self.try_acquire()?;
        let child = create_command(tool)
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
            let (stdout, stderr) = read_process_output(&mut child, budget).await?;
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
                stop_child(&mut child).await?;
                return Err(fout);
            }
            Err(_) => {
                stop_child(&mut child).await?;
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
// De functie read_capped leest de uitvoer van een asynchrone reader (zoals stdout of stderr van een proces) en beperkt de hoeveelheid gelezen bytes op basis van een opgegeven budget. Als de uitvoer boven het budget uitkomt, wordt een fout geretourneerd.
where
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
    child: &mut tokio::process::Child,
    budget: Arc<Semaphore>,
) -> Result<(CapturedOutput, CapturedOutput), ExecutionError> {
    // Neem de stdout- en stderr-pijpen van het proces en lees ze parallel met behulp van read_capped. Als een van de pijpen ontbreekt, retourneer een fout.
    let stdout = child.stdout.take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stdout-pipe ontbreekt"))
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        ExecutionError::OutputReadFailed(io::Error::other("stderr-pipe ontbreekt"))
    })?;

    // Gebruik tokio::try_join! om de uitvoer van stdout en stderr parallel te lezen met een gedeeld budget. Als een van de lezingen boven het budget uitkomt, wordt een fout geretourneerd.
    tokio::try_join!(
        read_capped(stdout, Arc::clone(&budget)),
        read_capped(stderr, budget)
    )
}

// Stopt een lopend proces door te controleren of het al is voltooid. Als het proces nog actief is, wordt het beëindigd. Eventuele fouten bij het wachten of beëindigen van het proces worden geretourneerd als ExecutionError.
async fn stop_child(child: &mut tokio::process::Child) -> Result<(), ExecutionError> {
    // Controleer of het proces al is voltooid door te proberen te wachten op de status. Als het proces nog actief is, wordt het beëindigd.
    let status = child
        .try_wait()
        .map_err(ExecutionError::ProcessWaitFailed)?;

    // Als het proces nog actief is, probeer het te beëindigen en retourneer eventuele fouten als ExecutionError.
    if status.is_none() {
        child
            .kill()
            .await
            .map_err(ExecutionError::ProcessKillFailed)?;
    }

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
mod tests {
    use tokio::io::AsyncWriteExt;

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

    #[cfg(any(windows, unix))]
    #[tokio::test]
    // Deze test is bedoeld om te controleren of de platformwrapper correct werkt. Het start een proces met de wrapper en wacht op een succesvolle afloop.
    async fn start_met_platformwrapper_en_wacht_op_succes() {
        let registry = test_registry(Vec::new());
        let tool = find_parameterless_tool(&registry, "git_versie")
            .expect("de ingestelde parameterloze tool moet bestaan");

        let mut command = create_wrapped_command(tool);
        let mut child = command
            .spawn()
            .expect("Het process met platformwrapper moet starten");

        let status = child
            .wait()
            .await
            .expect("op het proces met platformwrapper moet gewacht kunnen worden");

        assert!(status.success());
    }

    #[test]
    // Deze test controleert of de functie find_parameterless_tool correct een tool zonder parameters kan vinden.
    fn vindt_parameterloze_tool() {
        let registry = test_registry(Vec::new());

        let tool = find_parameterless_tool(&registry, "git_versie")
            .expect("de ingestelde tool zonder parameters moet worden gevonden");

        assert_eq!(tool.command, "git");
    }

    #[test]
    // Deze test controleert of de functie find_parameterless_tool correct een foutmelding geeft bij een onbekende tool.
    fn weigert_onbekende_tool() {
        let registry = test_registry(Vec::new());

        let resultaat = find_parameterless_tool(&registry, "onbekende_tool");

        assert!(matches!(resultaat, Err(ExecutionError::UnknownTool)));
    }

    #[test]
    // Deze test controleert of de functie find_parameterless_tool correct een foutmelding geeft bij een tool met parameters.
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
    // Deze test controleert of de ExecutionService correct een foutmelding geeft wanneer de maximale uitvoercapaciteit is bereikt.
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
    // Deze test controleert of de ExecutionService correct de capaciteit vrijgeeft wanneer het starten van een proces mislukt.
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
    // Deze test controleert of de ExecutionService correct een proces start en wacht op een succesvolle afloop.
    async fn start_proces_en_wacht_op_succesvolle_afloop() {
        let service = ExecutionService::new();
        let registry = test_registry(Vec::new());

        let resultaat = service
            .execute_parameterless(&registry, "git_versie")
            .await
            .expect("de git-testtool moet succesvol uitgevoerd worden");

        assert!(resultaat.success);
        assert_eq!(resultaat.exit_code, Some(0));
        assert!(!resultaat.stdout.is_empty());
        assert!(resultaat.stderr.is_empty());
    }

    #[tokio::test]
    // Deze test controleert of de ExecutionService correct een resultaat geeft bij een proces met een niet-succesvolle exitcode.
    async fn geeft_resultaat_bij_niet_succesvolle_exitcode() {
        let config = ConfigFile {
            tools: vec![ToolDefinition {
                name: "git_fout".to_owned(),
                description: "Test een niet-succesvolle exitcode".to_owned(),
                command: "git".to_owned(),
                arguments: vec!["--invalid-commandhub-test".to_owned()],
                parameters: Vec::new(),
                timeout: None,
            }],
        };
        let registry = ToolRegistry::new(config);
        let service = ExecutionService::new();

        let resultaat = service
            .execute_parameterless(&registry, "git_fout")
            .await
            .expect("een proces met een foutieve exitcode moet een resultaat geven");

        assert!(!resultaat.success);
        assert!(matches!(resultaat.exit_code, Some(code) if code != 0));
        assert!(!resultaat.stderr.is_empty());
    }

    #[cfg(any(windows, unix))]
    #[tokio::test]
    // Deze test controleert of de ExecutionService correct een timeout geeft bij een proces dat te lang duurt.
    async fn geeft_timeout_als_proces_te_lang_duurt() {
        #[cfg(windows)]
        let (command, arguments) = (
            "ping",
            vec!["-n".to_owned(), "5".to_owned(), "127.0.0.1".to_owned()],
        );

        #[cfg(unix)]
        let (command, arguments) = ("sleep", vec!["5".to_owned()]);

        let config = ConfigFile {
            tools: vec![ToolDefinition {
                name: "langzaam".to_owned(),
                description: "Test de proces-timeout".to_owned(),
                command: command.to_owned(),
                arguments,
                parameters: Vec::new(),
                timeout: Some(100),
            }],
        };
        let registry = ToolRegistry::new(config);
        let service = ExecutionService::new();

        let resultaat = service.execute_parameterless(&registry, "langzaam").await;

        assert!(
            matches!(resultaat, Err(ExecutionError::TimedOut)),
            "verwachtte een timeout, kreeg: {resultaat:?}"
        );

        let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
            .map(|_| {
                service
                    .try_acquire()
                    .expect("capaciteit moet na een timeout weer vrij zijn")
            })
            .collect();
    }

    #[tokio::test]
    // Deze test controleert of de ExecutionService correct een foutmelding geeft bij een proces dat te veel uitvoer genereert.
    async fn stopt_proces_bij_outputoverschrijding() {
        let testprogramma = std::env::current_exe()
            .expect("het pad naar het huidige testprogramma moet beschikbaar zijn");

        let config = ConfigFile {
            tools: vec![ToolDefinition {
                name: "grote_output".to_owned(),
                description: "Test het stoppen bij te grote uitvoer".to_owned(),
                command: testprogramma.to_string_lossy().into_owned(),
                arguments: vec![
                    "--exact".to_owned(),
                    "execution::tests::schrijft_output_boven_de_limiet".to_owned(),
                    "--ignored".to_owned(),
                    "--nocapture".to_owned(),
                ],
                parameters: Vec::new(),
                timeout: Some(10_000),
            }],
        };
        let registry = ToolRegistry::new(config);
        let service = ExecutionService::new();

        let resultaat = service
            .execute_parameterless(&registry, "grote_output")
            .await;

        assert!(
            matches!(resultaat, Err(ExecutionError::OutputLimitExceeded)),
            "verwachtte een outputlimietfout, kreeg: {resultaat:?}"
        );

        let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
            .map(|_| {
                service
                    .try_acquire()
                    .expect("capaciteit moet na outputoverschrijding vrij zijn")
            })
            .collect();
    }

    #[tokio::test]
    // Deze test controleert of de read_capped functie correct uitvoer onder de limiet kan lezen.
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
    // Deze test controleert of de read_capped functie correct een foutmelding geeft wanneer de gecombineerde uitvoer van stdout en stderr boven de limiet uitkomt.
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
    // Deze test controleert of de read_capped functie correct een foutmelding geeft wanneer de uitvoer boven de limiet uitkomt.
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

    #[test]
    #[ignore]
    // Deze test is bedoeld om te controleren of de ExecutionService correct een foutmelding geeft bij een proces dat te veel uitvoer genereert. Het wordt genegeerd omdat het een groot aantal bytes schrijft en daardoor de testuitvoer kan vervuilen.
    fn schrijft_output_boven_de_limiet() {
        use std::io::Write;

        let bytes = vec![b'x'; MAX_OUTPUT_BYTES + 1];
        std::io::stdout()
            .lock()
            .write_all(&bytes)
            .expect("de testuitvoer moet geschreven worden");
    }
}
