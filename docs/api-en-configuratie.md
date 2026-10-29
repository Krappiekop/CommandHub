# API- en configuratiecontract

Status: `GET /status` en `GET /tools` zijn geïmplementeerd. Het uitvoeringscontract hieronder is vastgesteld voor F03; `POST /tools/{toolname}` is nog niet geïmplementeerd.

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

Voor F03 geldt deze foutmapping:

| Situatie | HTTP-status | Gedrag |
| --- | --- | --- |
| Ongeldige JSON of ongeldige invoer | 400 | Geen proces starten |
| Onbekende tool | 404 | Geen proces starten |
| Requestbody groter dan 16 KiB | 413 | Request afwijzen vóór JSON-deserialisatie |
| Capaciteit van vier uitvoeringen bereikt | 503 | Direct afwijzen, niet in een wachtrij plaatsen |
| Timeout | 504 | Procesboom beëindigen |
| Gecombineerde stdout/stderr groter dan 1 MiB | 502 | Procesboom beëindigen; geen afgekapt succesresultaat teruggeven |
| Proces gestart met niet-nul exitcode | 200 | Normaal uitvoeringsresultaat met `success: false` en de exitcode |

Een requestbody voor F03 is een JSON-object; gebruik `{}` voor een parameterloze tool. Toolparameters en placeholders worden pas in F04 uitgevoerd.

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

De beheerder stelt `command` en de vaste argumenten in; de server laat de client dit niet overschrijven. De beheerder kan dus een absoluut pad of een naam gebruiken die via de server-`PATH` wordt gevonden. F03 start native executables. Een script mag alleen worden uitgevoerd wanneer de beheerder de interpreter expliciet als `command` configureert en het script als vast argument opgeeft. De server start niet automatisch een shell en dispatcht niet impliciet naar Windows `.bat`- of `.cmd`-bestanden.

### Uitvoeringsgrenzen voor F03

F03 hanteert maximaal 1 MiB gecombineerde stdout en stderr, maximaal 16 KiB requestbody en maximaal vier gelijktijdige uitvoeringen. Een volle capaciteit wordt direct afgewezen. Bij overschrijding van de outputlimiet wordt de procesboom beëindigd en als expliciete fout gemeld; afgekorte uitvoer mag niet op een normale succesvolle uitvoering lijken. De standaardtimeout blijft 10.000 ms; de configureerbare bovengrens blijft 60.000 ms.

Het childproces erft de omgeving en werkdirectory van het serverproces. De client kan deze niet instellen. Beheer van de serveromgeving is daarom onderdeel van het vertrouwensbeleid; omgevingswaarden worden niet gelogd.

Timeout en outputoverschrijding worden in de huidige F03-uitvoermethode afgehandeld; de gebruiker meldt dat de gerichte tests hiervoor slagen. [`process-wrap`](https://docs.rs/process-wrap/) 10.0.1 met feature `tokio1` is geïntegreerd: Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. De wrapper-smoketest toont starten en wachten, maar procesboomopruiming van subprocessen is nog niet getest of gegarandeerd. Annulering en HTTP-integratie zijn eveneens nog open.

## MCP

De voorbeeld-REST-routes zijn geen volledige MCP-implementatie. Gebruik de officiële SDK en kies in F00 een passende protocolversie en HTTP-transport. Laat zowel REST als MCP dezelfde registry en uitvoeringskern gebruiken.

F00 legt client, protocolversie, initialisatie/onderhandeling, toollisting, toolcalls, schema's en toepasselijke beveiligingsregels vast. De MCP-resultaatstructuur volgt het protocol en is niet simpelweg dezelfde body als REST.
