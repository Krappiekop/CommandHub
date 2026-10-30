# Testplan

Status bijgewerkt: 2026-10-30. F00 is handmatig met een echte MCP-client getest; F02 bevat configuratie- en ontdekkingstests. De actuele Windows-suite is gerapporteerd met **55 passed, 0 failed, 2 ignored**. HTTP-uitvoering, shutdown, future-drop/abort en begrensde Windows-procescontrole zijn toegevoegd. Logging, echte HTTP-concurrency, eindcontroles en Linux blijven open.

| Eisen | Aanwezig bewijs en resterende controles |
| --- | --- |
| R01–R03 | Configuratievalidatie en laadfouten; ontdekking via HTTP; eerdere handmatige herstartcontrole zonder rebuild |
| R04–R06 | Toolselectie, succes, niet-nul exit, startfout, output, timeout, annulering en shutdown. Windows-PID-controles na stoproutes, future-drop en taakafbreking. Linux blijft open |
| R07 | Gedeeld outputbudget, child-output boven 1 MiB, capaciteitsgrens/vrijgave. HTTP-body exact 16 KiB toegestaan en groter afgewezen. HTTP 503 bij vier vastgehouden permits; vijf echte gelijktijdige requests blijven te testen |
| R08–R10 | F04: dynamische parameters, typen, onbekende velden, optie-injectie en padbeleid |
| R11/R16 | F00: MCP Inspector CLI; protocol-/Origin-/Host-controles bij F05 verder verifiëren |
| R12 | Windows-routetests voor succes, niet-nul exit als 200, 400/404/413/500/502/503/504 en shutdownafwijzing; echte POST naar geconfigureerde tool geslaagd |
| R13 | Vaste foutcodes en geen OS-details in JSON getest. Uitvoeringslogging zonder gevoelige inhoud blijft open |
| R14/R15 | Kern is transportonafhankelijk; HTTP-adapter gebruikt gedeelde state. Dezelfde kernscenario's en PID-opruiming op Linux blijven open |
| R17 | README bevat start- en POST-voorbeeld; complete demo en schone-checkoutcontrole volgen bij oplevering |

## Testindeling

Alle vijf testbestanden staan in `src/tests/`: `config_tests.rs`, `execution_tests.rs`, `routes_tests.rs`, `execution_controller_tests.rs` en `execution_model_tests.rs`. De productiemodules koppelen ze met `#[cfg(test)]`, `#[path]` en `mod tests;`. De logische paden en toegang tot private code blijven gelijk. Deze indeling is geen zelfstandige integratiecrate onder `tests/` in de projecthoofdmap.

De twee genegeerde helpers worden bewust via de testbinary als child gestart met `--exact`, `--ignored` en `--nocapture`. De helper voor subprocessen start `ping` op Windows of `sleep` op Unix en schrijft een PID-bestand. Elke test gebruikt een eigen naam en verwijdert oude bestanden vooraf. Markers via `--skip` kiezen grote output of vroegtijdig afgerond child. Annulering en shutdown wachten met `join!` op een leesbaar PID.

## Procesopruiming

Op Windows verzamelt `windows_process_ids` via een Tokio-proces de PID's uit `tasklist`. Die aanroep heeft 3 s timeout en `kill_on_drop(true)`. `assert_windows_process_stopped` vraagt de lijst herhaald op met 50 ms tussenpozen en maximaal 5 s totale wachttijd. Zo wordt vertraagde beëindiging opgevangen. Dit is geen harde tijdsgarantie voor eventuele blokkerende OS-processtart.

Tests bewaken ook capaciteitsvrijgave. Shutdown weigert nieuwe reserveringen en wacht op alle permits. Een afzonderlijke test houdt één permit vast om te bewijzen dat shutdown wacht. Future-drop en Tokio-`abort` zijn getest; na `abort` wordt de taak afgewacht voordat capaciteit en PID worden beoordeeld. Normaal stoppen gebruikt expliciete annulering/shutdown; drop/abort is een vangnet zonder teruggegeven uitvoerings-/opruimresultaat.

Linux mist nog een daadwerkelijke processtatuscontrole en is niet uitgevoerd. Deze verificatie wordt als laatste behandeld via een nog te controleren CI-opzet; lokaal is geen Linux-omgeving beschikbaar.

## HTTP-controles

De tests lopen rechtstreeks door de router via `oneshot`, zonder TCP-poort. Ze controleren succes, resultaat-JSON, niet-nul exit, ongeldige JSON/contenttype/invoer, onbekende tool, parameters afwijzen, exacte bodygrens en overschrijding, startfout, timeout, outputlimiet, capaciteit en shutdownafwijzing. De capaciteitstest staat in `execution_tests.rs` om private permits te reserveren; hij start geen vier echte processen.

Handmatig is een echte POST uitgevoerd naar `/tools/git_version` met JSON `{}`. Resultaat: succes, Git 2.55.0.windows.3 op stdout, lege stderr, exitcode 0, duration 61 ms. Statusroute en Ctrl+C zonder actieve tools zijn ook gecontroleerd. Ctrl+C tijdens actieve HTTP-uitvoering en vijf echte gelijktijdige requests blijven open.

## Controles

```powershell
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

## Meetresultaten

| Datum/platform | Controle | Resultaat |
| --- | --- | --- |
| 2026-10-30, Windows | `cargo test --locked` na testverplaatsing | Gerapporteerd: 55 passed, 0 failed, 2 ignored, 0 filtered out, 2,89 s |
| 2026-10-30, Windows | Echte HTTP POST | `git_version`: succes, exitcode 0, duration 61 ms |
| 2026-10-30, Windows | Status en Ctrl+C | Status werkt; terminalprompt keert terug, zonder actieve tools |
| 2026-10-29, Windows | Eerdere formatter/Clippy-controles | Toen afgekeurd; geen actuele hercontrole na alle uitbreidingen |

Bij deze documentatie-update zijn geen tests uitgevoerd; de opdracht beperkte zich tot documentatie. De actuele code en testindeling zijn nagelezen en de gerapporteerde resultaten zijn vastgelegd. Formatter en Clippy moeten nog worden uitgevoerd; Linux en uitvoeringslogging zijn niet geverifieerd.

## Demo

1. Configuratie laden en tools ontdekken.
2. Parameterloze tool uitvoeren via HTTP.
3. Resultaat en niet-nul exit uitleggen.
4. Invoerfouten, timeout, outputlimiet en volle capaciteit tonen.
5. Logging en shutdown tijdens uitvoering aantonen zodra gecontroleerd.
6. MCP-demo bij F05; complete demo en herstart zonder rebuild bij oplevering.
