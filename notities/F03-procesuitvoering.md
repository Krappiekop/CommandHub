---
feature: F03
tags:
  - commandhub
  - feature-notitie
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

[Featureplan](../docs/features/F03-procesuitvoering.md)

## Wat bouw ik en waarom?

F03 voegt een gedeelde, begrensde procesuitvoerder en `POST /tools/{toolname}` toe voor geconfigureerde tools zonder dynamische parameters. Daarmee kan de server tools gecontroleerd uitvoeren zonder executablekeuze of shellopbouw vanuit clientinvoer. De eerste uitvoeringsstructuren en helpers zijn toegevoegd; het proces wordt nog niet gestart en de HTTP-route bestaat nog niet. De actuele checklist staat in het [featureplan](../docs/features/F03-procesuitvoering.md).

## Hoe werkt de code?

In `src/execution.rs` beschrijft `ExecutionResult` een afgeronde run en onderscheidt `ExecutionError` de uitvoeringsfouten. `find_parameterless_tool` zoekt in de registry en weigert tools met parameters of placeholders. De lifetime `'a` geeft aan dat de gevonden `ToolDefinition` geleend wordt uit de registry.

`ExecutionService` houdt met `Arc<Semaphore>` een gedeelde capaciteit bij. `try_acquire_owned` wijst direct af wanneer de vier permits bezet zijn; een `OwnedSemaphorePermit` geeft de plek automatisch terug wanneer hij wordt gedropt. `create_command` bouwt een Tokio-`Command` met executable en argumenten apart en sluit stdin.

`read_capped` leest een async stream in blokken van 8192 bytes. Een gedeelde semaphore bewaakt het gezamenlijke outputbudget; de permits blijven bij `CapturedOutput` bewaard zolang de bytes in het geheugen staan. De helper geeft een fout bij overschrijding, maar beëindigt zelf geen proces. De procescoördinatie, gelijktijdig lezen van beide streams en opruiming moeten nog worden gebouwd.

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

Vijf gerichte tests zijn door de gebruiker uitgevoerd op Windows met `cargo test --locked execution::tests`; alle vijf slaagden. Ze testen parameterloze toolselectie, weigering van onbekende tools en tools met parameters, plus output onder en boven de helperlimiet. Dit test nog geen gestart proces, timeout of procesopruiming. De volledige formatter-, Clippy- en testcontroles zijn voor deze tussenstand nog niet gemeld.

## Werksessies

### 2026-10-26 — F03 gestart

- Doel: F03 voorbereiden en uitvoeringsafspraken vastleggen.
- Gewijzigd: gedeelde uitvoeringsresultaten en fouttypen, parameterloze toolselectie, command-opbouw, concurrency- en outputbudgethelpers, en vijf unit tests. Documentatie bijgewerkt; branch `feature/F03-procesuitvoering` actief.
- Geleerd: procesboomopruiming moet expliciet per platform worden gerealiseerd en getest; een timeouterror alleen bewijst niet dat subprocessen zijn beëindigd.
- Gecontroleerd: vijf gerichte unit tests slagen volgens gebruiker. De complete procesuitvoering is nog niet aanwezig.
- Open vragen: issue-referentie vastleggen; procescoördinatie, timeout, outputstream-concurrency, procesboomimplementatie en tests op Windows/Linux uitvoeren.

## Vervolg

Vervolg met het starten en beheren van het childproces: stdout en stderr gelijktijdig begrensd lezen, timeout en outputoverschrijding afhandelen en het proces opruimen. Daarna volgen de HTTP-route en integratietests. Markeer niets als geverifieerd totdat de betreffende controle is uitgevoerd.

