# Testplan

Status: het volledige testplan is gepland. F00 is handmatig met een echte MCP-client getest; F02 heeft automatische configuratie- en routetests. Voor F03 zijn vijf unit tests voor toolselectie en outputlimiet geslaagd; procesintegratie en de overige uitvoeringsscenario's zijn nog niet getest.

| Eisen | Bewijs |
| --- | --- |
| R01–R03 | Geldig bestand, onleesbaar bestand, kapotte JSON, duplicaten, ongeldige definities, registratie na herstart zonder rebuild |
| R04–R06 | Alleen geconfigureerde executable met vaste argumenten; succes, niet-nul exitcode, startfout, stdin gesloten, stdout/stderr tegelijk, timeout en aantoonbare procesboombeëindiging op Windows/Linux |
| R07 | Gecombineerde stdout/stderr boven 1 MiB, requestbody boven 16 KiB, vijfde gelijktijdige call bij capaciteit vier; output en request worden begrensd en capaciteit wordt direct afgewezen |
| R08–R10 | Verplichte/ontbrekende/verkeerde/onbekende parameters, optie-injectie, paden buiten root, symlinks |
| R11/R16 | F00: handmatige `initialize`, listing/call met MCP Inspector CLI; aanvullende ongeldige protocolrequests en toepasselijke Origin/Host-controles blijven te verifiëren bij integratie |
| R12/R13 | Routertests voor 400/404/413/502/503/504, niet-nul exitcode als HTTP 200, requestcontext en geen gevoelige inhoud in logs |
| R14/R15 | Dezelfde kernscenario's op Windows en Linux; adapters gebruiken dezelfde kern |
| R17 | Schone checkout met README starten; demo met succes, fout, logging en timeout |

Gebruik een klein zelfgeschreven testproces voor gecontroleerde vertraging, output en exitcodes, zodat tests niet onnodig van shellcommando's afhangen.

## Controles

cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked

## Meetresultaten

Noteer per feature: datum, platform, commando, resultaat, beperkingen en verwijzing naar test/PR. Een timeouterror alleen bewijst niet dat het proces weg is.

## Demo

1. Geldige configuratie laden.
2. Tools ontdekken via HTTP en MCP.
3. Tool uitvoeren en resultaat uitleggen.
4. Ongeldige invoer afwijzen.
5. Timeout en outputlimiet aantonen.
6. Logging laten zien.
7. Nieuwe tool toevoegen, herstarten en zonder rebuild gebruiken.
