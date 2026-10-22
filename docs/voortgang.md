# Huidige voortgang

Bijgewerkt: 2026-10-22.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. F01 is lokaal gecontroleerd op Windows: formatter, Clippy, tests (0 tests), build en `GET /status` slagen. Een GitHub Actions-workflow voor Ubuntu en Windows is toegevoegd, maar nog niet op GitHub uitgevoerd.

F02 is in uitvoering. De hoofdapp leest en valideert de JSON-configuratie bij startup, bouwt een immutable registry en serveert `GET /tools` met toolmetadata en inputschema's. De route laat command en vaste argumenten weg. De gebruiker rapporteert 11 geslaagde tests en heeft `/tools` handmatig gecontroleerd. De standaardtimeout is 10 seconden (maximaal 60 seconden); standaardbestand is `config/tools.json`, overschrijfbaar met `COMMANDHUB_CONFIG`. Nog open: gerichte tests voor configuratiebestand-laadfouten, aantonen dat een nieuwe tool na herstart zonder rebuild zichtbaar wordt, en volledige onafhankelijke verificatie. Zie [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, minimale Axum-statusroute met logging, F02-configuratielader, registry, `GET /tools` en tests, CI-workflow en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. De hoofdapplicatie heeft nog geen procesuitvoerder of MCP-integratie.

## Nog open

- F00: review en PR-merge door gebruiker. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: `Cargo.lock` committen en repository-/develop-/GitHub-Project-inrichting afronden. CI op GitHub laten draaien; Linux is lokaal niet gecontroleerd. Status blijft open tot review en merge.
- F02 resterende laadfouttests en herstartscenario afronden; daarna complete verificatie uitvoeren.

## Volgende stap

Vervolg F02 met tests voor configuratiebestand-laadfouten en bewijs dat een gewijzigde toolconfiguratie na herstart zichtbaar wordt zonder rebuild. Voer daarna volledige formatter-, Clippy- en testcontroles uit. F00-review en F01-afronding blijven openstaande projectstappen. Leg voor F03 nog toegestane executabletypen en uitvoeringsgrenzen vast; zie ook [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

## Verificatie

Op Windows zijn voor F01 `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked` en een HTTP-request naar `/status` uitgevoerd; alles slaagde. De testbuild bevat 0 tests. De server stopte na Ctrl+C; de terminal gaf exitcode 1 voor de interrupt. De workflow `.github/workflows/ci.yml` is toegevoegd, maar nog niet door GitHub uitgevoerd. Voor F00 zijn de Cargo-controles ook op de hoofdcrate en proefcrate geslaagd; beide test-runs vonden 0 tests. Linux is nog niet geverifieerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Dit verifieert de basis op Windows, maar niet de platformondersteuning op Linux.
