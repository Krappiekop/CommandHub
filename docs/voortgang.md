# Huidige voortgang

Bijgewerkt: 2026-10-30.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. F01 is lokaal gecontroleerd op Windows: formatter, Clippy, tests (0 tests), build en `GET /status` slagen. Een GitHub Actions-workflow voor Ubuntu en Windows is toegevoegd, maar nog niet op GitHub uitgevoerd.

F02 is gemerged. De hoofdapp leest en valideert de JSON-configuratie bij startup, bouwt een immutable registry en serveert `GET /tools` met toolmetadata en inputschema's. De route laat command en vaste argumenten weg. Configuratie-laadfouten zijn gericht getest; handmatig is bevestigd dat ongeldige configuratie startup stopt met een bruikbare diagnose en dat een aangepaste tool na herstart zonder rebuild zichtbaar wordt. Formatter, Clippy en 14 tests slaagden op Windows. De standaardtimeout is 10 seconden (maximaal 60 seconden); standaardbestand is `config/tools.json`, overschrijfbaar met `COMMANDHUB_CONFIG`. Zie [F02-featureplan](features/F02-toolconfiguratie.md) en [F02-notitie](../notities/F02-toolconfiguratie.md).

F03 blijft In uitvoering op de eerder vastgelegde branch `feature/F03-procesuitvoering` (geen Git-controle uitgevoerd). De kern ondersteunt begrensde procesuitvoering, expliciete annulering en shutdown via een `watch`-signaal. `AppState` deelt registry en executor via `Arc`; Ctrl+C signaleert shutdown en daarna wordt capaciteitsvrijgave afgewacht. `POST /tools/{toolname}` is op Windows geïmplementeerd en geverifieerd, inclusief alleen `{}` als invoer, 16 KiB bodylimiet en vaste JSON-foutresponses. Gerapporteerd: 55 geslaagde tests, 2 genegeerde helpers; een echte POST naar `git_version` gaf succes, exitcode 0 en 61 ms. Windows-tests controleren subprocessen ook na shutdown, future-drop en taakafbreking. Zie [F03-featureplan](features/F03-procesuitvoering.md) en [F03-notitie](../notities/F03-procesuitvoering.md).

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, Axum-statusroute, configuratielader, registry, ontdekking en uitvoering via HTTP, gedeelde `AppState`, uitvoeringskern met `process-wrap`, shutdown en automatische tests onder `src/tests/`. Verder de CI-workflow en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. MCP-integratie ontbreekt nog in de hoofdapplicatie.

## Nog open

- F00: review en PR-merge. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: `Cargo.lock` committen en repository-/develop-/GitHub-Project-inrichting afronden. CI op GitHub laten draaien; Linux is lokaal niet gecontroleerd. Status blijft open tot review en merge.
- F03: uitvoeringslogging zonder gevoelige inhoud, integratietest met vijf echte gelijktijdige HTTP-verzoeken en Ctrl+C tijdens actieve HTTP-uitvoering. Formatter en Clippy opnieuw uitvoeren en eventuele meldingen oplossen. Linux-procescontroles en daadwerkelijke Linux-verificatie worden als laatste behandeld. Future-drop/abort is alleen op Windows gecontroleerd; gecontroleerde annulering/shutdown blijft de normale stoproute.

## Volgende stap

Begin met het todo-punt uitvoeringslogging: leg toolnaam, foutcategorie/uitkomst en duur vast zonder parameterwaarden, executablepaden of stdout/stderr. Daarna de gelijktijdige HTTP-uitvoeringen en resterende Windows-eindcontroles; als laatste Linux via een nog te controleren CI-opzet. Er is lokaal geen Linux-omgeving beschikbaar. Benoem per leerstap het behandelde todo-punt en wanneer een onderdeel is geverifieerd. Geef volledige functies wanneer dat het invoeren duidelijker maakt, met exact bestand en plaats. Bestanden lezen is toegestaan; wijzigingen en andere uitvoering alleen op expliciet verzoek. Geen Git-commando's uitvoeren. Documentatie blijft algemeen geformuleerd. Zie [AGENTS.md](../AGENTS.md#leerbegeleiding).

## Verificatie

Op Windows zijn voor F01 `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, `cargo build --locked` en een HTTP-request naar `/status` uitgevoerd; alles slaagde. De testbuild bevat 0 tests. De server stopte na Ctrl+C; de terminal gaf exitcode 1 voor de interrupt. De workflow `.github/workflows/ci.yml` is toegevoegd, maar nog niet door GitHub uitgevoerd. Voor F00 zijn de Cargo-controles ook op de hoofdcrate en proefcrate geslaagd; beide test-runs vonden 0 tests. Linux is nog niet geverifieerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Dit verifieert de basis op Windows, maar niet de platformondersteuning op Linux.

Actuele F03-verificatie (2026-10-30, Windows): voor `cargo test --locked` is na de testverplaatsing gerapporteerd: 55 passed, 0 failed, 2 ignored, 0 filtered out, 2,89 s. De genegeerde functies zijn helpers die door andere tests als child worden gestart. Alle vijf testmodules zijn via `#[path]` gekoppeld aan bestanden in `src/tests/`; de logische modulepaden zijn behouden.

De statusroute werkt met gedeelde state; Ctrl+C gaf de terminalprompt terug (controle zonder actieve tools). Een echte `POST /tools/git_version` met `{}` gaf `success: true`, stdout `git version 2.55.0.windows.3\n`, lege stderr, exitcode 0 en duration 61 ms. De HTTP-tests bewaken onder meer 400/404/413/500/502/503/504, niet-nul exit als HTTP 200, precies 16 KiB en grotere bodies, parameters afwijzen en shutdownafwijzing. De 503-capaciteitstest houdt vier permits vast; dit is nog geen test met vijf echte gelijktijdige requests.

Formatter en Clippy zijn na de uitbreidingen niet opnieuw geverifieerd. Eerdere afkeur op 2026-10-29 is historische informatie, geen actuele diagnose. De volledige testsuite slaagt inmiddels volgens het bovenstaande resultaat. Linux is niet uitgevoerd. Tijdens deze documentatie-update zijn uitsluitend documentatiebestanden aangepast; geen Cargo- of Git-commando's uitgevoerd.
