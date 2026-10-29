# Testplan

Status: het volledige testplan is nog niet aangetoond. F00 is handmatig met een echte MCP-client getest; F02 heeft automatische configuratie- en routetests. Gerapporteerd testresultaat op 2026-10-29: 20 geslaagde execution-tests op Windows, naast 2 genegeerde helpers. De procesboomtests controleren subprocess-PID's; expliciete annulering is getest. Shutdown, future-drop/abort, Linux en HTTP-uitvoeringstests blijven open. Formatter, Clippy en de volledige suite zijn na de uitbreidingen niet opnieuw geverifieerd; eerdere controles en meldingen staan in de [F03-notitie](../notities/F03-procesuitvoering.md).

| Eisen | Bewijs |
| --- | --- |
| R01–R03 | Geldig bestand, onleesbaar bestand, kapotte JSON, duplicaten, ongeldige definities, registratie na herstart zonder rebuild |
| R04–R06 | Tests voor toolselectie, succes, niet-nul exitcode, startfout, gelijktijdig stdout/stderr, timeout en platformwrapper. Windows-PID-controles na timeout, outputoverschrijding, eerder afgerond direct child en expliciete annulering. Een gesloten annuleringskanaal zonder bericht laat uitvoering doorgaan. Gerichte controles zijn als geslaagd gerapporteerd; shutdown, future-drop/abort en Linux-opruiming blijven open |
| R07 | Gedeeld outputbudget voor streams en echte child-output boven 1 MiB, plus capaciteitsgrens/-vrijgave slagen in de Windows-suite. Requestbody boven 16 KiB en vijfde gelijktijdige HTTP-call blijven te testen |
| R08–R10 | Verplichte/ontbrekende/verkeerde/onbekende parameters, optie-injectie, paden buiten root, symlinks |
| R11/R16 | F00: handmatige `initialize`, listing/call met MCP Inspector CLI; aanvullende ongeldige protocolrequests en toepasselijke Origin/Host-controles blijven te verifiëren bij integratie |
| R12/R13 | Routertests voor 400/404/413/502/503/504, niet-nul exitcode als HTTP 200, requestcontext en geen gevoelige inhoud in logs |
| R14/R15 | Dezelfde kernscenario's op Windows en Linux; adapters gebruiken dezelfde kern |
| R17 | Schone checkout met README starten; demo met succes, fout, logging en timeout |

De outputlimiettest en procesboomtests gebruiken de testbinary zelf als child. De subprocess-helper start `ping` op Windows of `sleep` op Unix en schrijft het subprocess-ID naar een tijdelijk bestand. Markers via `--skip` kiezen optionele grote output of vroegtijdige terugkeer van de helper. De tests verwijderen oude bestanden vooraf en gebruiken afzonderlijke namen per scenario en bovenliggende testrun.

Op Windows verzamelt `windows_process_ids` via `tasklist` de PID's en controleert de test dat het subprocess ontbreekt. De annuleringstest gebruikt `tokio::join!` om uitvoering en signaalgever samen te pollen en verstuurt pas een bericht zodra het PID-bestand een geldig getal bevat. Bij timeout/outputoverschrijding/annulering wordt ook gecontroleerd dat alle vier capaciteits-permits weer beschikbaar zijn.

Resterende testbeperkingen: Linux mist een daadwerkelijke PID-controle; Windows gebruikt één momentopname die timinggevoelig kan zijn. De blokkerende `tasklist`-aanroep wordt niet betrouwbaar begrensd door de async-timeout. Verbeter die bewaking en test server-shutdown en het afbreken/droppen van de future afzonderlijk. De tests tonen de benoemde scenario's aan, geen algemene garantie voor elke mogelijke procesboom.

## Controles

cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked

## Meetresultaten

Noteer per feature: datum, platform, commando, resultaat, beperkingen en verwijzing naar test/PR. Een timeouterror alleen bewijst niet dat het proces weg is.

| Datum/platform | Controle | Resultaat en herkomst |
| --- | --- | --- |
| 2026-10-29, Windows | `cargo test --locked execution::tests` | Gerapporteerd: 20 geslaagd; 2 helpers genegeerd in de hoofdrun, 14 overige tests buiten het filter. Actuele code nagelezen |
| 2026-10-29, Windows, vóór uitbreidingen | `cargo test --locked` | Toen 29 passed, 2 ignored; geen actuele volledige-suitecontrole na de latere wijzigingen |
| 2026-10-29, Windows, vóór uitbreidingen | Formatter en Clippy | Toen afgekeurd; niet opnieuw uitgevoerd. Ongebruikte `Cancelled` is inmiddels niet meer gerapporteerd; overige meldingen nog te verifiëren |

Bij deze documentatie-update zijn geen tests uitgevoerd; de opdracht beperkte zich tot documentatie. Linux was niet beschikbaar in deze Windows-sessie; shutdown en HTTP-uitvoering zijn nog niet geïmplementeerd.

## Demo

1. Geldige configuratie laden.
2. Tools ontdekken via HTTP en MCP.
3. Tool uitvoeren en resultaat uitleggen.
4. Ongeldige invoer afwijzen.
5. Timeout en outputlimiet aantonen.
6. Logging laten zien.
7. Nieuwe tool toevoegen, herstarten en zonder rebuild gebruiken.
