---
feature: F03
tags:
  - commandhub
  - feature-notitie
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

[Featureplan](../docs/features/F03-procesuitvoering.md)

## Wat bouw ik en waarom?

F03 bevat een gedeelde, begrensde procesuitvoerder en `POST /tools/{toolname}` voor geconfigureerde tools zonder dynamische parameters. De kern levert stdout, stderr, exitcode, succes en duur en begrenst output, tijd en capaciteit. Expliciete annulering en shutdown zijn aangesloten. De HTTP-route is op Windows automatisch en handmatig geverifieerd. Gerapporteerde stand op 2026-10-30: **55 geslaagde tests, 2 genegeerde helpers**. Logging, echte HTTP-concurrency, formatter/Clippy en als laatste Linux blijven open. F03 blijft In uitvoering.

## Hoe werkt de code?

`src/execution.rs` bevat `ExecutionResult`, `ExecutionError` en `ExecutionService`. Toolselectie weigert onbekende tools en tooldefinities met parameters/placeholders. De lifetime bij de gevonden definitie betekent dat die uit de registry geleend blijft; er wordt geen tool gekopieerd.

`Arc<Semaphore>` deelt vier capaciteits-permits. `try_acquire_owned` wijst volle capaciteit direct af. Een `OwnedSemaphorePermit` geeft de plek terug bij drop. De command-opbouw houdt executable en argumenten gescheiden, sluit stdin en pipet stdout/stderr. Omgeving en werkdirectory worden van de server geërfd.

`read_capped` leest blokken van 8192 bytes. Een gedeeld bytebudget begrenst stdout en stderr samen op 1 MiB. De outputstructuur bewaart de bijbehorende permits zolang de bytes aanwezig zijn. `try_join!` leest beide streams tegelijk. De uitvoeringsfuture wacht daarna op het proces; `timeout` bewaakt de geconfigureerde duur.

`process-wrap` 10.0.1 met `tokio1` gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. `Box<dyn ChildWrapper>` biedt een gemeenschappelijke interface voor pipes, wachten en beëindigen. `stop_child` gebruikt `try_wait`: alleen bij ontbrekende exitstatus wordt `kill` afgewacht. Een opruimfout kan via `?` de oorspronkelijke uitvoeringsfout vervangen.

## Annulering, shutdown en eigendom

`execute_parameterless` delegeert naar de annuleerbare methode. Die ontvangt een `oneshot::Receiver<()>`: één expliciet signaal zonder aanvullende gegevens. `tokio::select!` wacht op uitvoering met timeout, `Ok(())` uit dit kanaal of shutdown. Een gesloten zender zonder bericht past niet op `Ok(())` en annuleert niet.

Shutdown gebruikt `watch::Sender<bool>`: `false` is actief, `true` betekent afsluiten. Elke uitvoering maakt vóór processtart een ontvanger. `try_acquire` houdt de leesbewaker vast tijdens controle en reservering, zodat shutdown niet daartussen kan worden ingesteld. `send_replace(true)` bewaart de status ook zonder ontvangers. Nieuwe uitvoeringen krijgen `Cancelled`; actieve uitvoeringen gaan door hun opruimroute.

`shutdown().await` vraagt alle vier permits tegelijk aan en wacht zo op afronding van bestaande uitvoeringsfutures. Hij verzamelt geen opruimfouten en bewijst zonder PID-controle niet dat elk proces weg is. Bij Ctrl+C signaleert `main.rs` eerst shutdown, laat Axum graceful afsluiten en wacht daarna op de uitvoeringsservice, ook voordat een eventuele serverfout wordt doorgegeven.

`AppState` bevat gedeelde `Arc`-verwijzingen naar registry en executor. `Clone` kopieert deze verwijzingen, niet de service. `FromRef<AppState>` laat de ontdekcontroller alleen de registry ontvangen. `async move` geeft de Ctrl+C-future een eigen serviceverwijzing. `state.clone()` voorkomt dat de router de enige state verbruikt voordat shutdown die opnieuw nodig heeft.

