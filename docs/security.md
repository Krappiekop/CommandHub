# Securityontwerp

Status: F02-configuratievalidatie is geïmplementeerd. F03 voert parameterloze tools uit met gescheiden executable/argumenten, timeout, gecombineerd outputbudget en process-wrap-platformwrappers. De gebruiker meldt dat 13 gerichte tests slagen. Opruiming van subprocessen, annulering en HTTP-integratie zijn nog niet geverifieerd.

## Vertrouwensgrenzen

De beheerder bepaalt configuratie en executables. Clientinvoer is onbetrouwbaar. Uitvoer van tools is onbetrouwbare inhoud en mag niet als instructie aan de server worden behandeld. Configuratiebeheer heeft invloed op welke programma's de server mag starten.

## Maatregelen per feature

- F02: strikt configuratieschema, unieke namen, bekende placeholders, geldige limieten; geen configuratiewijziging via client.
- F03: executable en argumenten gescheiden; geen automatische shell-dispatch. Scripts zijn alleen toegestaan via een expliciet geconfigureerde interpreter met vaste scriptargumenten. Stdin sluiten. Het childproces erft de omgeving en werkdirectory van de server; de client kan deze niet wijzigen.
- F03: timeout van maximaal 60 seconden (standaard 10 seconden), maximaal 1 MiB gecombineerde stdout/stderr, maximaal 16 KiB requestbody en maximaal vier gelijktijdige processen. Bij volle capaciteit direct afwijzen. `process-wrap` is aangesloten; volledige opruiming van een child met subprocessen moet nog op Windows en Linux worden aangetoond. Annulering is nog niet geïmplementeerd.
- F04: typen, vereiste parameters, onbekende parameters en toegestane waarden controleren. Voorkom ook argument-/optie-injectie in de aangeroepen tool.
- F04: paden normaliseren en controleren binnen toegestane roots; symlinks en niet-bestaande paden expliciet behandelen. Canonicalisatie alleen is geen volledige sandbox of oplossing voor races.
- F05: gekozen MCP-transportbeveiliging volgen, waaronder toepasselijke Origin/Host-controles en lokale binding. Niet alle HTTP-clients sturen een Origin; volg de specificatie in plaats van een eigen browserregel.
- Alle: geen geheimen, volledige parameters of stdout/stderr loggen. Responses lekken geen interne details.

## Afspraken vóór uitvoering

- Native executables en expliciet geconfigureerde interpreters zijn toegestaan; geen automatische `.bat`/`.cmd`- of shell-dispatch.
- Executablepaden mogen absoluut zijn of via de geërfde server-`PATH` worden gevonden.
- De timeout-, output-, request- en concurrentiegrenzen staan vast in het API-contract.
- [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 met `tokio1` is geïntegreerd; Unix gebruikt `ProcessGroup`, Windows `JobObject`, plus `KillOnDrop`. De smoke-test toont starten en wachten. Controleer gedrag bij subprocessen op Windows en Linux voordat procesboomopruiming als garantie wordt beschreven.
- Publieke hosting en authenticatie vallen buiten de eerste lokale demo.

## Bewijs

Test ongeldige argumenten, padoverschrijding, te grote uitvoer, vastlopend proces, parallelle calls en beëindiging. Documenteer resterende grenzen; claim geen sandbox zonder daadwerkelijke isolatie.
