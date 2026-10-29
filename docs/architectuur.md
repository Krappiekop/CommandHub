# Architectuur

Status: de statusroute, configuratielader, immutable registry en `GET /tools` zijn geïmplementeerd. F03 bevat een executor voor parameterloze tools met begrensde output, timeout en expliciete annulering per uitvoering. `process-wrap` 10.0.1 gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Windows-tests controleren subprocess-PID's na timeout, outputoverschrijding en annulering, ook bij een eerder afgerond direct child. Linux-verificatie, coördinatie van actieve uitvoeringen bij server-shutdown, de HTTP-uitvoerroute en MCP-adapter blijven open.

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
| Process executor | Proces, uitvoer, tijdslimiet, capaciteit, expliciete annulering en opruimen beheren; subprocessopruiming is in gerichte Windows-tests gecontroleerd, Linux en shutdown blijven open |
| Logging | Context, uitkomst en duur vastleggen zonder gevoelige inhoud |

## Structuur groeit met de features

De basis bevat src/main.rs, routes.rs, controllers/ en models/. Voeg services/, config/, registry/, validation/ en mcp/ pas toe wanneer daar werkende code voor komt. Vermijd lege lagen zonder functie.

Models voor HTTP/MCP zijn transportmodellen. Gedeelde kernresultaten en fouttypen mogen niet afhankelijk zijn van Axum of HTTP-statuscodes.

## State en concurrency

De registry wordt via `Arc` gedeeld. `ExecutionService` begrenst gelijktijdige uitvoeringen met `Arc<Semaphore>` op vier per service-instantie en wijst volle capaciteit direct af. De uitvoeringsservice is nog niet in de serverstate opgenomen; de toekomstige adapters moeten dezelfde instantie delen om de grens serverbreed te bewaken.

`execute_parameterless` delegeert naar `execute_parameterless_with_cancellation` en houdt zelf een ongebruikte `oneshot`-zender vast. De annuleerbare methode ontvangt `oneshot::Receiver<()>`; `tokio::select!` wacht op de uitvoering met timeout of een expliciet `Ok(())`-signaal. Het sluiten van het kanaal zonder bericht annuleert niet. Bij annulering wordt `stop_child` afgewacht voordat `Cancelled` wordt teruggegeven; een opruimfout kan die fout vervangen. Dit blijft onafhankelijk van HTTP en MCP.

De huidige Ctrl+C-handler in `main.rs` regelt Axum-graceful-shutdown, maar signaleert nog geen actieve tooluitvoeringen. Serverbrede annulering en het afwachten van hun opruiming zijn de volgende ontwerpstap. Het droppen/afbreken van een uitvoeringsfuture is een apart scenario en is nog niet getest.

## Fouten

Maak onderscheid tussen ongeldige invoer, onbekende tool, capaciteitsgebrek, startfout, niet-succesvolle exitcode, timeout, outputlimiet, annulering en opruimfout. HTTP en MCP vertalen dezelfde kernfout naar hun eigen contract; de transportmapping voor annulering is nog niet vastgesteld.

## Platformbeleid

Cross-platform code betekent niet dat elke executable op ieder OS bestaat. Lever platformgeschikte configuratievoorbeelden. [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 gebruikt Tokio `ProcessGroup` op Unix en `JobObject` op Windows. De testhelper start `ping` op Windows of `sleep` op Unix. Alleen Windows heeft nu een controle op het verdwijnen van het subprocess via `tasklist`. `stop_child` slaat `kill()` over bij een afgerond direct child; het bijbehorende Windows-scenario slaagt, maar bewijst geen identiek gedrag op Linux of welke wrapperactie voor opruiming zorgt.
