---
feature: F03
tags:
  - commandhub
  - feature-notitie
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

[Featureplan](../docs/features/F03-procesuitvoering.md)

## Wat bouw ik en waarom?

F03 voegt een gedeelde, begrensde procesuitvoerder en `POST /tools/{toolname}` toe voor geconfigureerde tools zonder dynamische parameters. Daarmee kan de server tools gecontroleerd uitvoeren zonder executablekeuze of shellopbouw vanuit clientinvoer. De eerste werkende bouwstenen voor processtart en begrensd gelijktijdig uitlezen zijn aanwezig en gericht getest. De volledige uitvoeringsmethode en HTTP-route bestaan nog niet. De actuele checklist staat in het [featureplan](../docs/features/F03-procesuitvoering.md).

## Hoe werkt de code?

In `src/execution.rs` beschrijft `ExecutionResult` een afgeronde run en onderscheidt `ExecutionError` de uitvoeringsfouten. `find_parameterless_tool` zoekt in de registry en weigert tools met parameters of placeholders. De lifetime `'a` geeft aan dat de gevonden `ToolDefinition` geleend wordt uit de registry.

`ExecutionService` houdt met `Arc<Semaphore>` een gedeelde capaciteit bij. `try_acquire_owned` wijst direct af wanneer de vier permits bezet zijn; een `OwnedSemaphorePermit` geeft de plek automatisch terug wanneer hij wordt gedropt. `create_command` bouwt een Tokio-`Command` met executable en argumenten apart en sluit stdin.

`read_capped` leest een async stream in blokken van 8192 bytes. Een gedeelde semaphore bewaakt het gezamenlijke outputbudget; de permits blijven bij `CapturedOutput` bewaard zolang de bytes in het geheugen staan. `read_process_output` haalt de stdout- en stderr-pipes uit het childproces en gebruikt `tokio::try_join!` om ze tegelijk met hetzelfde budget te lezen. Bij een leesfout geeft de helper die fout terug; de helper coördineert zelf nog niet de volledige levenscyclus, timeout of procesboomopruiming.

`ExecutionService::start_process` reserveert een capaciteit-permit en start het proces met `create_command`. Als het starten mislukt, wordt de permit automatisch vrijgegeven. `kill_on_drop(true)` is ingesteld als vangnet voor het directe childproces; dit bewijst geen beëindiging van de volledige procesboom.

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

De gebruiker heeft de volgende gerichte tests op Windows uitgevoerd en gemeld dat ze slagen:

- De oorspronkelijke tests voor parameterloze toolselectie, onbekende tools, tools met parameters en output binnen/boven de helperlimiet.
- `weigert_extra_uitvoering_bij_volledige_capaciteit`: een extra uitvoering wordt afgewezen wanneer alle permits bezet zijn.
- `geeft_capaciteit_vrij_als_starten_mislukt`: capaciteit komt vrij na een startfout.
- `start_proces_en_wacht_op_succesvolle_afloop`: `git --version` start, stdout en stderr worden met de helper tegelijk gelezen, en het proces eindigt succesvol.
- `begrenz_stdout_en_stderr_met_een_gedeeld_budget`: twee streams delen aantoonbaar één outputbudget.

Dit zijn gerichte tussentijdse controles, geen volledige verificatie van F03. De volledige formatter-, Clippy- en testcontroles zijn voor deze tussenstand niet gemeld. Timeout, outputoverschrijding door een echt proces, annulering en procesboomopruiming zijn nog niet getest.

## Werksessies

### 2026-10-26 — F03 gestart

- Doel: F03 voorbereiden en uitvoeringsafspraken vastleggen.
- Gewijzigd: gedeelde uitvoeringsresultaten en fouttypen, parameterloze toolselectie, command-opbouw, concurrency- en outputbudgethelpers, en vijf unit tests. Documentatie bijgewerkt; branch `feature/F03-procesuitvoering` actief.
- Geleerd: procesboomopruiming moet expliciet per platform worden gerealiseerd en getest; een timeouterror alleen bewijst niet dat subprocessen zijn beëindigd.
- Gecontroleerd: vijf gerichte unit tests slagen volgens gebruiker. De complete procesuitvoering is nog niet aanwezig.
- Open vragen: issue-referentie vastleggen; procescoördinatie, timeout, outputstream-concurrency, procesboomimplementatie en tests op Windows/Linux uitvoeren.

### 2026-10-27 — Eerste procesbouwstenen en iteratieve leerwerkwijze

- Doel: de processtart en het gezamenlijk begrensd lezen van stdout en stderr stapsgewijs opbouwen en testen.
- Gewijzigd: `start_process` reserveert capaciteit en start de geconfigureerde tool; `create_command` gebruikt `kill_on_drop(true)`; `read_process_output` leest beide pipes tegelijk met het gedeelde budget. Tests controleren de capaciteitsgrens, vrijgave bij startfout, een succesvolle `git --version`-run en het gedeelde outputbudget.
- Geleerd: `Option::take()` haalt een pipe uit `Child`; als de test die pipe al heeft weggehaald, kan de helper hem niet opnieuw ophalen. De test hoort de pipes daarom aan de helper over te laten. Ook is een geslaagde read-helper nog geen volledige procescoördinatie.
- Gecontroleerd: de gebruiker heeft de bovenstaande gerichte tests uitgevoerd en gemeld dat ze slagen. Geen volledige suite of Clippy/formattercontrole gemeld.
- Werkwijze: eerst een kleine werkende verticale slice maken, die gericht verifiëren en daarna uitbreiden tot de acceptatiecriteria. Code in korte fragmenten aanbieden met bestand/plek en syntax-uitleg; de eigenaar typt zelf en bevestigt de testuitkomst. Documentatie alleen aanpassen op verzoek.
- Open: resultaatcoördinatie, timeout, echte outputoverschrijding met opruiming, annulering, procesboomgedrag en HTTP-route. Zie ook de [projectbrede leerwerkwijze](../AGENTS.md#leerbegeleiding).

## Vervolg

Maak als volgende stap één kleine executor-methode die de bestaande onderdelen samenbrengt: parameterloze tool selecteren, proces starten, stdout en stderr gelijktijdig begrensd lezen, op het proces wachten en een `ExecutionResult` met uitvoer, exitcode, succes en duur teruggeven. Voeg een gerichte end-to-end unit test toe. Breid daarna de werkende slice uit met timeout, outputoverschrijding, annulering en procesopruiming, en sluit vervolgens de HTTP-route aan. Markeer niets als geverifieerd totdat de betreffende controle is uitgevoerd.
