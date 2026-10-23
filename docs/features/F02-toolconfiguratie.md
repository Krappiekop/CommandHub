---
id: F02
prioriteit: Must
status: Review
milestone: "Ontdekken"
---

# F02 — Configuratie laden en tools ontdekken

## Doel

Configuratie laden en tools ontdekken. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F01; ontwerpinput F00.

## Requirements

R01, R02, R03, R12.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Schema-afspraken

De server leest standaard `config/tools.json` relatief aan de huidige werkdirectory; `COMMANDHUB_CONFIG` kan dit pad overschrijven. De eerste versie ondersteunt `string`-parameters met een `required`-boolean. Placeholders zoals `{path}` zijn alleen toegestaan als volledig argument. `command` en vaste argumenten komen uit beheerdersconfiguratie; `command` mag een absoluut pad zijn of een executable-naam die via de server-`PATH` wordt gevonden. Clients kiezen geen executable. Onbekende velden, ontbrekende verplichte velden, ongeldige waarden, dubbele namen en onbekende placeholders worden afgewezen. Per-tool `timeout` is optioneel: de standaard is 10.000 ms (10 seconden), en een ingestelde waarde moet tussen 1 en 60.000 ms (60 seconden) liggen. Zie ook [het configuratievoorbeeld](../../config/README.md).

De keuze tussen absoluut pad en zoeken via `PATH` is gemaakt; de toegestane executabletypen (waaronder eventueel Windows `.bat`/`.cmd`) moeten vóór procesuitvoering in F03 worden besloten. Het volledige contract en de scheiding met F03 staan in [API en configuratie](../api-en-configuratie.md).

## Todo

- [x] Eerste JSON-configuratieschema met tools, parameters en timeout vastleggen.
- [x] JSON bij startup laden en onbekende velden afwijzen.
- [x] Dubbele namen, timeoutgrenzen en placeholders valideren.
- [x] Tests toevoegen voor ontbrekende verplichte velden en ongeldige JSON.
- [x] Immutable registry bouwen en via Axum-state beschikbaar maken.
- [x] GET /tools met beschrijvingen en inputschema's toevoegen.
- [x] Een gerichte routetest toevoegen die de responsevorm en het weglaten van het command controleert.
- [ ] Config-bestand laadfouten gericht testen.
- [x] Config-bestand laadfouten gericht testen.

## Acceptatiecriteria

- [ ] Ongeldige configuratie verhindert serverstart met bruikbare diagnose.
- [x] Ongeldige configuratie verhindert serverstart met bruikbare diagnose.
- [x] Nieuwe tool wordt na herstart zichtbaar zonder rebuild.
- [x] Dubbele namen en onbekende placeholders worden afgewezen.
- [x] Placeholders worden alleen als volledig argument vervangen; clients kunnen de executable niet kiezen.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F02-toolconfiguratie.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: gebruiker rapporteert 11 geslaagde tests en bevestigt dat `GET /tools` werkt; zie de F02-notitie.
- Tests/handmatige verificatie: gerichte configuratietests en controles van startup, herstart zonder rebuild en `GET /tools`; zie de F02-notitie.
- [Featurenotitie](../../notities/F02-toolconfiguratie.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [x] Relevante controles geslaagd.
- [x] Documentatie en notitie bijgewerkt.
- [x] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.
