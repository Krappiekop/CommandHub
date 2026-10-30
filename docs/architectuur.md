# Architectuur

Status: status, toolontdekking en parameterloze uitvoering via HTTP zijn aangesloten. `AppState` deelt registry en uitvoeringsservice. De kern ondersteunt timeout, outputbudget, capaciteit, expliciete annulering en shutdown. Windows-tests controleren subprocessopruiming na deze stoproutes, future-drop en Tokio-taakafbreking. Linux en MCP-integratie blijven open.

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
| Process executor | Proces, uitvoer, tijdslimiet, capaciteit, expliciete annulering en opruimen beheren; subprocessopruiming en shutdown zijn in Windows-tests gecontroleerd; Linux blijft open |
| Logging | Context, uitkomst en duur vastleggen zonder gevoelige inhoud |

## Structuur groeit met de features

De basis bevat src/main.rs, routes.rs, controllers/ en models/. Voeg services/, config/, registry/, validation/ en mcp/ pas toe wanneer daar werkende code voor komt. Vermijd lege lagen zonder functie.

Models voor HTTP/MCP zijn transportmodellen. Gedeelde kernresultaten en fouttypen mogen niet afhankelijk zijn van Axum of HTTP-statuscodes.

## State en concurrency

De server maakt één `AppState` met `Arc<ToolRegistry>` en `Arc<ExecutionService>`. Klonen van die state kopieert gedeelde verwijzingen, geen registries of executors. `FromRef<AppState> for Arc<ToolRegistry>` laat de ontdekcontroller alleen de registry opvragen. De uitvoercontroller gebruikt de volledige state. Eén gedeelde executor bewaakt serverbreed vier uitvoeringsplekken en wijst volle capaciteit direct af.

`execute_parameterless` delegeert naar de annuleerbare methode. Die wacht met `tokio::select!` op uitvoering met timeout, een expliciet `oneshot`-bericht of een wijziging van het shutdown-`watch`-kanaal. Alleen een expliciet bericht annuleert via `oneshot`; sluiten zonder bericht niet. De shutdownontvanger wordt vóór processtart aangemaakt om een wijziging tijdens toelating niet te missen.

`try_acquire` controleert de shutdownstatus en houdt de leesbewaker vast tijdens capaciteitsreservering. `begin_shutdown` bewaart `true` via `send_replace`, ook zonder ontvangers. Nieuwe uitvoeringen krijgen daarna `Cancelled`. `shutdown().await` signaleert shutdown en wacht op alle vier permits. Dat bewijst afronding van de uitvoeringsfutures en hun opruimpogingen, maar verzamelt geen opruimfouten en is op zichzelf geen processtatuscontrole.

Bij Ctrl+C roept `main.rs` eerst `begin_shutdown` aan. Axum sluit graceful af; daarna wacht `shutdown().await` op capaciteitsvrijgave. Pas vervolgens wordt een eventuele serverfout doorgegeven. Normaal stoppen verloopt via expliciete annulering of shutdown. Future-drop en `JoinHandle::abort` gebruiken `KillOnDrop` als vangnet; daarbij kan geen normaal uitvoeringsresultaat of afgewachte opruimfout worden teruggegeven. Dit vangnet is getest op Windows, nog niet op Linux.

## HTTP-adapter en tests

De uitvoercontroller vangt JSON-extractiefouten op, accepteert uitsluitend een leeg object en kiest daarna de tool uit de registry. `DefaultBodyLimit` begrenst deze POST-route op 16 KiB vóór JSON-deserialisatie. `ExecutionResponse::from` vertaalt het kernresultaat naar JSON met `exitCode` en milliseconden. De kern kent geen HTTP-statuscodes; de controller bepaalt foutmapping en vaste teksten zonder OS-details.

Tests staan fysiek in `src/tests/`, maar blijven via `#[cfg(test)]` en `#[path = "..."]` submodules van hun oorspronkelijke module. Daardoor blijven private toegang, `super`-imports en helperselecties zoals `execution::tests::subprocess_helper_bij_timeout` geldig. Dit zijn geen zelfstandige integratiecrates onder een projectmap `tests/`.

## Fouten

Maak onderscheid tussen ongeldige invoer, onbekende tool, capaciteitsgebrek, startfout, niet-succesvolle exitcode, timeout, outputlimiet, annulering en opruimfout. HTTP vertaalt dezelfde kernfout naar vaste statuscodes en JSON-foutcodes. Annulering/shutdown wordt HTTP 503; interne procesbeheerfouten worden 500. MCP-mapping volgt in F05. Clientdisconnects zijn nog niet als expliciet annuleringsbeleid gekoppeld.

## Platformbeleid

Cross-platform code betekent niet dat elke executable op ieder OS bestaat. Lever platformgeschikte configuratievoorbeelden. [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 gebruikt Tokio `ProcessGroup` op Unix en `JobObject` op Windows. De testhelper start `ping` op Windows of `sleep` op Unix. Alleen Windows heeft nu een controle op het verdwijnen van het subprocess via `tasklist`. `stop_child` slaat `kill()` over bij een afgerond direct child; het bijbehorende Windows-scenario slaagt, maar bewijst geen identiek gedrag op Linux of welke wrapperactie voor opruiming zorgt.
