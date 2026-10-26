---
id: F03
prioriteit: Must
status: In uitvoering
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

## Vastgestelde afspraken

- F03 voert alleen tools zonder dynamische parameters uit. F04 voegt parametercontrole en placeholdervervanging toe.
- De beheerder configureert de executable en vaste argumenten. Native executables zijn toegestaan; scripts alleen via een expliciet geconfigureerde interpreter met vaste argumenten. Geen automatische shell-, `.bat`- of `.cmd`-dispatch.
- Het childproces erft de omgeving en werkdirectory van de server. De client kan deze niet wijzigen.
- De standaardtimeout is 10.000 ms en het maximum 60.000 ms.
- stdout en stderr worden samen begrensd op 1 MiB. Bij overschrijding wordt de procesboom beëindigd en volgt een expliciete fout.
- Requestbody maximaal 16 KiB; maximaal vier gelijktijdige uitvoeringen. Bij volle capaciteit direct afwijzen.
- Procesboomopruiming wordt voor Windows en Linux onderzocht en getest, met platformgeschikte procesgroepen/Windows Job Objects.
- HTTP-mapping: 400 invoerfout, 404 onbekende tool, 413 te grote requestbody, 503 volle capaciteit, 504 timeout, 502 outputlimiet. Een proces met niet-nul exitcode geeft HTTP 200 met `success: false`.

## Todo

- [x] Gedeelde kernresultaten en fouttypen definiëren.
- [x] Command-opbouw met geconfigureerde executable, losse vaste argumenten, gesloten stdin en gepipete stdout/stderr voorbereiden.
- [ ] Executor maken en de procesuitvoering integreren met het vastgestelde omgevingsbeleid.
- [x] Semaphore voor maximaal vier gelijktijdige uitvoeringen en een helper voor de gecombineerde outputlimiet toevoegen.
- [ ] Timeout, outputlimiet en concurrentielimiet implementeren.
- [ ] Annulering, shutdown en procesboomgedrag onderzoeken en implementeren volgens gekozen scope.
- [ ] POST /tools/{toolname} voor tools zonder dynamische parameters aansluiten.
- [ ] Testproces maken voor duur, stdout/stderr, exitcode en subprocessen.
- [ ] Logging en reproduceerbare tests toevoegen.

## Acceptatiecriteria

- [ ] Alleen geconfigureerde executables voor parameterloze tools worden gestart; geen clientgestuurde command-, argument- of shellinvoer.
- [ ] Stdout/stderr worden gelijktijdig en begrensd gelezen.
- [ ] Timeout, te grote output en volle capaciteit hebben voorspelbaar gedrag.
- [ ] Procesopruiming is getest en platformbeperkingen zijn expliciet.

## Bewijs en koppelingen

- GitHub issue: nog vast te leggen.
- Branch: feature/F03-procesuitvoering.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: vijf gerichte unit tests geslaagd volgens gebruiker; zie de F03-notitie. Procesintegratie is nog niet getest.
- [Featurenotitie](../../notities/F03-procesuitvoering.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

