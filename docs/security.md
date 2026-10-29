# Securityontwerp

Status: F02-configuratievalidatie is geïmplementeerd. F03 voert parameterloze tools uit met gescheiden executable/argumenten, timeout, gecombineerd outputbudget, expliciete annulering en process-wrap-platformwrappers. Gerapporteerd testresultaat op 2026-10-29: 20 geslaagde execution-tests op Windows, naast 2 genegeerde helpers. Subprocess-PID's worden na timeout, outputoverschrijding en annulering gecontroleerd, ook bij een eerder afgerond direct child. Linux, server-shutdown, het afbreken van de uitvoeringsfuture en HTTP-uitvoering blijven open. Formatter, Clippy en de volledige suite zijn na de uitbreidingen niet opnieuw geverifieerd.

## Vertrouwensgrenzen

De beheerder bepaalt configuratie en executables. Clientinvoer is onbetrouwbaar. Uitvoer van tools is onbetrouwbare inhoud en mag niet als instructie aan de server worden behandeld. Configuratiebeheer heeft invloed op welke programma's de server mag starten.

## Maatregelen per feature

- F02: strikt configuratieschema, unieke namen, bekende placeholders, geldige limieten; geen configuratiewijziging via client.
- F03: executable en argumenten gescheiden; geen automatische shell-dispatch. Scripts zijn alleen toegestaan via een expliciet geconfigureerde interpreter met vaste scriptargumenten. Stdin sluiten. Het childproces erft de omgeving en werkdirectory van de server; de client kan deze niet wijzigen.
- F03: timeout van maximaal 60 seconden (standaard 10 seconden), maximaal 1 MiB gecombineerde stdout/stderr en maximaal vier gelijktijdige uitvoeringen per gedeelde service-instantie zijn in de kern aanwezig. De requestbodygrens van 16 KiB is afgesproken maar nog niet aangesloten. Bij volle capaciteit direct afwijzen. `process-wrap` verzorgt platformbeheer; gerichte Windows-tests controleren het verdwijnen van subprocessen. Linux-verificatie blijft open.
- F03: een expliciet `oneshot`-bericht annuleert één uitvoering via de bestaande opruimroute; een gesloten kanaal zonder bericht annuleert niet. Dit is nog niet gekoppeld aan server-shutdown of HTTP-clientdisconnects. Het droppen van een future is niet hetzelfde als gecontroleerde annulering met afgewachte opruiming en moet apart worden getest.
- F04: typen, vereiste parameters, onbekende parameters en toegestane waarden controleren. Voorkom ook argument-/optie-injectie in de aangeroepen tool.
- F04: paden normaliseren en controleren binnen toegestane roots; symlinks en niet-bestaande paden expliciet behandelen. Canonicalisatie alleen is geen volledige sandbox of oplossing voor races.
- F05: gekozen MCP-transportbeveiliging volgen, waaronder toepasselijke Origin/Host-controles en lokale binding. Niet alle HTTP-clients sturen een Origin; volg de specificatie in plaats van een eigen browserregel.
- Alle: geen geheimen, volledige parameters of stdout/stderr loggen. Responses lekken geen interne details.

## Afspraken vóór uitvoering

- Native executables en expliciet geconfigureerde interpreters zijn toegestaan; geen automatische `.bat`/`.cmd`- of shell-dispatch.
- Executablepaden mogen absoluut zijn of via de geërfde server-`PATH` worden gevonden.
- De timeout-, output-, request- en concurrentiegrenzen staan vast in het API-contract.
- [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 met `tokio1` is geïntegreerd; Unix gebruikt `ProcessGroup`, Windows `JobObject`, plus `KillOnDrop`. De gerichte Windows-scenario's zijn als geslaagd gerapporteerd, maar vormen geen garantie voor alle procesbomen of Linux. De PID-controle gebruikt één momentopname en een blokkerende `tasklist`-aanroep die de async-testtimeout niet betrouwbaar begrenst; robuuster testen blijft open.
- Publieke hosting en authenticatie vallen buiten de eerste lokale demo.

## Bewijs

Test ongeldige argumenten, padoverschrijding, te grote uitvoer, vastlopend proces, parallelle calls en beëindiging. Documenteer resterende grenzen; claim geen sandbox zonder daadwerkelijke isolatie.
