---
id: F04
prioriteit: Must
status: Backlog
milestone: "Gecontroleerde invoer"
---

# F04 — Dynamische parameters en padbeleid

## Doel

Dynamische parameters en padbeleid. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F03.

## Requirements

R07, R08, R09, R10.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Todo

- [ ] Ondersteunde parametertypen en validatieregels vastleggen.
- [ ] Vereiste, onbekende en ongeldige parameters controleren.
- [ ] Placeholders omzetten naar afzonderlijke argumenten.
- [ ] Optie-injectie per toegestane tool voorkomen.
- [ ] Padbeleid, canonicalisatie en symlinkgedrag implementeren.
- [ ] Requestgrootte begrenzen en grensgevallen testen.

## Acceptatiecriteria

- [ ] Ongeldige invoer start geen proces.
- [ ] Client kan geen executable of ongeconfigureerde opties kiezen.
- [ ] Paden buiten toegestane roots worden afgewezen.
- [ ] Geldige parameters leveren de gedocumenteerde argumenten op.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F04-parameters.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: nog vast te leggen.
- [Featurenotitie](../../notities/F04-parameters.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

