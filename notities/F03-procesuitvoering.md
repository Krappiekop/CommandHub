---
feature: F03
tags:
  - commandhub
  - feature-notitie
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

[Featureplan](../docs/features/F03-procesuitvoering.md)

## Wat bouw ik en waarom?

F03 voegt een gedeelde, begrensde procesuitvoerder en `POST /tools/{toolname}` toe voor geconfigureerde tools zonder dynamische parameters. Daarmee kan de server tools gecontroleerd uitvoeren zonder executablekeuze of shellopbouw vanuit clientinvoer. De actuele checklist staat in het [featureplan](../docs/features/F03-procesuitvoering.md). De implementatie is nog niet begonnen.

## Hoe werkt de code?

De bestaande code heeft nog geen procesuitvoerder. Tijdens de implementatie leg ik hier de gegevensstroom, resultaat- en fouttypen, en gebruikte Rust-concepten uit met links naar de actuele bronbestanden.

## Keuzes en alternatieven

- F03 voert uitsluitend tools zonder dynamische parameters uit; placeholdervervanging hoort bij F04.
- Native executables zijn toegestaan. Scripts starten alleen via een expliciet geconfigureerde interpreter met vaste argumenten; geen automatische shell- of `.bat`/`.cmd`-dispatch.
- Het childproces erft de omgeving en werkdirectory van de server. De client kan deze niet instellen.
- Limieten: 10.000 ms standaardtimeout, maximaal 60.000 ms timeout, 1 MiB gecombineerde stdout/stderr, 16 KiB requestbody en vier gelijktijdige uitvoeringen. Volle capaciteit wordt direct afgewezen.
- Timeout, outputoverschrijding en annulering moeten de procesboom op Windows en Linux beëindigen. Platformimplementatie en tests zijn nog open.
- HTTP-foutmapping: 400 invoerfout, 404 onbekende tool, 413 request te groot, 503 capaciteit vol, 504 timeout, 502 outputlimiet. Een niet-nul exitcode geeft HTTP 200 met `success: false`.

Deze afspraken staan ook in [API en configuratie](../docs/api-en-configuratie.md) en het [securityontwerp](../docs/security.md).

## Problemen en oplossingen

Noteer concrete oorzaak, oplossing en hoe de oplossing is gecontroleerd.

## Verificatie

Nog niet uitgevoerd. Noteer per controle commando, platform, resultaat en beperkingen.

## Werksessies

### 2026-10-26 — F03 gestart

- Doel: F03 voorbereiden en uitvoeringsafspraken vastleggen.
- Gewijzigd: documentatiecontract bijgewerkt; branch `feature/F03-procesuitvoering` actief.
- Geleerd: procesboomopruiming moet expliciet per platform worden gerealiseerd en getest; een timeouterror alleen bewijst niet dat subprocessen zijn beëindigd.
- Gecontroleerd: huidige branch en schone werkboom gecontroleerd. Er is nog geen F03-code of gedragstest.
- Open vragen: issue-referentie vastleggen; procesboomimplementatie en tests op Windows/Linux uitvoeren.

## Vervolg

Begin met de gedeelde uitvoeringsresultaten en fouttypen volgens het [featureplan](../docs/features/F03-procesuitvoering.md); daarna volgen executor, HTTP-route en tests. Markeer niets als geverifieerd totdat de betreffende controle is uitgevoerd.

