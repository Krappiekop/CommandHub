# Securityontwerp

Status: configuratievalidatie en parameterloze HTTP-uitvoering zijn geïmplementeerd. De gedeelde executor begrenst output, tijd en capaciteit en ondersteunt expliciete annulering en server-shutdown. Windows-verificatie: 55 geslaagde tests en een echte POST-aanroep. Linux, uitvoeringslogging, echte HTTP-concurrency en formatter-/Clippy-eindcontrole blijven open.

## Vertrouwensgrenzen

De beheerder bepaalt configuratie en executables. Clientinvoer is onbetrouwbaar. Uitvoer van tools is onbetrouwbare inhoud en mag niet als instructie aan de server worden behandeld. Configuratiebeheer heeft invloed op welke programma's de server mag starten.

## Maatregelen per feature

- F02: strikt configuratieschema, unieke namen, bekende placeholders, geldige limieten; geen configuratiewijziging via client.
- F03: executable en argumenten gescheiden; geen automatische shell-dispatch. Scripts zijn alleen toegestaan via een expliciet geconfigureerde interpreter met vaste scriptargumenten. Stdin sluiten. Het childproces erft de omgeving en werkdirectory van de server; de client kan deze niet wijzigen.
- F03: timeout van maximaal 60 seconden (standaard 10 seconden), maximaal 1 MiB gecombineerde stdout/stderr en maximaal vier gelijktijdige uitvoeringen per gedeelde service-instantie zijn in de kern aanwezig. De POST-route heeft een bodylimiet van 16 KiB vóór JSON-deserialisatie; de exacte grens en overschrijding zijn getest. Bij volle capaciteit direct afwijzen. `process-wrap` verzorgt platformbeheer; gerichte Windows-tests controleren het verdwijnen van subprocessen. Linux-verificatie blijft open; deze wordt als laatste uitgevoerd.
- F03: een expliciet `oneshot`-bericht annuleert één uitvoering via de opruimroute; sluiten zonder bericht niet. Server-shutdown gebruikt een gedeelde `watch`-status, weigert nieuwe uitvoeringen en wacht op capaciteitsvrijgave. Ctrl+C is aangesloten. Future-drop/abort is een apart Windows-getest vangnet via `KillOnDrop`, zonder normaal teruggegeven opruimresultaat. Expliciet clientdisconnectbeleid en Linux blijven open.
- F04: typen, vereiste parameters, onbekende parameters en toegestane waarden controleren. Voorkom ook argument-/optie-injectie in de aangeroepen tool.
- F04: paden normaliseren en controleren binnen toegestane roots; symlinks en niet-bestaande paden expliciet behandelen. Canonicalisatie alleen is geen volledige sandbox of oplossing voor races.
- F05: gekozen MCP-transportbeveiliging volgen, waaronder toepasselijke Origin/Host-controles en lokale binding. Niet alle HTTP-clients sturen een Origin; volg de specificatie in plaats van een eigen browserregel.
- HTTP: accepteer alleen `{}` en JSON-contenttype; parameters, onbekende velden en andere JSON-typen worden afgewezen. Executable en argumenten komen uit de registry. Vaste JSON-foutresponses geven geen interne OS-details terug; startfout en foutmapping zijn via HTTP getest.
- Alle: geen geheimen, volledige parameters, executablepaden of stdout/stderr loggen. Uitvoeringslogging moet nog worden toegevoegd en gecontroleerd.

## Afspraken vóór uitvoering

- Native executables en expliciet geconfigureerde interpreters zijn toegestaan; geen automatische `.bat`/`.cmd`- of shell-dispatch.
- Executablepaden mogen absoluut zijn of via de geërfde server-`PATH` worden gevonden.
- De timeout-, output-, request- en concurrentiegrenzen staan vast in het API-contract.
- `process-wrap` 10.0.1 met `tokio1` gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Windows-procescontroles zijn async en begrensd: 3 s per `tasklist`, maximaal 5 s herhaald controleren. Dit is geen sandbox of garantie voor alle procesbomen of blokkerende OS-processtart.
- Publieke hosting en authenticatie vallen buiten de eerste lokale demo.

## Bewijs

Test ongeldige argumenten, padoverschrijding, te grote uitvoer, vastlopend proces, parallelle calls en beëindiging. Documenteer resterende grenzen; claim geen sandbox zonder daadwerkelijke isolatie.