**Stopbeleid:** expliciete annulering/shutdown is de normale route met afgewachte opruimpoging. Future-drop en `JoinHandle::abort` gebruiken `KillOnDrop` als vangnet. Daarbij is geen normaal uitvoeringsresultaat of afgewachte opruimfout beschikbaar. Na `abort` wacht de test de taak af voordat capaciteit en PID worden gecontroleerd. Deze scenario's zijn getest op Windows; Linux en expliciet HTTP-clientdisconnectbeleid blijven open.

## HTTP-adapter

`src/controllers/execution_controller.rs` leest `State`, `Path` en als laatste de body via `Result<Json<Value>, JsonRejection>`. De handler vangt extractiefouten op en accepteert uitsluitend een leeg object. Pas daarna wordt de tool geselecteerd. `DefaultBodyLimit` op de POST-route begrenst JSON-uitlezen op 16 KiB vóór deserialisatie. Precies 16 KiB mag door.

`src/models/execution.rs` vertaalt met `From<ExecutionResult>` het kernresultaat naar `ExecutionResponse`. De strings worden verplaatst, niet gekloond. Serde `camelCase` maakt `exitCode`; `Option<i32>` wordt een getal of null. `Duration::as_millis()` levert `u128`, gebruikt voor het numerieke veld `duration`. De uitvoeringskern krijgt geen HTTP- of Serde-afhankelijkheid.

Foutresponses gebruiken vaste `code` en `message` met `&'static str`; interne OS-fouten worden niet teruggestuurd. Mapping: 400 ongeldige JSON/invoer of parameters, 404 onbekende tool, 413 body te groot, 500 interne procesbeheerfout, 502 outputlimiet, 503 volle capaciteit of annulering/shutdown, 504 timeout. Een niet-nul exitcode blijft HTTP 200 met `success: false`. Zie [API-contract](../docs/api-en-configuratie.md) voor details.

## Testindeling en procescontrole

Alle vijf testmodules staan onder `src/tests/`. De productiebestanden koppelen ze via `#[cfg(test)]`, `#[path]` en `mod tests;`. Voor modules in `controllers/` en `models/` begint het pad met `../tests/`. De logische modules blijven hetzelfde: `super`-imports en de helperselectie `execution::tests::subprocess_helper_bij_timeout` wijzigen niet. Het is geen projectmap met zelfstandige integratiecrates.

De testbinary start genegeerde helpers als aparte processen. De subprocess-helper schrijft een PID-bestand en start `ping` op Windows of `sleep` op Unix. Markers via `--skip` kiezen grote output of een direct child dat eerder terugkeert; drop van een `std::process::Child` beëindigt het subprocess niet automatisch. Oude PID-bestanden worden vooraf verwijderd; scenario's hebben verschillende namen met het bovenliggende test-PID. `join!` laat uitvoering en signaalgever voortgang maken totdat een geldig PID beschikbaar is.

De zeven Windows-opruimscenario's gebruiken `assert_windows_process_stopped`. De proceslijst wordt async opgevraagd met 3 s timeout en `kill_on_drop(true)`; gedurende maximaal 5 s wordt elke 50 ms opnieuw gecontroleerd totdat het PID ontbreekt. Dat voorkomt een enkele timinggevoelige momentopname. Eventuele blokkerende OS-processtart krijgt hiermee geen harde tijdsgarantie. Linux heeft nog geen PID-controle.

## Verificatie — actuele stand 2026-10-30

| Controle | Resultaat en beperkingen |
| --- | --- |
| `cargo test --locked` na testverplaatsing | Gerapporteerd: 55 passed, 0 failed, 2 ignored, 0 filtered out; 2,89 s op Windows |
| Echte `POST /tools/git_version`, JSON `{}` | Succes; stdout `git version 2.55.0.windows.3\n`, lege stderr, exitcode 0, duration 61 ms |
| Status en Ctrl+C | Status werkt; terminalprompt keert terug bij Ctrl+C zonder actieve tools |
| Formatter en Clippy | Niet opnieuw geverifieerd na alle uitbreidingen; eerdere afkeur op 2026-10-29 is historisch |
| Linux | Niet uitgevoerd; lokaal geen Linux-omgeving. Als laatste behandelen via te controleren CI-opzet |

