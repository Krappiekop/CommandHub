# Testplan

Status: het volledige testplan is gepland. F00 is handmatig met een echte MCP-client getest; F02 heeft automatische configuratie- en routetests. Voor F03 meldt de gebruiker dat de gerichte suite 13 tests laat slagen en 1 test negeert. De process-wrap-smoketest toont starten en wachten; procesboomopruiming met subprocessen, annulering, platformmatrix, HTTP-tests en volledige Cargo-controles blijven open.

| Eisen | Bewijs |
| --- | --- |
| R01–R03 | Geldig bestand, onleesbaar bestand, kapotte JSON, duplicaten, ongeldige definities, registratie na herstart zonder rebuild |
| R04–R06 | Alleen geconfigureerde executable met vaste argumenten; gerichte tests voor succes, niet-nul exitcode, startfout, gelijktijdig stdout/stderr, timeout en process-wrap-smoketest zijn door gebruiker gemeld als geslaagd. Procesboombeëindiging bij subprocessen op Windows/Linux blijft te testen |
| R07 | Gedeeld outputbudget voor streams en echte child-output boven 1 MiB, plus capaciteitsgrens/-vrijgave zijn gericht getest volgens gebruiker. Requestbody boven 16 KiB en vijfde gelijktijdige HTTP-call blijven te testen |
| R08–R10 | Verplichte/ontbrekende/verkeerde/onbekende parameters, optie-injectie, paden buiten root, symlinks |
| R11/R16 | F00: handmatige `initialize`, listing/call met MCP Inspector CLI; aanvullende ongeldige protocolrequests en toepasselijke Origin/Host-controles blijven te verifiëren bij integratie |
| R12/R13 | Routertests voor 400/404/413/502/503/504, niet-nul exitcode als HTTP 200, requestcontext en geen gevoelige inhoud in logs |
| R14/R15 | Dezelfde kernscenario's op Windows en Linux; adapters gebruiken dezelfde kern |
| R17 | Schone checkout met README starten; demo met succes, fout, logging en timeout |

Gebruik een klein zelfgeschreven testproces voor gecontroleerde vertraging, output, exitcodes en subprocessen, zodat tests niet onnodig van shellcommando's afhangen. De huidige outputlimiettest gebruikt de testbinary zelf als child; voeg voor procesboomverificatie een child toe die op zijn beurt een subprocess start.

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
