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

## Werkaanpak

Bouw F03 iteratief: begin met een kleine werkende verticale slice die een geconfigureerde parameterloze tool start, stdout en stderr gelijktijdig begrensd uitleest, op het proces wacht en een resultaat oplevert. Breid die slice daarna gecontroleerd uit met timeout, outputoverschrijding, annulering en procesopruiming, en sluit pas daarna de HTTP-route aan. Behoud de vastgestelde veiligheidsgrenzen in elke stap. Werk met korte codefragmenten met een exacte bestandslocatie en begrijpelijke uitleg; voer per stap een gerichte controle uit voordat de volgende stap begint. De algemene leerwerkwijze staat in [AGENTS.md](../../AGENTS.md#leerbegeleiding).

## Vastgestelde afspraken

- F03 voert alleen tools zonder dynamische parameters uit. F04 voegt parametercontrole en placeholdervervanging toe.
- De beheerder configureert de executable en vaste argumenten. Native executables zijn toegestaan; scripts alleen via een expliciet geconfigureerde interpreter met vaste argumenten. Geen automatische shell-, `.bat`- of `.cmd`-dispatch.
- Het childproces erft de omgeving en werkdirectory van de server. De client kan deze niet wijzigen.
- De standaardtimeout is 10.000 ms en het maximum 60.000 ms.
- stdout en stderr worden samen begrensd op 1 MiB. Bij overschrijding wordt de procesboom beëindigd en volgt een expliciete fout.
- Requestbody maximaal 16 KiB; maximaal vier gelijktijdige uitvoeringen. Bij volle capaciteit direct afwijzen.
- `process-wrap` 10.0.1 is toegevoegd met de Tokio-feature. De executor gebruikt Unix `ProcessGroup`, Windows `JobObject` en `KillOnDrop`. Een smoke-test start en wacht op een wrapped proces; volledige opruiming van subprocessen is nog niet bewezen.
- HTTP-mapping: 400 invoerfout, 404 onbekende tool, 413 te grote requestbody, 503 volle capaciteit, 504 timeout, 502 outputlimiet. Een proces met niet-nul exitcode geeft HTTP 200 met `success: false`.

## Todo

- [x] Gedeelde kernresultaten en fouttypen definiëren.
- [x] Command-opbouw met geconfigureerde executable, losse vaste argumenten, gesloten stdin en gepipete stdout/stderr voorbereiden.
- [x] Basisexecutor voor parameterloze tools maken; executable en vaste argumenten komen uit de beheerdersconfiguratie en de serveromgeving/werkdirectory worden geërfd. Procesboomopruiming en annulering blijven open.
- [x] Semaphore voor maximaal vier gelijktijdige uitvoeringen en een helper voor de gecombineerde outputlimiet toevoegen.
- [x] Timeout, outputlimiet en concurrentielimiet implementeren voor de directe child; volledige procesboomopruiming blijft open.
- [x] `process-wrap` integreren voor Unix-procesgroepen en Windows Job Objects.
- [ ] Procesboomopruiming bij een child met eigen subprocess testen bij timeout en outputoverschrijding op Windows en Linux; annulering en shutdown afhandelen.
- [ ] POST /tools/{toolname} voor tools zonder dynamische parameters aansluiten.
- [x] Gerichte tests voor succesvolle/niet-succesvolle exit, timeout, outputlimiet en capaciteitsvrijgave toevoegen.
- [ ] Reproduceerbaar testproces maken dat ook een subprocess start en het procesboomgedrag controleert.
- [ ] Logging en reproduceerbare tests toevoegen.

## Acceptatiecriteria

- [ ] Alleen geconfigureerde executables voor parameterloze tools worden gestart; geen clientgestuurde command-, argument- of shellinvoer.
- [x] Stdout/stderr worden gelijktijdig en begrensd gelezen.
- [x] Timeout, te grote output en volle capaciteit hebben voorspelbaar gedrag voor de geteste directe child; procesboomgedrag met subprocessen blijft open.
- [ ] Procesopruiming is getest en platformbeperkingen zijn expliciet.

## Bewijs en koppelingen

- GitHub issue: nog vast te leggen.
- Branch: feature/F03-procesuitvoering.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: gebruiker meldt dat `cargo test --locked execution::tests` slaagt: 13 passed, 1 ignored, 14 filtered. Dit is een gerichte test op Windows; volledige Cargo-controles, subprocessopruiming op beide platforms en HTTP-integratie zijn nog niet geverifieerd. Zie de [F03-notitie](../../notities/F03-procesuitvoering.md).
- [Featurenotitie](../../notities/F03-procesuitvoering.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.
