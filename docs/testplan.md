# Testplan

Status: het volledige testplan is gepland. F00 heeft handmatig een MCP-proef via een echte client uitgevoerd; er zijn nog geen automatische tests.

| Eisen | Bewijs |
| --- | --- |
| R01–R03 | Geldig bestand, onleesbaar bestand, kapotte JSON, duplicaten, ongeldige definities, registratie na herstart zonder rebuild |
| R04–R06 | Succes, niet-nul exitcode, startfout, stdout/stderr tegelijk, timeout en aantoonbare procesbeëindiging |
| R07 | Output boven grens, request boven grens, meer calls dan capaciteit; geheugen en wachttijd begrensd |
| R08–R10 | Verplichte/ontbrekende/verkeerde/onbekende parameters, optie-injectie, paden buiten root, symlinks |
| R11/R16 | F00: handmatige `initialize`, listing/call met MCP Inspector CLI; aanvullende ongeldige protocolrequests en toepasselijke Origin/Host-controles blijven te verifiëren bij integratie |
| R12/R13 | Routertests, foutmapping, requestcontext en geen gevoelige inhoud in logs |
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
