---
feature: F03
tags:
  - commandhub
  - feature-notitie
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

[Featureplan](../docs/features/F03-procesuitvoering.md)

## Wat bouw ik en waarom?

F03 voegt een gedeelde, begrensde procesuitvoerder en `POST /tools/{toolname}` toe voor geconfigureerde tools zonder dynamische parameters. `ExecutionService::execute_parameterless` start een ingestelde tool, leest stdout en stderr tegelijk binnen één budget, wacht op het proces en levert een `ExecutionResult`. `process-wrap` 10.0.1 is geïntegreerd: Unix gebruikt `ProcessGroup`, Windows `JobObject` en beide gebruiken `KillOnDrop`. De gebruiker meldt dat de wrapper-smoketest en de gerichte executietests slagen. De HTTP-route ontbreekt nog; opruiming van een volledige boom met subprocessen en annulering zijn nog niet bewezen. De actuele checklist staat in het [featureplan](../docs/features/F03-procesuitvoering.md).

## Hoe werkt de code?

In `src/execution.rs` beschrijft `ExecutionResult` een afgeronde run en onderscheidt `ExecutionError` de uitvoeringsfouten. `find_parameterless_tool` zoekt in de registry en weigert tools met parameters of placeholders. De lifetime `'a` geeft aan dat de gevonden `ToolDefinition` geleend wordt uit de registry.

`ExecutionService` houdt met `Arc<Semaphore>` een gedeelde capaciteit bij. `try_acquire_owned` wijst direct af wanneer de vier permits bezet zijn; een `OwnedSemaphorePermit` geeft de plek automatisch terug wanneer hij wordt gedropt. `create_command` bouwt een Tokio-`Command` met executable en argumenten apart en sluit stdin.

`read_capped` leest een async stream in blokken van 8192 bytes. Een gedeelde semaphore bewaakt het gezamenlijke outputbudget; de permits blijven bij `CapturedOutput` bewaard zolang de bytes in het geheugen staan. `read_process_output` haalt de stdout- en stderr-pipes uit het childproces en gebruikt `tokio::try_join!` om ze tegelijk met hetzelfde budget te lezen.

`ExecutionService::start_process` reserveert een capaciteit-permit en start het proces met `create_wrapped_command`. De wrapper geeft `Box<dyn ChildWrapper>` terug; via die trait leest de executor de pipes en wacht of beëindigt hij het proces. Als starten mislukt, wordt de permit automatisch vrijgegeven. Bij timeout of outputleesfout roept `stop_child` `kill()` van de wrapper aan. De platformwrappers zijn aangesloten, maar gedrag met een child die zelf subprocessen start moet nog op Windows en Linux worden getest.

## Keuzes en alternatieven