De suite omvat configuratie, uitvoering, resultaatmodel, controller en router. De twee genegeerde functies zijn helpers die door andere tests expliciet worden gestart, geen overgeslagen acceptatietests. HTTP-controles bewaken succes, niet-nul exit, 400/404/413/500/502/503/504, parameters afwijzen, shutdownafwijzing en bodygrens. De 503-capaciteitstest reserveert vier permits; vijf echte gelijktijdige HTTP-verzoeken en Ctrl+C tijdens actieve HTTP-uitvoering zijn nog niet getest.

Tijdens deze documentatie-update zijn de actuele code en tests nagelezen. Er zijn geen Cargo- of Git-commando's uitgevoerd; de gerapporteerde controles zijn vastgelegd. Alleen documentatie is gewijzigd.

## Problemen en oplossingen

- De eerste subprocess-timeouttest controleerde alleen een foutuitkomst. PID-bestanden en processtatuscontrole maken opruiming nu controleerbaar.
- Een overgebleven ongebruikt `oneshot::channel()` in de shutdowntest gaf E0282: zonder gebruik kon Rust het berichttype niet afleiden. Het overbodige kanaal is verwijderd.
- State zonder `.clone()` aan de router doorgeven gaf E0382 bij later shutdowngebruik. Gedeelde verwijzingen klonen behoudt beide eigenaars.
- Blokkerend `tasklist` en één momentopname zijn vervangen door async begrensd pollen. `from_utf8_lossy` voorkomt fouten door procesnamen met niet-UTF-8-bytes.
- Testcode maakte productiebestanden lang. Verplaatsing naar `src/tests/` houdt code gescheiden met behoud van private toegang en helperpaden.
- Naamopschoning van de helper en `_cancellation`, overige formatter-/lintmeldingen en Linux-opruimsemantiek zijn nog niet als afgerond geverifieerd.

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

### 2026-10-30 — Shutdown, HTTP-route en teststructuur

- Toegevoegd: shutdown met `watch`, toelatingscontrole, wachten op capaciteitsvrijgave, gedeelde `AppState` en Ctrl+C-koppeling.
- Getest: shutdownafwijzing, wachten op een vastgehouden permit, subprocessopruiming bij shutdown, future-drop en taakafbreking op Windows.
- Verbeterd: Windows-PID-controles async en begrensd, met herhaalde controle in plaats van één momentopname.
- Toegevoegd: parameterloze POST-route, resultaat-/foutmodellen, uitsluitend leeg JSON-object, bodylimiet en foutmapping met routertests.
- Gecontroleerd: echte HTTP-aanroep geeft Git-versie, succes, exitcode 0 en 61 ms; volledige suite 55 geslaagd en 2 genegeerde helpers.
- Geordend: alle testmodules naar `src/tests/` verplaatst; volledige suite opnieuw geslaagd.
- Overdracht: eerst logging en echte HTTP-concurrency, daarna Windows-eindcontroles en documentatie, als laatste Linux. F03 blijft In uitvoering; geen commit/merge uitgevoerd.

## Vervolg

Behandel nu het todo-punt uitvoeringslogging met toolnaam, uitkomst/foutcategorie en duur, zonder parameterwaarden, executablepaden, geheimen of stdout/stderr. Voeg daarna een integratiecontrole met vijf echte gelijktijdige requests en shutdown tijdens actieve HTTP-uitvoering toe. Voer formatter, Clippy en volledige tests uit en los eventuele meldingen op. Als laatste Linux-procescontrole en daadwerkelijke verificatie via CI; lokaal is geen Linux beschikbaar.

Blijf werken met kleine controleerbare leerstappen, exacte bestandslocaties en begrijpelijke Rust-uitleg. Benoem het behandelde todo-punt en de geverifieerde afronding. Geef volledige functies wanneer dat het invoeren verduidelijkt. Bestanden lezen mag; wijzigingen en overige uitvoering alleen op expliciet verzoek. Documentatie algemeen formuleren en alleen op verzoek aanpassen. Geen Git-commando's uitvoeren.
