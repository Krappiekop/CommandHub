use tokio::io::AsyncWriteExt;

use super::*;
use crate::config::{ ConfigFile, ParameterDefinition, ParameterType };

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
    let tool = find_parameterless_tool(&registry, "git_versie").expect(
        "de ingestelde parameterloze tool moet bestaan"
    );

    let mut command = create_wrapped_command(tool);
    let mut child = command.spawn().expect("Het process met platformwrapper moet starten");

    let status = child
        .wait().await
        .expect("op het proces met platformwrapper moet gewacht kunnen worden");

    assert!(status.success());
}

#[test]
// Deze test controleert of de functie find_parameterless_tool correct een tool zonder parameters kan vinden.
fn vindt_parameterloze_tool() {
    let registry = test_registry(Vec::new());

    let tool = find_parameterless_tool(&registry, "git_versie").expect(
        "de ingestelde tool zonder parameters moet worden gevonden"
    );

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
    let registry = test_registry(
        vec![ParameterDefinition {
            name: "pad".to_owned(),
            kind: ParameterType::String,
            required: true,
        }]
    );

    let resultaat = find_parameterless_tool(&registry, "git_versie");

    assert!(matches!(resultaat, Err(ExecutionError::ParametersNotSupported)));
}

#[test]
// Deze test controleert of de ExecutionService correct een foutmelding geeft wanneer de maximale uitvoercapaciteit is bereikt.
fn weigert_extra_uitvoering_bij_volledige_capaciteit() {
    let service = ExecutionService::new();
    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| { service.try_acquire().expect("capaciteit moet beschikbaar zijn") })
        .collect();

    assert!(matches!(service.try_acquire(), Err(ExecutionError::CapacityExceeded)));
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

    assert!(matches!(service.start_process(&tool), Err(ExecutionError::ProcessStartFailed(_))));

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| { service.try_acquire().expect("capaciteit moet na de fout vrij zijn") })
        .collect();
}

#[tokio::test]
// Deze test controleert of de ExecutionService correct een proces start en wacht op een succesvolle afloop.
async fn start_proces_en_wacht_op_succesvolle_afloop() {
    let service = ExecutionService::new();
    let registry = test_registry(Vec::new());

    let resultaat = service
        .execute_parameterless(&registry, "git_versie").await
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
        .execute_parameterless(&registry, "git_fout").await
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
        .map(|_| { service.try_acquire().expect("capaciteit moet na een timeout weer vrij zijn") })
        .collect();
}

#[tokio::test]
// Deze test controleert of de ExecutionService correct een foutmelding geeft bij een proces dat te veel uitvoer genereert.
async fn stopt_proces_bij_outputoverschrijding() {
    let testprogramma = std::env
        ::current_exe()
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
                "--nocapture".to_owned()
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    };
    let registry = ToolRegistry::new(config);
    let service = ExecutionService::new();

    let resultaat = service.execute_parameterless(&registry, "grote_output").await;

    assert!(
        matches!(resultaat, Err(ExecutionError::OutputLimitExceeded)),
        "verwachtte een outputlimietfout, kreeg: {resultaat:?}"
    );

    let _permits: Vec<_> = (0..MAX_CONCURRENT_EXECUTIONS)
        .map(|_| {
            service.try_acquire().expect("capaciteit moet na outputoverschrijding vrij zijn")
        })
        .collect();
}

#[tokio::test]
// Deze test controleert of de read_capped functie correct uitvoer onder de limiet kan lezen.
async fn leest_output_binnen_de_limiet() {
    let (mut schrijver, lezer) = tokio::io::duplex(16);
    schrijver.write_all(b"uitvoer").await.expect("testuitvoer moet geschreven worden");
    drop(schrijver);

    let budget = Arc::new(Semaphore::new(MAX_OUTPUT_BYTES));
    let uitvoer = read_capped(lezer, budget).await.expect(
        "uitvoer onder de limiet moet worden gelezen"
    );

    assert_eq!(uitvoer.bytes, b"uitvoer");
}

#[tokio::test]
// Deze test controleert of de read_capped functie correct een foutmelding geeft wanneer de gecombineerde uitvoer van stdout en stderr boven de limiet uitkomt.
async fn begrenst_stdout_en_stderr_met_een_gedeeld_budget() {
    let (mut stdout_writer, stdout_reader) = tokio::io::duplex(16);
    let (mut stderr_writer, stderr_reader) = tokio::io::duplex(16);

    stdout_writer.write_all(b"123").await.expect("stdout-testdata moet geschreven worden");
    stderr_writer.write_all(b"456").await.expect("stderr-testdata moet geschreven worden");
    drop(stdout_writer);
    drop(stderr_writer);

    let budget = Arc::new(Semaphore::new(5));
    let resultaat = tokio::try_join!(
        read_capped(stdout_reader, Arc::clone(&budget)),
        read_capped(stderr_reader, budget)
    );

    assert!(matches!(resultaat, Err(ExecutionError::OutputLimitExceeded)));
}

#[tokio::test]
// Deze test controleert of de read_capped functie correct een foutmelding geeft wanneer de uitvoer boven de limiet uitkomt.
async fn weigert_output_boven_de_limiet() {
    let (mut schrijver, lezer) = tokio::io::duplex(16);
    schrijver.write_all(b"12345").await.expect("testuitvoer moet geschreven worden");
    drop(schrijver);

    let budget = Arc::new(Semaphore::new(4));
    let resultaat = read_capped(lezer, budget).await;

    assert!(matches!(resultaat, Err(ExecutionError::OutputLimitExceeded)));
}

#[test]
#[ignore]
// Deze test is bedoeld om te controleren of de ExecutionService correct een foutmelding geeft bij een proces dat te veel uitvoer genereert. Het wordt genegeerd omdat het een groot aantal bytes schrijft en daardoor de testuitvoer kan vervuilen.
fn schrijft_output_boven_de_limiet() {
    use std::io::Write;

    let bytes = vec![b'x'; MAX_OUTPUT_BYTES + 1];
    std::io::stdout().lock().write_all(&bytes).expect("de testuitvoer moet geschreven worden");
}
