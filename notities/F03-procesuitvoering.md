---
feature: F03
tags:
  - commandhub
  - feature-notitie
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

[Featureplan](../docs/features/F03-procesuitvoering.md)

## Wat bouw ik en waarom?

F03 voegt een gedeelde, begrensde procesuitvoerder en `POST /tools/{toolname}` toe voor geconfigureerde tools zonder dynamische parameters. De uitvoeringskern start een ingestelde tool, leest stdout en stderr tegelijk binnen één budget, wacht op het proces en levert een `ExecutionResult`. Expliciete annulering per uitvoering is toegevoegd. `process-wrap` 10.0.1 gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Gerapporteerd testresultaat op 2026-10-29: 20 geslaagde execution-tests op Windows, naast 2 genegeerde helpers. De tests controleren subprocess-PID's na timeout, outputoverschrijding en annulering, ook bij een eerder afgerond direct child. Server-shutdown, Linux-verificatie, HTTP-integratie en volledige Cargo-controles blijven open. De actuele checklist staat in het [featureplan](../docs/features/F03-procesuitvoering.md).

## Hoe werkt de code?

In `src/execution.rs` beschrijft `ExecutionResult` een afgeronde run en onderscheidt `ExecutionError` de uitvoeringsfouten. `find_parameterless_tool` zoekt in de registry en weigert tools met parameters of placeholders. De lifetime `'a` geeft aan dat de gevonden `ToolDefinition` geleend wordt uit de registry.

`ExecutionService` houdt met `Arc<Semaphore>` een gedeelde capaciteit bij. `try_acquire_owned` wijst direct af wanneer de vier permits bezet zijn; een `OwnedSemaphorePermit` geeft de plek automatisch terug wanneer hij wordt gedropt. `create_command` bouwt een Tokio-`Command` met executable en argumenten apart en sluit stdin.

`read_capped` leest een async stream in blokken van 8192 bytes. Een gedeelde semaphore bewaakt het gezamenlijke outputbudget; de permits blijven bij `CapturedOutput` bewaard zolang de bytes in het geheugen staan. `read_process_output` haalt de stdout- en stderr-pipes uit het childproces en gebruikt `tokio::try_join!` om ze tegelijk met hetzelfde budget te lezen.

`ExecutionService::start_process` reserveert een capaciteit-permit en start het proces met `create_wrapped_command`. De wrapper geeft `Box<dyn ChildWrapper>` terug; via die trait leest de executor de pipes en wacht of beëindigt hij het proces. Als starten mislukt, wordt de permit automatisch vrijgegeven. Bij timeout, annulering of een uitvoeringsfout roept de executor `stop_child` aan. Die controleert met `try_wait()` of het child al is afgerond: `Some(status)` betekent afgerond, `None` betekent nog geen exitstatus. Alleen bij `None` wordt `kill()` aangeroepen. De regressietest voor een al afgerond proces en het Windows-scenario met een achterblijvend subprocess zijn als geslaagd gerapporteerd; welke wrapperactie in dat laatste scenario opruimt is niet afzonderlijk vastgesteld.

`execute_parameterless` delegeert naar `execute_parameterless_with_cancellation` en bewaart zelf de zender. De nieuwe methode ontvangt een `oneshot::Receiver<()>`: één bericht zonder inhoud, waarbij alleen het signaal telt. `tokio::select!` wacht op de uitvoering met timeout of op `Ok(())` uit het kanaal. Een verstuurd bericht wordt `Ok(Err(ExecutionError::Cancelled))`, zodat de bestaande foutarm eerst de opruiming afwacht. Een zender die zonder bericht wordt gedropt levert een kanaalfout; het patroon `Ok(())` past dan niet en de annuleringsarm wordt uitgeschakeld. De uitvoering gaat verder.

