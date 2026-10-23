# API- en configuratiecontract

Status: ontwerp voor F02. Alleen GET /status is geïmplementeerd.

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

Het eerste formaat sluit aan op het configuratievoorbeeld uit de opdracht: tools met `name`, `description`, `command`, `arguments`, `parameters` en optioneel `timeout`. `parameters` ondersteunen aanvankelijk `name`, `type` (`string`) en `required`. Zie [configuratievoorbeelden](../config/README.md).

### Eerste schema-keuzes

De casus geeft richting en voorbeelden, maar legt niet alle randgevallen of limietwaarden vast. Voor de eerste versie gelden deze keuzes:

- De server leest standaard `config/tools.json`, relatief aan de huidige werkdirectory. De beheerder kan dit pad overschrijven met de omgevingsvariabele `COMMANDHUB_CONFIG`.
- Ondersteun als parametertype alleen `string` en een `required`-boolean. Breid dit pas uit als een feature dat nodig maakt.
- Een placeholder heeft de vorm `{naam}` en mag alleen een volledig argument innemen, bijvoorbeeld `"{path}"`. Plak geen vaste tekst en placeholder aan elkaar; zo blijft elk procesargument een afzonderlijk item.
- `command` en de vaste argumenten komen uitsluitend uit beheerdersconfiguratie. `command` mag een absoluut pad of een executable-naam zijn die via de `PATH` van het serverproces wordt gevonden. Een HTTP- of MCP-client kan geen executable kiezen of de `PATH` aanpassen.
- Wijs onbekende configuratievelden af, zodat typefouten niet ongemerkt worden genegeerd. Wijs ook ontbrekende verplichte velden, ongeldige waarden, dubbele toolnamen en placeholders die niet bij een parameter horen af.
- `timeout` is optioneel per tool en wordt uitgedrukt in milliseconden. Zonder deze instelling geldt een standaardtimeout van 10.000 ms (10 seconden). Een ingestelde timeout moet groter zijn dan 0 en mag maximaal 60.000 ms (60 seconden) zijn; grotere waarden worden afgewezen.
- De configuratie blijft na startup onveranderlijk; een gewijzigde configuratie wordt actief na een herstart, niet via hot-reload.

De beheerder stelt `command` en de vaste argumenten in; de server laat de client dit niet overschrijven. De beheerder kan dus een absoluut pad of een naam gebruiken die via de server-`PATH` wordt gevonden. Platformgeschikte paden en de toegestane executabletypen (waaronder eventueel Windows `.bat`/`.cmd`) moeten vóór procesuitvoering in F03 worden vastgelegd.

### Uitvoeringsgrenzen voor F03

F03 legt ook concrete grenzen vast voor uitvoer, requestgrootte en gelijktijdige uitvoering. Bij overschrijding van de outputlimiet moet de uitvoering worden beëindigd en als expliciete fout worden gemeld; afgekorte uitvoer mag niet op een normale succesvolle uitvoering lijken. De concrete grenswaarden moeten vóór implementatie van F03 worden vastgesteld.

## MCP

De voorbeeld-REST-routes zijn geen volledige MCP-implementatie. Gebruik de officiële SDK en kies in F00 een passende protocolversie en HTTP-transport. Laat zowel REST als MCP dezelfde registry en uitvoeringskern gebruiken.

F00 legt client, protocolversie, initialisatie/onderhandeling, toollisting, toolcalls, schema's en toepasselijke beveiligingsregels vast. De MCP-resultaatstructuur volgt het protocol en is niet simpelweg dezelfde body als REST.
