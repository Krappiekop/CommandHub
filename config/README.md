# Configuratievoorbeelden

Deze documentatie beschrijft het afgesproken eerste JSON-formaat voor F02. De basis leest nog geen configuratiebestand; het voorbeeld is dus nog niet door code gevalideerd.

Bij startup leest de server standaard `config/tools.json`, relatief aan de huidige werkdirectory. De beheerder kan een andere bestandslocatie instellen met de omgevingsvariabele `COMMANDHUB_CONFIG`.

Een tool heeft een naam, beschrijving, executable (`command`), lijst losse argumenten en parameters. Een placeholder verwijst naar een parameter en staat als volledig argument. `timeout` is optioneel, in milliseconden, met een standaard van 10.000 ms en een maximum van 60.000 ms.

`command` kan een absoluut pad zijn of een executable-naam, zoals `git`, die via de `PATH` van het serverproces wordt gevonden. De configuratiebeheerder bepaalt deze waarde; een client kan die niet aanleveren of wijzigen.

```json
{
  "tools": [
    {
      "name": "list_files",
      "description": "List files in a directory",
      "command": "ls",
      "arguments": ["-la", "{path}"],
      "parameters": [
        {
          "name": "path",
          "type": "string",
          "required": true
        }
      ]
    },
    {
      "name": "git_status",
      "description": "Show the status of a Git repository",
      "command": "git",
      "arguments": ["-C", "{repository}", "status"],
      "parameters": [
        {
          "name": "repository",
          "type": "string",
          "required": true
        }
      ],
      "timeout": 5000
    }
  ]
}
```

`timeout` weglaten betekent 10 seconden. Een ingestelde timeout moet groter zijn dan 0 en maximaal 60 seconden zijn. De voorbeelden volgen de casus en zijn bedoeld om het configureerbare model te tonen. Ze zijn nog niet geïmplementeerd of volledig beveiligd: F02 moet ze nog strikt inlezen en valideren. De genoemde `ls`-opdracht is niet op elk platform beschikbaar; kies bij gebruik een platformgeschikte executable.

De client levert alleen parameterwaarden en kan `command` of vaste argumenten niet wijzigen. De toegestane executabletypen (waaronder eventueel Windows `.bat`/`.cmd`) en uitvoer- en capaciteitsgrenzen moeten vóór generieke uitvoering in F03 worden vastgelegd.