De helper `subprocess_helper_bij_timeout` wordt als afzonderlijke testbinary gestart. Hij start `ping` op Windows of `sleep` op Unix, schrijft het subprocess-ID naar een tijdelijk bestand en wacht normaal op het subprocess. Markers, meegegeven als `--skip`-filters aan het Rust-testharnas, kiezen een PID-bestand, grote output of vroegtijdige terugkeer van de helper. Een `std::process::Child` droppen beëindigt het proces niet automatisch. Elke procesboomtest gebruikt een eigen bestandsnaam met het PID van de bovenliggende testrun en verwijdert een oud bestand vooraf.

`windows_process_ids` leest de CSV-uitvoer van `tasklist`, controleert het resultaat en verzamelt de PID-kolom. `String::from_utf8_lossy` voorkomt een fout door niet-UTF-8-bytes in procesnamen; de numerieke kolom wordt strikt geparseerd. De tests controleren dat het opgeslagen PID ontbreekt. De annuleringstest gebruikt `tokio::join!` om uitvoering en signaalgever samen voortgang te laten maken: pas zodra het PID-bestand een geldig getal bevat wordt het signaal verstuurd. Hiermee vermijden we een willekeurige slaap vóór annulering.

## Keuzes en alternatieven

- F03 voert uitsluitend tools zonder dynamische parameters uit; placeholdervervanging hoort bij F04.
- Native executables zijn toegestaan. Scripts starten alleen via een expliciet geconfigureerde interpreter met vaste argumenten; geen automatische shell- of `.bat`/`.cmd`-dispatch.
- Het childproces erft de omgeving en werkdirectory van de server. De client kan deze niet instellen.
- Limieten: 10.000 ms standaardtimeout, maximaal 60.000 ms timeout, 1 MiB gecombineerde stdout/stderr, 16 KiB requestbody en vier gelijktijdige uitvoeringen. Volle capaciteit wordt direct afgewezen.
- Timeout, outputoverschrijding en annulering moeten de volledige procesboom op Windows en Linux beëindigen. De gerichte Windows-tests controleren nu het verdwijnen van een subprocess; Linux en shutdown blijven te bewijzen.
- [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 is toegevoegd met feature `tokio1`; bij de eerdere cratekeuze is MSRV 1.87 vastgelegd tegenover gebruikte Rust 1.99.1. De code gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. De nieuwe PID-tests gaan verder dan de wrapper-smoketest; ze vormen nog geen garantie voor alle procesbomen of Linux.
- `stop_child` gebruikt de exitstatus om een al afgerond child niet opnieuw te beëindigen. De Windows-tests voor een afgerond child met en zonder subprocess slagen. Omdat de `Some`-tak geen `kill()` aanroept, blijft toetsing van dat scenario op Linux belangrijk. Een fout bij controleren of beëindigen wordt met `?` doorgegeven en vervangt de oorspronkelijke uitvoeringsfout.
- Alleen een expliciet annuleringsbericht stopt een uitvoering; een gesloten kanaal zonder bericht niet. De bestaande methode zonder expliciete annulering blijft bruikbaar. Serverbrede shutdown, clientdisconnects en het afbreken van de uitvoeringsfuture zijn afzonderlijke vervolgstappen.
- HTTP-foutmapping: 400 invoerfout, 404 onbekende tool, 413 request te groot, 503 capaciteit vol, 504 timeout, 502 outputlimiet. Een niet-nul exitcode geeft HTTP 200 met `success: false`.

Deze afspraken staan ook in [API en configuratie](../docs/api-en-configuratie.md) en het [securityontwerp](../docs/security.md).

## Problemen en oplossingen

Een timeoutfout alleen bewees niet dat het subprocess weg was. Daarom schrijft de helper nu zijn subprocess-ID weg en controleert de Windows-test de proceslijst achteraf. Het vooraf verwijderen van oude bestanden voorkomt dat een vorige run als startbewijs wordt gebruikt. Verschillende bestandsnamen voorkomen botsingen tussen de parallelle scenario's.

De eerste proceslijstcontrole veronderstelde UTF-8; dat kan op Windows onjuist zijn. Omgezet naar `from_utf8_lossy`, omdat de test alleen de ASCII-scheiding en numerieke PID-kolom nodig heeft. Dubbele proceslijstcode is daarna ondergebracht in `windows_process_ids`.

Open beperkingen: de procescontrole is één momentopname, kan gevoelig zijn voor vertraagde beëindiging en gebruikt een blokkerende `tasklist`-aanroep die de async-timeout niet betrouwbaar begrenst. Bij een falende assertie kan het tijdelijke bestand achterblijven; de volgende run verwijdert het vooraf. De helpernaam noemt nog timeout hoewel hij meerdere scenario's ondersteunt, en `_cancellation` wordt inmiddels gebruikt; eventuele naamopschoning is nog niet uitgevoerd.

## Verificatie

### Actuele stand na de leerstappen — 2026-10-29

Voor `cargo test --locked execution::tests` zijn 20 geslaagde tests gerapporteerd, naast 2 genegeerde helpers; 14 overige tests vallen buiten het filter. De actuele uitvoeringscode en tests zijn nagelezen. Er zijn tijdens deze documentatie-update geen Cargo- of Git-commando's uitgevoerd, conform de afspraak.

Naast de oorspronkelijke tests zijn de volgende tests als geslaagd gerapporteerd:

- `stopt_subproces_bij_timeout`: PID vastleggen, timeout, capaciteitsvrijgave en op Windows ontbreken van het subprocess in de proceslijst.
- `stopt_subproces_bij_outputoverschrijding`: dezelfde controles bij output boven de limiet.
- `stopt_subproces_als_parent_al_is_afgerond`: helper keert terug zonder op het subprocess te wachten; de Windows-PID-controle slaagt na timeout.
- `annuleert_uitvoering_bij_expliciet_signaal`: vooraf verstuurd bericht geeft `Cancelled` en capaciteit komt vrij.
- `voert_tool_uit_als_annuleringszender_wordt_gedropt`: sluiten zonder bericht laat `git --version` succesvol afronden.
- `stopt_subproces_bij_annulering`: pas na een leesbaar PID annuleren, daarna `Cancelled`, capaciteitsvrijgave en Windows-PID-controle.

De subprocess-timeout gebruikt 1500 ms. Output- en annuleringstests gebruiken een tooltimeout van 10 seconden en een buitenste bewaking van 6 seconden; de bedoelde fout moet dus vóór de tooltimeout optreden. De bewaking begrenst niet betrouwbaar de blokkerende proceslijstcontrole. Linux bevat nog geen PID-controle en is niet uitgevoerd. Server-shutdown, future-drop/abort, HTTP-route, requestlimiet en logging zijn niet getest.

Formatter, Clippy en de volledige suite zijn na de uitbreidingen niet opnieuw uitgevoerd. De waarschuwing voor ongebruikte `Cancelled` is niet meer gerapporteerd. De overige eerder vastgestelde meldingen zijn nog niet als opgelost geverifieerd.

### Eerdere controle vóór de bovenstaande uitbreidingen

De volgende gerichte Windows-tests zijn als geslaagd gerapporteerd:

- De oorspronkelijke tests voor parameterloze toolselectie, onbekende tools, tools met parameters en output binnen/boven de helperlimiet.
- `weigert_extra_uitvoering_bij_volledige_capaciteit`: een extra uitvoering wordt afgewezen wanneer alle permits bezet zijn.
- `geeft_capaciteit_vrij_als_starten_mislukt`: capaciteit komt vrij na een startfout.
- `start_proces_en_wacht_op_succesvolle_afloop`: `git --version` start, stdout en stderr worden met de helper tegelijk gelezen, en het proces eindigt succesvol.
- `geeft_resultaat_bij_niet_succesvolle_exitcode`: een proces dat start maar met een niet-nul exitcode eindigt levert een `ExecutionResult` met `success: false` en stderr.
- `geeft_timeout_als_proces_te_lang_duurt`: een langer draaiend direct childproces levert `TimedOut`; de test controleert daarna dat alle capaciteits-permits opnieuw beschikbaar zijn.
- `stopt_proces_bij_outputoverschrijding`: een childproces schrijft meer dan het outputbudget; de executor geeft `OutputLimitExceeded` terug en capaciteit komt vrij.
- `begrenz_stdout_en_stderr_met_een_gedeeld_budget`: twee streams delen aantoonbaar één outputbudget.
- `start_met_platformwrapper_en_wacht_op_succes`: start `git --version` via `process-wrap` en wacht op succesvolle afloop.

Het eerder gerapporteerde resultaat (13 passed, 1 ignored, 14 filtered) is op 2026-10-29 aangevuld met een controle van de actuele code en de volledige testsuite op Windows:

| Controle | Resultaat bij de eerdere controle |
| --- | --- |
| `cargo fmt --check` | Faalt: formatteringsverschillen in `src/execution.rs` en `src/execution/tests.rs` |
| `cargo clippy --all-targets --locked -- -D warnings` | Faalt: overbodige haakjes bij de placeholdercontrole; ongebruikte uitvoeringscode in de hoofdapplicatie; ongelezen `duration`; ongebruikte `Cancelled`; niet meer vervulde `#[expect(dead_code)]` bij `ToolRegistry`-ondersteunende configuratiecode in `src/config.rs:90` |
| `cargo test --locked` | Slaagt: 29 passed, 0 failed, 2 ignored, 0 filtered out; 15 geslaagde execution-tests en 14 overige tests |

De twee genegeerde functies zijn helpers (`schrijft_output_boven_de_limiet` en `subprocess_helper_bij_timeout`). Ze worden bewust niet rechtstreeks in de hoofdrun uitgevoerd, maar via `--exact`, `--ignored` en `--nocapture` door hun bijbehorende tests als childproces gestart.

Bij die eerdere controle nieuw aangetroffen execution-tests:

- `stopt_geen_al_afgerond_proces`: wacht op `git --version` en controleert dat `stop_child` daarna zonder fout terugkeert.
- `stopt_subproces_bij_timeout`: start de testbinary als child; de helper start op Windows `ping -n 30 127.0.0.1` en op Unix `sleep 30`. De tooltimeout is 1500 ms; een buitenste timeout van 6 seconden bewaakt de test. De test verwacht `TimedOut`, maar controleert geen proces-ID, startbevestiging van het subprocess of processtatus achteraf. Daarmee is volledige procesboomopruiming nog niet bewezen.

De eerdere controles vereisten toegang buiten de sandbox omdat Rustup binnen de sandbox zijn toolchainmap niet kon openen. Na toestemming zijn alle drie uitgevoerd. Bij die documentatiecontrole is uitsluitend documentatie gewijzigd. Deze resultaten beschrijven de toenmalige code; de actuele resultaten en resterende punten staan hierboven.

## Werksessies

### 2026-10-26 — F03 gestart

- Doel: F03 voorbereiden en uitvoeringsafspraken vastleggen.
- Gewijzigd: gedeelde uitvoeringsresultaten en fouttypen, parameterloze toolselectie, command-opbouw, concurrency- en outputbudgethelpers, en vijf unit tests. Documentatie bijgewerkt; branch `feature/F03-procesuitvoering` actief.
- Geleerd: procesboomopruiming moet expliciet per platform worden gerealiseerd en getest; een timeouterror alleen bewijst niet dat subprocessen zijn beëindigd.
- Gecontroleerd: vijf gerichte unit tests zijn als geslaagd gerapporteerd. De complete procesuitvoering is nog niet aanwezig.
- Open vragen: issue-referentie vastleggen; procescoördinatie, timeout, outputstream-concurrency, procesboomimplementatie en tests op Windows/Linux uitvoeren.

### 2026-10-27 — Eerste procesbouwstenen en iteratieve leerwerkwijze

- Doel: de processtart en het gezamenlijk begrensd lezen van stdout en stderr stapsgewijs opbouwen en testen.
- Gewijzigd: `start_process` reserveert capaciteit en start de geconfigureerde tool; `create_command` gebruikt `kill_on_drop(true)`; `read_process_output` leest beide pipes tegelijk met het gedeelde budget. Tests controleren de capaciteitsgrens, vrijgave bij startfout, een succesvolle `git --version`-run en het gedeelde outputbudget.
- Geleerd: `Option::take()` haalt een pipe uit `Child`; als de test die pipe al heeft weggehaald, kan de helper hem niet opnieuw ophalen. De test hoort de pipes daarom aan de helper over te laten. Ook is een geslaagde read-helper nog geen volledige procescoördinatie.
- Gecontroleerd: de bovenstaande gerichte tests zijn als geslaagd gerapporteerd. Geen volledige suite of Clippy/formattercontrole gemeld.
- Werkwijze: eerst een kleine werkende verticale slice maken, die gericht verifiëren en daarna uitbreiden tot de acceptatiecriteria. Code in korte fragmenten aanbieden met bestand/plek en syntax-uitleg; na het invoeren volgt een gerichte controle voordat de volgende stap begint. Documentatie alleen aanpassen op verzoek.
- Open: resultaatcoördinatie, timeout, echte outputoverschrijding met opruiming, annulering, procesboomgedrag en HTTP-route. Zie ook de [projectbrede leerwerkwijze](../AGENTS.md#leerbegeleiding).

### 2026-10-27 — Resultaat, timeout en outputlimiet

- Doel: de verticale slice uitbreiden met een bruikbaar `ExecutionResult` en voorspelbare foutuitkomsten.
- Gewijzigd: `execute_parameterless` coördineert toolselectie, start, gelijktijdig begrensd lezen, wachten en resultaatvorming. Timeout wordt uit de toolconfiguratie gelezen met 10 seconden als standaard. `stop_child` probeert bij timeout en leesfout het directe proces te beëindigen. Een test gebruikt de Rust-testbinary als child voor output boven de limiet.
- Geleerd: een niet-nul exitcode is een normaal `ExecutionResult` met `success: false`, niet een processtartfout. Een gerichte test van outputoverschrijding moet aantonen dat de bedoelde childtest daadwerkelijk draait; een verkeerde `--exact`-filter resulteerde eerst in nul childtests en alleen output van het testharnas.
- Gecontroleerd: de tests voor succesvolle uitvoering, niet-nul exitcode, timeout (inclusief vrijgekomen capaciteit) en outputoverschrijding door een childproces (inclusief vrijgekomen capaciteit) slagen. Geen volledige Cargo-controles gemeld.
- Procesbeheer: onderzocht is [`process-wrap`](https://docs.rs/process-wrap/), opvolger van `command-group`, met Tokio-ondersteuning, Unix `ProcessGroup` en Windows `JobObject`. De geraadpleegde documentatie noemt Rust 1.87 als MSRV en MIT/Apache-2.0 als licenties. De dependency is nog niet toegevoegd of geïntegreerd; MSRV en gedrag bij een subprocess moeten eerst worden gecontroleerd. De huidige code stopt alleen het directe childproces.
- Open: bevestig cratekeuze na controle van API/features en MSRV; integreer procesboomopruiming en test met child én subprocess op Windows/Linux; werk annulering/shutdown af; daarna HTTP-route en integratietests.

### 2026-10-29 — Platformwrapper aangesloten

- Doel: de bestaande executor via `process-wrap` laten starten en beheren.
- Gewijzigd: dependency `process-wrap` 10.0.1 met feature `tokio1`; Unix gebruikt `ProcessGroup::leader()`, Windows `JobObject`, en `KillOnDrop` is toegevoegd. `start_process`, outputlezen, wachten en `stop_child` gebruiken `ChildWrapper`.
- Gecontroleerd: voor `cargo test --locked execution::tests` is het volgende resultaat gerapporteerd: 13 passed, 1 ignored en 14 filtered. De wrapper-smoketest start `git --version` en wacht op succes.
- Beperking: de smoke-test bewijst geen beëindiging van subprocessen; de test is niet op beide platforms aangetoond. Volledige formatter-, Clippy- en testsuitecontroles zijn niet gemeld.
- Open: controleren van de `try_wait()`-afhandeling in `stop_child`; een proces met eigen subprocess testen bij timeout/outputoverschrijding op Windows en Linux; daarna annulering/shutdown en HTTP-route.

### 2026-10-29 — Codecontrole en documentatie bijgewerkt

- Op verzoek uitsluitend documentatie aangepast; de bestaande wijzigingen in `src/execution.rs` en `src/execution/tests.rs` zijn ongemoeid gelaten.
- Vastgesteld: `stop_child` gebruikt nu `try_wait().is_none()`; tests voor een al afgerond child en timeout met een subprocess zijn toegevoegd en slagen op Windows.
- Geleerd: `Option<ExitStatus>` beschrijft de status van het child; een afgerond child bewijst niet dat alle subprocessen weg zijn. Ook bewijst een geslaagde timeouttest zonder statuscontrole achteraf geen volledige opruiming.
- Gecontroleerd: volledige testsuite 29 passed, 2 ignored; formatter en Clippy uitgevoerd maar afgekeurd, zoals hierboven vastgelegd. F03 blijft In uitvoering.

### 2026-10-29 — Procesboombewijs en expliciete annulering

- Werkwijze: kleine codefragmenten met bestandslocatie en Rust-uitleg, gevolgd door invoeren en een gerichte controle per stap. Bestanden lezen is toegestaan; wijzigingen en andere uitvoering alleen op expliciet verzoek. Geen Git-commando's.
- Toegevoegd: PID-bestanden, gedeelde Windows-proceslijsthelper, subprocessscenario's voor timeout/outputlimiet/eerder afgerond child, capaciteitscontroles, expliciete `oneshot`-annulering met `select!`, en drie annuleringstests.
- Geleerd: lenen met `&`, automatisch vrijgeven van permits bij drop, `Option`/`Result`-patronen, een kanaal sluiten tegenover een bericht versturen, en gelijktijdige voortgang via `select!` en `join!`. Een gestarte async future is geen afzonderlijk gespawnde taak; `join!` pollt beide futures in dezelfde taak.
- Verificatie: 20 geslaagde execution-tests gerapporteerd, naast 2 genegeerde helpers. Code gelezen; Linux en volledige Cargo-controles niet opnieuw uitgevoerd.
- Overdracht: volgende werksessie serverstate en Ctrl+C-handler lezen, daarna serverbrede annulering en afgewachte opruiming ontwerpen. F03 blijft In uitvoering. Op expliciet verzoek alleen documentatie bijgewerkt; geen commit uitgevoerd.

## Vervolg

Begin met `src/main.rs`, de router/serverstate en de uitvoeringskern. De huidige Ctrl+C-handler stopt Axum graceful, maar stuurt geen annuleringssignalen naar actieve tooluitvoeringen. Ontwerp hoe shutdown alle actieve uitvoeringen bereikt en hun opruiming afwacht; de bestaande `oneshot` bereikt één uitvoering. Test future-drop/abort apart. Voeg Linux-procescontroles toe en verifieer de platformmatrix, verbeter de Windows-testbewaking en los formatter-/lintmeldingen op. Sluit daarna de HTTP-route aan met requestlimiet, foutmapping en logging. Voer de verplichte volledige Cargo-controles uit vóór afronding. Blijf werken met één kleine leerstap en wacht op de gerichte testuitkomst voordat de volgende stap begint.
