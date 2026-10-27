# Huidige voortgang

Bijgewerkt: 2026-10-27.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. F01 is lokaal gecontroleerd op Windows: formatter, Clippy, tests (0 tests), build en `GET /status` slagen. Een GitHub Actions-workflow voor Ubuntu en Windows is toegevoegd, maar nog niet op GitHub uitgevoerd.

F02 is gemerged. De hoofdapp leest en valideert de JSON-configuratie bij startup, bouwt een immutable registry en serveert `GET /tools` met toolmetadata en inputschema's. De route laat command en vaste argumenten weg. Configuratie-laadfouten zijn gericht getest; handmatig is bevestigd dat ongeldige configuratie startup stopt met een bruikbare diagnose en dat een aangepaste tool na herstart zonder rebuild zichtbaar wordt. Formatter, Clippy en 14 tests slaagden op Windows. De standaardtimeout is 10 seconden (maximaal 60 seconden); standaardbestand is `config/tools.json`, overschrijfbaar met `COMMANDHUB_CONFIG`. Zie [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

F03 is actief op branch `feature/F03-procesuitvoering`. `ExecutionService::execute_parameterless` voert de normale route uit: toolselectie, processtart, gelijktijdig begrensd lezen van stdout/stderr, wachten en samenstellen van `ExecutionResult`. De gebruiker meldt dat gerichte tests voor succes, niet-nul exitcode, timeout, capaciteit vrijgeven na fouten en outputoverschrijding door een echt childproces slagen. De directe-child opruiming is geen volledige procesboomopruiming; subprocessen, annulering, HTTP-route en volledige Cargo-controles blijven open. `process-wrap` is onderzocht als kandidaat voor Unix-procesgroepen en Windows Job Objects, maar nog niet toegevoegd of geïntegreerd. Zie [F03-featureplan](features/F03-procesuitvoering.md) en [F03-notitie](../notities/F03-procesuitvoering.md).

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, minimale Axum-statusroute met logging, F02-configuratielader, registry, `GET /tools` en tests, F03-uitvoeringsslice voor parameterloze tools met gerichte tests voor normale exit, timeout en outputlimiet, CI-workflow en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. De hoofdapplicatie heeft nog geen HTTP-uitvoerroute of MCP-integratie.

## Nog open

- F00: review en PR-merge door gebruiker. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: `Cargo.lock` committen en repository-/develop-/GitHub-Project-inrichting afronden. CI op GitHub laten draaien; Linux is lokaal niet gecontroleerd. Status blijft open tot review en merge.

## Volgende stap

Onderzoek de actuele API/features en MSRV van `process-wrap` als kandidaat; voeg die dependency pas toe wanneer de keuze bevestigd is. Integreer daarna de procesboomwrapper en test een child die zelf een subprocess start bij timeout en outputoverschrijding op Windows en Linux. Werk vervolgens annulering/shutdown af en sluit daarna de HTTP-route aan. F00-review en F01-afronding blijven afzonderlijke openstaande projectstappen. De leerwerkwijze voor volgende chats staat in [AGENTS.md](../AGENTS.md#leerbegeleiding).

## Verificatie

Op Windows zijn voor F01 `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked` en een HTTP-request naar `/status` uitgevoerd; alles slaagde. De testbuild bevat 0 tests. De server stopte na Ctrl+C; de terminal gaf exitcode 1 voor de interrupt. De workflow `.github/workflows/ci.yml` is toegevoegd, maar nog niet door GitHub uitgevoerd. Voor F00 zijn de Cargo-controles ook op de hoofdcrate en proefcrate geslaagd; beide test-runs vonden 0 tests. Linux is nog niet geverifieerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Dit verifieert de basis op Windows, maar niet de platformondersteuning op Linux.

Voor F03 heeft de gebruiker gerichte tests uitgevoerd en succes gemeld voor toolselectie/outputhelpers, capaciteit en vrijgave na startfout, succesvolle en niet-succesvolle procesresultaten, timeout met capaciteitsvrijgave en outputoverschrijding door een childproces met capaciteitsvrijgave. Er zijn geen volledige formatter-, Clippy- of testsuite-resultaten gemeld. Procesboomopruiming bij subprocessen, annulering, Windows/Linux-matrix en HTTP-integratie zijn niet aangetoond.
