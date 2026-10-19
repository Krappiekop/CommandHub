---
id: F02
prioriteit: Must
status: Backlog
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

## Todo

- [ ] Definitief configuratieschema met parameters en uitvoeringsgrenzen beschrijven.
- [ ] JSON laden en onbekende/ontbrekende/ongeldige velden afwijzen.
- [ ] Unieke namen, limieten en placeholders valideren.
- [ ] Immutable registry bouwen en via state beschikbaar maken.
- [ ] GET /tools met beschrijvingen en inputschema's toevoegen.
- [ ] Gerichte configuratie- en routetests toevoegen.

## Acceptatiecriteria

- [ ] Ongeldige configuratie verhindert serverstart met bruikbare diagnose.
- [ ] Nieuwe tool wordt na herstart zichtbaar zonder rebuild.
- [ ] Dubbele namen en onbekende placeholders worden afgewezen.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F02-toolconfiguratie.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: nog vast te leggen.
- [Featurenotitie](../../notities/F02-toolconfiguratie.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

