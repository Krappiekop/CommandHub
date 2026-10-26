# Huidige voortgang

Bijgewerkt: 2026-10-26.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. F01 is lokaal gecontroleerd op Windows: formatter, Clippy, tests (0 tests), build en `GET /status` slagen. Een GitHub Actions-workflow voor Ubuntu en Windows is toegevoegd, maar nog niet op GitHub uitgevoerd.

F02 is gemerged. De hoofdapp leest en valideert de JSON-configuratie bij startup, bouwt een immutable registry en serveert `GET /tools` met toolmetadata en inputschema's. De route laat command en vaste argumenten weg. Configuratie-laadfouten zijn gericht getest; handmatig is bevestigd dat ongeldige configuratie startup stopt met een bruikbare diagnose en dat een aangepaste tool na herstart zonder rebuild zichtbaar wordt. Formatter, Clippy en 14 tests slaagden op Windows. De standaardtimeout is 10 seconden (maximaal 60 seconden); standaardbestand is `config/tools.json`, overschrijfbaar met `COMMANDHUB_CONFIG`. Zie [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

F03 is gestart op branch `feature/F03-procesuitvoering`. De gedeelde uitvoeringsresultaten en fouttypen, parameterloze toolselectie, command-opbouw, semaphore en begrensde outputhelper zijn toegevoegd. Vijf gerichte unit tests voor toolselectie en outputbegrenzing slagen volgens gebruiker. Volledige procesuitvoering, timeout/annulering, procesboomopruiming en de HTTP-route zijn nog niet geïmplementeerd. Zie [F03-featureplan](features/F03-procesuitvoering.md) en [F03-notitie](../notities/F03-procesuitvoering.md).

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, minimale Axum-statusroute met logging, F02-configuratielader, registry, `GET /tools` en tests, eerste F03-uitvoeringshelpers en unit tests, CI-workflow en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. De hoofdapplicatie heeft nog geen complete procesuitvoerder of MCP-integratie.

## Nog open

- F00: review en PR-merge door gebruiker. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: `Cargo.lock` committen en repository-/develop-/GitHub-Project-inrichting afronden. CI op GitHub laten draaien; Linux is lokaal niet gecontroleerd. Status blijft open tot review en merge.

## Volgende stap

Ga verder met de async procesuitvoering: start het proces, lees stdout en stderr gelijktijdig met hetzelfde outputbudget, en behandel timeout, fouten en opruiming. Voeg daarna gerichte tests toe voordat de HTTP-route wordt aangesloten. F00-review en F01-afronding blijven afzonderlijke openstaande projectstappen.

## Verificatie

Op Windows zijn voor F01 `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked` en een HTTP-request naar `/status` uitgevoerd; alles slaagde. De testbuild bevat 0 tests. De server stopte na Ctrl+C; de terminal gaf exitcode 1 voor de interrupt. De workflow `.github/workflows/ci.yml` is toegevoegd, maar nog niet door GitHub uitgevoerd. Voor F00 zijn de Cargo-controles ook op de hoofdcrate en proefcrate geslaagd; beide test-runs vonden 0 tests. Linux is nog niet geverifieerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Dit verifieert de basis op Windows, maar niet de platformondersteuning op Linux.

Voor F03 meldt de gebruiker vijf geslaagde gerichte tests via `cargo test --locked execution::tests`: drie tests voor toolselectie en twee voor output onder/boven de testlimiet. De procesaanroep zelf is nog niet gestart; timeout, outputoverschrijding tijdens procesuitvoering, concurrency, annulering en procesboomopruiming zijn daarmee nog niet aangetoond.
