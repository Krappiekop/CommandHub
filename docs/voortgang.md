# Huidige voortgang

Bijgewerkt: 2026-10-29.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. F01 is lokaal gecontroleerd op Windows: formatter, Clippy, tests (0 tests), build en `GET /status` slagen. Een GitHub Actions-workflow voor Ubuntu en Windows is toegevoegd, maar nog niet op GitHub uitgevoerd.

F02 is gemerged. De hoofdapp leest en valideert de JSON-configuratie bij startup, bouwt een immutable registry en serveert `GET /tools` met toolmetadata en inputschema's. De route laat command en vaste argumenten weg. Configuratie-laadfouten zijn gericht getest; handmatig is bevestigd dat ongeldige configuratie startup stopt met een bruikbare diagnose en dat een aangepaste tool na herstart zonder rebuild zichtbaar wordt. Formatter, Clippy en 14 tests slaagden op Windows. De standaardtimeout is 10 seconden (maximaal 60 seconden); standaardbestand is `config/tools.json`, overschrijfbaar met `COMMANDHUB_CONFIG`. Zie [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

F03 is actief op branch `feature/F03-procesuitvoering` (eerder vastgelegd; geen Git-controle in deze sessie). De uitvoeringskern begrenst stdout/stderr, timeout en capaciteit. `process-wrap` 10.0.1 gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. `execute_parameterless_with_cancellation` ontvangt een expliciet `oneshot`-signaal; `tokio::select!` leidt annulering naar dezelfde opruimroute als uitvoeringsfouten. Een zender die zonder bericht wordt gedropt annuleert niet. Gerapporteerd testresultaat: 20 geslaagde execution-tests op Windows, naast 2 genegeerde helpers. Procesboomtests controleren met PID-bestanden en `tasklist` het verdwijnen van subprocessen na timeout, outputoverschrijding en annulering; ook een eerder afgerond direct child is getest. F03 blijft In uitvoering. Zie [F03-featureplan](features/F03-procesuitvoering.md) en [F03-notitie](../notities/F03-procesuitvoering.md).

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, minimale Axum-statusroute met logging, F02-configuratielader, registry, `GET /tools` en tests, F03-uitvoeringskern met `process-wrap` en gerichte tests, CI-workflow en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. De hoofdapplicatie heeft nog geen HTTP-uitvoerroute of MCP-integratie.

## Nog open

- F00: review en PR-merge. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: `Cargo.lock` committen en repository-/develop-/GitHub-Project-inrichting afronden. CI op GitHub laten draaien; Linux is lokaal niet gecontroleerd. Status blijft open tot review en merge.
- F03: actieve uitvoeringen bij server-shutdown annuleren en hun opruiming afwachten; afbreken/droppen van de uitvoeringsfuture apart testen; Linux-procescontroles en platformverificatie toevoegen. Daarna HTTP-uitvoerroute met requestlimiet, foutmapping en logging. Formatter-/lintmeldingen en volledige Cargo-controles blijven open. De Windows-procescontrole gebruikt één momentopname en een blokkerende `tasklist`-aanroep; de async-testtimeout begrenst die aanroep niet betrouwbaar.

## Volgende stap

Begin de volgende werksessie met het lezen van `src/main.rs`, de serverstate/router en de uitvoeringskern. Ontwerp daarna in kleine leerstappen hoe server-shutdown alle actieve uitvoeringen een annuleringssignaal geeft en hun opruiming afwacht. De bestaande `oneshot` geldt voor één uitvoering en is nog niet op Ctrl+C aangesloten. Werk vervolgens de overige F03-punten uit het featureplan af, waaronder Linux-verificatie en HTTP-integratie. F00-review en F01-afronding blijven afzonderlijke projectstappen. De leerwerkwijze bestaat uit kleine codefragmenten met uitleg en een gerichte controle per stap. Bestanden lezen is toegestaan; wijzigingen en overige uitvoering alleen op expliciet verzoek. Geen Git-commando's. Zie ook [AGENTS.md](../AGENTS.md#leerbegeleiding).

## Verificatie

Op Windows zijn voor F01 `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked` en een HTTP-request naar `/status` uitgevoerd; alles slaagde. De testbuild bevat 0 tests. De server stopte na Ctrl+C; de terminal gaf exitcode 1 voor de interrupt. De workflow `.github/workflows/ci.yml` is toegevoegd, maar nog niet door GitHub uitgevoerd. Voor F00 zijn de Cargo-controles ook op de hoofdcrate en proefcrate geslaagd; beide test-runs vonden 0 tests. Linux is nog niet geverifieerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Dit verifieert de basis op Windows, maar niet de platformondersteuning op Linux.

Laatste gerichte F03-verificatie (2026-10-29, Windows): voor `cargo test --locked execution::tests` zijn 20 geslaagde tests gerapporteerd. De 2 genegeerde helpers worden door bijbehorende tests als apart proces gestart; 14 overige tests vallen buiten het filter. De code is nagelezen, inclusief annulering en PID-controles. De tests zijn tijdens deze documentatie-update niet opnieuw uitgevoerd, waarbij de gerapporteerde testresultaten zijn overgenomen.

Eerder op dezelfde datum slaagde de volledige suite met 29 passed en 2 ignored, vóór de latere uitbreidingen. Dat is geen actuele volledige-suitecontrole. Toen faalde `cargo fmt --check` op beide execution-bestanden en faalde Clippy op overbodige haakjes, ongebruikte uitvoeringscode, `duration`, `Cancelled` en een niet meer vervulde `#[expect(dead_code)]` in `src/config.rs`. De waarschuwing voor `Cancelled` is na de implementatie niet meer gerapporteerd. Formatter, Clippy en de volledige testsuite zijn sindsdien niet opnieuw geverifieerd. Linux, shutdown en HTTP-uitvoering zijn niet getest.