- F03 voert uitsluitend tools zonder dynamische parameters uit; placeholdervervanging hoort bij F04.
- Native executables zijn toegestaan. Scripts starten alleen via een expliciet geconfigureerde interpreter met vaste argumenten; geen automatische shell- of `.bat`/`.cmd`-dispatch.
- Het childproces erft de omgeving en werkdirectory van de server. De client kan deze niet instellen.
- Limieten: 10.000 ms standaardtimeout, maximaal 60.000 ms timeout, 1 MiB gecombineerde stdout/stderr, 16 KiB requestbody en vier gelijktijdige uitvoeringen. Volle capaciteit wordt direct afgewezen.
- Timeout, outputoverschrijding en annulering moeten de volledige procesboom op Windows en Linux beëindigen. De huidige tests tonen timeout en outputoverschrijding voor een direct childproces aan, niet het opruimen van subprocessen.
- [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 is toegevoegd met feature `tokio1`; de MSRV 1.87 is lager dan de gebruikte Rust 1.99.1. De code gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. De wrapper-smoketest slaagt, maar bewijst niet dat een volledige procesboom bij timeout/outputoverschrijding wordt beëindigd. Test dit op Windows en Linux.
- Aandachtspunt: `stop_child` haalt de status op met `try_wait()`, maar de huidige code gebruikt die status niet voordat `kill()` wordt aangeroepen. Controleer dat een al beëindigd proces geen fout oplevert die de oorspronkelijke timeout-/outputfout maskeert.
- HTTP-foutmapping: 400 invoerfout, 404 onbekende tool, 413 request te groot, 503 capaciteit vol, 504 timeout, 502 outputlimiet. Een niet-nul exitcode geeft HTTP 200 met `success: false`.

Deze afspraken staan ook in [API en configuratie](../docs/api-en-configuratie.md) en het [securityontwerp](../docs/security.md).

## Problemen en oplossingen

Noteer concrete oorzaak, oplossing en hoe de oplossing is gecontroleerd.

## Verificatie

De gebruiker heeft de volgende gerichte tests op Windows uitgevoerd en gemeld dat ze slagen:

- De oorspronkelijke tests voor parameterloze toolselectie, onbekende tools, tools met parameters en output binnen/boven de helperlimiet.
- `weigert_extra_uitvoering_bij_volledige_capaciteit`: een extra uitvoering wordt afgewezen wanneer alle permits bezet zijn.
- `geeft_capaciteit_vrij_als_starten_mislukt`: capaciteit komt vrij na een startfout.
- `start_proces_en_wacht_op_succesvolle_afloop`: `git --version` start, stdout en stderr worden met de helper tegelijk gelezen, en het proces eindigt succesvol.
- `geeft_resultaat_bij_niet_succesvolle_exitcode`: een proces dat start maar met een niet-nul exitcode eindigt levert een `ExecutionResult` met `success: false` en stderr.
- `geeft_timeout_als_proces_te_lang_duurt`: een langer draaiend direct childproces levert `TimedOut`; de test controleert daarna dat alle capaciteits-permits opnieuw beschikbaar zijn.
- `stopt_proces_bij_outputoverschrijding`: een childproces schrijft meer dan het outputbudget; de executor geeft `OutputLimitExceeded` terug en capaciteit komt vrij.
- `begrenz_stdout_en_stderr_met_een_gedeeld_budget`: twee streams delen aantoonbaar één outputbudget.
- `start_met_platformwrapper_en_wacht_op_succes`: start `git --version` via `process-wrap` en wacht op succesvolle afloop.

De gebruiker meldt dat `cargo test --locked execution::tests` op 2026-10-29 slaagt: 13 passed, 1 ignored, 14 filtered. Dit zijn gerichte tussentijdse controles, geen volledige verificatie van F03. De formatter-, Clippy- en volledige testsuite zijn niet gemeld. De wrapper-smoketest bevestigt alleen starten en wachten; opruiming van een child met eigen subprocessen is niet getest. Annulering, HTTP-integratie, requestlimiet en Linux-verificatie blijven open.

## Werksessies

### 2026-10-26 — F03 gestart

- Doel: F03 voorbereiden en uitvoeringsafspraken vastleggen.
- Gewijzigd: gedeelde uitvoeringsresultaten en fouttypen, parameterloze toolselectie, command-opbouw, concurrency- en outputbudgethelpers, en vijf unit tests. Documentatie bijgewerkt; branch `feature/F03-procesuitvoering` actief.
- Geleerd: procesboomopruiming moet expliciet per platform worden gerealiseerd en getest; een timeouterror alleen bewijst niet dat subprocessen zijn beëindigd.
- Gecontroleerd: vijf gerichte unit tests slagen volgens gebruiker. De complete procesuitvoering is nog niet aanwezig.
- Open vragen: issue-referentie vastleggen; procescoördinatie, timeout, outputstream-concurrency, procesboomimplementatie en tests op Windows/Linux uitvoeren.

### 2026-10-27 — Eerste procesbouwstenen en iteratieve leerwerkwijze

- Doel: de processtart en het gezamenlijk begrensd lezen van stdout en stderr stapsgewijs opbouwen en testen.
- Gewijzigd: `start_process` reserveert capaciteit en start de geconfigureerde tool; `create_command` gebruikt `kill_on_drop(true)`; `read_process_output` leest beide pipes tegelijk met het gedeelde budget. Tests controleren de capaciteitsgrens, vrijgave bij startfout, een succesvolle `git --version`-run en het gedeelde outputbudget.
- Geleerd: `Option::take()` haalt een pipe uit `Child`; als de test die pipe al heeft weggehaald, kan de helper hem niet opnieuw ophalen. De test hoort de pipes daarom aan de helper over te laten. Ook is een geslaagde read-helper nog geen volledige procescoördinatie.
- Gecontroleerd: de gebruiker heeft de bovenstaande gerichte tests uitgevoerd en gemeld dat ze slagen. Geen volledige suite of Clippy/formattercontrole gemeld.
- Werkwijze: eerst een kleine werkende verticale slice maken, die gericht verifiëren en daarna uitbreiden tot de acceptatiecriteria. Code in korte fragmenten aanbieden met bestand/plek en syntax-uitleg; de eigenaar typt zelf en bevestigt de testuitkomst. Documentatie alleen aanpassen op verzoek.
- Open: resultaatcoördinatie, timeout, echte outputoverschrijding met opruiming, annulering, procesboomgedrag en HTTP-route. Zie ook de [projectbrede leerwerkwijze](../AGENTS.md#leerbegeleiding).

### 2026-10-27 — Resultaat, timeout en outputlimiet

- Doel: de verticale slice uitbreiden met een bruikbaar `ExecutionResult` en voorspelbare foutuitkomsten.
- Gewijzigd: `execute_parameterless` coördineert toolselectie, start, gelijktijdig begrensd lezen, wachten en resultaatvorming. Timeout wordt uit de toolconfiguratie gelezen met 10 seconden als standaard. `stop_child` probeert bij timeout en leesfout het directe proces te beëindigen. Een test gebruikt de Rust-testbinary als child voor output boven de limiet.
- Geleerd: een niet-nul exitcode is een normaal `ExecutionResult` met `success: false`, niet een processtartfout. Een gerichte test van outputoverschrijding moet aantonen dat de bedoelde childtest daadwerkelijk draait; een verkeerde `--exact`-filter resulteerde eerst in nul childtests en alleen output van het testharnas.
- Gecontroleerd: de gebruiker meldt dat de tests voor succesvolle uitvoering, niet-nul exitcode, timeout (inclusief vrijgekomen capaciteit) en outputoverschrijding door een childproces (inclusief vrijgekomen capaciteit) slagen. Geen volledige Cargo-controles gemeld.
- Procesbeheer: onderzocht is [`process-wrap`](https://docs.rs/process-wrap/), opvolger van `command-group`, met Tokio-ondersteuning, Unix `ProcessGroup` en Windows `JobObject`. De geraadpleegde documentatie noemt Rust 1.87 als MSRV en MIT/Apache-2.0 als licenties. De dependency is nog niet toegevoegd of geïntegreerd; MSRV en gedrag bij een subprocess moeten eerst worden gecontroleerd. De huidige code stopt alleen het directe childproces.
- Open: bevestig cratekeuze na controle van API/features en MSRV; integreer procesboomopruiming en test met child én subprocess op Windows/Linux; werk annulering/shutdown af; daarna HTTP-route en integratietests.

### 2026-10-29 — Platformwrapper aangesloten

- Doel: de bestaande executor via `process-wrap` laten starten en beheren.
- Gewijzigd: dependency `process-wrap` 10.0.1 met feature `tokio1`; Unix gebruikt `ProcessGroup::leader()`, Windows `JobObject`, en `KillOnDrop` is toegevoegd. `start_process`, outputlezen, wachten en `stop_child` gebruiken `ChildWrapper`.
- Gecontroleerd: de gebruiker meldt dat `cargo test --locked execution::tests` slaagt met 13 passed, 1 ignored en 14 filtered. De wrapper-smoketest start `git --version` en wacht op succes.
- Beperking: de smoke-test bewijst geen beëindiging van subprocessen; de test is niet op beide platforms aangetoond. Volledige formatter-, Clippy- en testsuitecontroles zijn niet gemeld.
- Open: controleren van de `try_wait()`-afhandeling in `stop_child`; een proces met eigen subprocess testen bij timeout/outputoverschrijding op Windows en Linux; daarna annulering/shutdown en HTTP-route.

## Vervolg

De wrapper is geïntegreerd en de smoke-test slaagt. Controleer eerst de statusafhandeling in `stop_child`. Voeg daarna een gerichte test toe waarin het childproces zelf een subprocess start; verifieer dat beide worden beëindigd bij timeout én outputoverschrijding op Windows en Linux. Onderzoek daarna annulering/shutdown en sluit pas vervolgens de HTTP-route aan. Markeer niets als geverifieerd totdat de betreffende controle is uitgevoerd.
