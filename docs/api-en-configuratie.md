# API- en configuratiecontract

Status: concept. Alleen GET /status is geïmplementeerd.

## HTTP

| Route | Doel |
| --- | --- |
| GET /status | Lokale beschikbaarheid |
| GET /tools | Toolbeschrijvingen en parameters opvragen |
| POST /tools/{toolname} | Tool uitvoeren met een JSON-object als invoer |

Een optionele GET /tools/{toolname} kan later worden toegevoegd. Niet nodig om de eerste scope groter te maken.

Beoogd uitvoeringsresultaat sluit aan op de opdracht:

```json
{
  "success": true,
  "stdout": "...",
  "stderr": "",
  "exitCode": 0,
  "duration": 12
}
```

duration is milliseconden. exitCode kan null zijn wanneer het OS geen numerieke code levert. Een niet-nul exitcode is een afgeronde uitvoering met success=false; een uitvoeringsfout is een afzonderlijke foutcategorie.

Leg foutcodes en HTTP-statusmapping vóór implementatie vast, inclusief timeout, outputlimiet en capaciteit.

## Configuratie

Het eerste formaat sluit aan op de opdracht: tools met name, description, command, arguments, parameters en timeout. Extra grenzen en padbeleid worden expliciet toegevoegd. Zie config/README.md. Voorbeelden zijn nog geen definitief schema.

Open beslissingen:
- Welke parametertypen ondersteunen we aanvankelijk?
- Zijn placeholders uitsluitend hele argumenten?
- Defaultlimieten versus verplichte limieten per tool.
- Platformspecifieke executablepaden.
- Verboden/onbekende configuratievelden afwijzen.
- Gedrag bij outputlimiet: beëindigen en duidelijke fout, geen stilzwijgende truncatie.

## MCP

De voorbeeld-REST-routes zijn geen volledige MCP-implementatie. Gebruik de officiële SDK en kies in F00 een passende protocolversie en HTTP-transport. Laat zowel REST als MCP dezelfde registry en uitvoeringskern gebruiken.

F00 legt client, protocolversie, initialisatie/onderhandeling, toollisting, toolcalls, schema's en toepasselijke beveiligingsregels vast. De MCP-resultaatstructuur volgt het protocol en is niet simpelweg dezelfde body als REST.

