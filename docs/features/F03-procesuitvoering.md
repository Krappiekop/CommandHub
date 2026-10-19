---
id: F03
prioriteit: Must
status: Backlog
milestone: "Uitvoeren"
---

# F03 — Geconfigureerde tool met vaste argumenten uitvoeren

## Doel

Geconfigureerde tool met vaste argumenten uitvoeren. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F02.

## Requirements

R04, R05, R06, R07, R12, R14.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Todo

- [ ] Gedeelde kernresultaten en fouttypen definiëren.
- [ ] Executor maken met losse argumenten, stdinbeleid en omgevingsbeleid.
- [ ] Timeout, outputlimiet en concurrentielimiet implementeren.
- [ ] Annulering, shutdown en procesboomgedrag onderzoeken en implementeren volgens gekozen scope.
- [ ] POST /tools/{toolname} voor tools zonder dynamische parameters aansluiten.
- [ ] Testproces maken voor duur, stdout/stderr, exitcode en subprocessen.
- [ ] Logging en reproduceerbare tests toevoegen.

## Acceptatiecriteria

- [ ] Alleen geconfigureerde executables worden gestart.
- [ ] Stdout/stderr worden gelijktijdig en begrensd gelezen.
- [ ] Timeout, te grote output en volle capaciteit hebben voorspelbaar gedrag.
- [ ] Procesopruiming is getest en platformbeperkingen zijn expliciet.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F03-procesuitvoering.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: nog vast te leggen.
- [Featurenotitie](../../notities/F03-procesuitvoering.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

