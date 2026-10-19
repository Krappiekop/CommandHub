# Securityontwerp

Status: ontwerp; maatregelen zijn nog niet gerealiseerd.

## Vertrouwensgrenzen

De beheerder bepaalt configuratie en executables. Clientinvoer is onbetrouwbaar. Uitvoer van tools is onbetrouwbare inhoud en mag niet als instructie aan de server worden behandeld. Configuratiebeheer heeft invloed op welke programma's de server mag starten.

## Maatregelen per feature

- F02: strikt configuratieschema, unieke namen, bekende placeholders, geldige limieten; geen configuratiewijziging via client.
- F03: executable en argumenten gescheiden, geen shell; stdin sluiten; minimale uitvoeringsrechten; expliciet werkmap- en omgevingsbeleid.
- F03: timeout, bounded output, capaciteitslimiet, annulering en procesopruiming. Procesboomgedrag per OS onderzoeken.
- F04: typen, vereiste parameters, onbekende parameters en toegestane waarden controleren. Voorkom ook argument-/optie-injectie in de aangeroepen tool.
- F04: paden normaliseren en controleren binnen toegestane roots; symlinks en niet-bestaande paden expliciet behandelen. Canonicalisatie alleen is geen volledige sandbox of oplossing voor races.
- F05: gekozen MCP-transportbeveiliging volgen, waaronder toepasselijke Origin/Host-controles en lokale binding. Niet alle HTTP-clients sturen een Origin; volg de specificatie in plaats van een eigen browserregel.
- Alle: geen geheimen, volledige parameters of stdout/stderr loggen. Responses lekken geen interne details.

## Afspraken vóór uitvoering

- Ondersteunde executabletypen, inclusief Windows .bat/.cmd: besluit nog nodig; geen willekeurige scripts toestaan.
- Executablepaden en PATH-beleid: besluit nog nodig.
- Timeout-, output-, request- en concurrentiegrenzen: meetbare waarden in configuratiecontract vastleggen.
- Procesboomcontrole: keuze voor process groups / Windows Job Objects of expliciete beperkte scope onderbouwen.
- Publieke hosting en authenticatie vallen buiten de eerste lokale demo.

## Bewijs

Test ongeldige argumenten, padoverschrijding, te grote uitvoer, vastlopend proces, parallelle calls en beëindiging. Documenteer resterende grenzen; claim geen sandbox zonder daadwerkelijke isolatie.

