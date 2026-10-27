# Architectuur

Status: de statusroute, configuratielader, immutable registry en `GET /tools` zijn geïmplementeerd. F03 bevat een executor voor parameterloze tools met begrensde output en timeout, maar platformbrede procesboomopruiming en de HTTP-uitvoerroute zijn nog open. `process-wrap` is onderzocht als kandidaat voor Unix-procesgroepen en Windows Job Objects; de crate is nog niet toegevoegd of geïntegreerd. De MCP-adapter is nog niet in de hoofdapplicatie geïmplementeerd.

## Beoogde gegevensstroom

HTTP-handler of MCP-adapter → gedeelde toolservice → registry → parameter- en beleidsvalidatie → process executor → uitvoeringsresultaat → transportresponse.

Bij het starten: configuratie laden → configuratie valideren → registry bouwen → gedeelde state samenstellen → server starten.

## Onderdelen

| Module | Verantwoordelijkheid |
| --- | --- |
| HTTP/routes en controllers | Requests uitlezen en HTTP-responses maken |
| MCP-adapter | Protocolafhandeling via SDK en vertaling naar gedeelde kern |
| Toolservice | Opzoeken, valideren en uitvoering coördineren |
| Registry | Gevalideerde tooldefinities op naam beschikbaar maken |
| Config loader | JSON lezen en controleren; ongeldige configuratie stopt startup |
| Validation | Parameters en uitvoeringsbeleid controleren |
| Process executor | Proces, uitvoer, tijdslimiet, capaciteit en opruimen beheren; F03 implementeert de normale uitvoering en directe-child-timeout/outputfoutafhandeling, platformbrede boomopruiming blijft open |
| Logging | Context, uitkomst en duur vastleggen zonder gevoelige inhoud |

## Structuur groeit met de features

De basis bevat src/main.rs, routes.rs, controllers/ en models/. Voeg services/, config/, registry/, validation/ en mcp/ pas toe wanneer daar werkende code voor komt. Vermijd lege lagen zonder functie.

Models voor HTTP/MCP zijn transportmodellen. Gedeelde kernresultaten en fouttypen mogen niet afhankelijk zijn van Axum of HTTP-statuscodes.

## State en concurrency

Een immutable registry kan via Arc gedeeld worden. De executor krijgt een expliciete capaciteitslimiet, bijvoorbeeld via een semaphore. Een volle capaciteit krijgt een gedocumenteerde afwijzing of begrensde wachttijd; geen onbegrensde wachtrij.

## Fouten

Maak onderscheid tussen ongeldige invoer, onbekende tool, capaciteitsgebrek, startfout, niet-succesvolle exitcode, timeout, outputlimiet en opruimfout. HTTP en MCP vertalen dezelfde kernfout naar hun eigen contract.

## Platformbeleid

Cross-platform code betekent niet dat elke executable op ieder OS bestaat. Lever platformgeschikte configuratievoorbeelden. Leg procesboom- en afsluitgedrag per OS vast en toets dit expliciet. De kandidaat [`process-wrap`](https://docs.rs/process-wrap/) heeft een Tokio-API met Unix `ProcessGroup` en Windows `JobObject` wrappers. Houd de platformkeuze afgebakend en test op beide platforms een proces dat zelf een subprocess start; de crate is nog niet geïntegreerd en is geen bewijs van correcte opruiming.
