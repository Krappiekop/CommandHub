# Plan van aanpak

Status: concept vóór realisatie.

## Doel

Ik bouw een generieke server waarmee een HTTP- of MCP-client vooraf geconfigureerde commandline-tools kan ontdekken en uitvoeren. Nieuwe tools moeten zonder opnieuw compileren kunnen worden toegevoegd.

## Technologie

Rust met Axum voor HTTP, Tokio voor async verwerking, Serde/serde_json voor JSON, tracing voor logging en de officiële Rust MCP SDK voor het protocol. De SDK-versie en features worden in F00 onderzocht en daarna vastgelegd.

Motivatie: ik heb al ervaring met Laravel en .NET en wil mij verdiepen in ownership, expliciete foutafhandeling, concurrency en procesbeheer. Zie ADR-001.

## Eerste oplevering

JSON-configuratie bij opstarten, registry, HTTP-toolroutes, gecontroleerde parameters, begrensde procesuitvoering, MCP via HTTP, logging, voorbeeldtools en demonstraties op Windows en Linux. Lokaal beheer is het uitgangspunt; publieke hosting is geen onderdeel van de eerste oplevering.

Geen database of frontend nodig. Hot-reload, YAML, jobqueue, WebSockets en distributed execution vallen buiten de eerste versie.

## Aanpak

1. Eisen en ontwerp nalopen en onbekende MCP-details met een kleine proef onderzoeken.
2. Projectbasis controleren.
3. Werkende features realiseren volgens docs/roadmap.md.
4. Security en tests in elke feature meenemen.
5. Demonstreerbare eerste versie opleveren en documentatie actualiseren.

## Evaluatie

Per feature toon ik acceptatiecriteria aan met tests en/of reproduceerbare controles. Ik meet uitvoeringsduur, beëindiging bij timeout, begrensde output en gedrag bij gelijktijdige requests. Succes betekent dat grenzen aantoonbaar worden gerespecteerd, niet alleen dat de normale route werkt.

## Planning

Concrete datums vul ik in GitHub Project Roadmap in, met Start Date, End Date en weeklabels. Afhankelijkheden staan in de featurebestanden. Ik plan eerst F00–F02 concreet; latere schattingen herzie ik op basis van ervaring.

## Oplevering en werkwijze

Code, README, ontwerp, configuratievoorbeelden, voorbeeldrequests en demonstratie. Per issue: wat, waarom, hoe en voortgang. Volgens de Educom-werkwijze commit en push ik rond 12:00 en 17:00, ook als het werk nog niet klaar is. Dit is mijn werkwijze; er worden geen automatische pushes ingesteld.

## Nog vast te stellen

- Deadline en beschikbare ontwikkeltijd.
- Verwachte MCP-protocolversie, transport en client.
- Invulling van voorbeeld-REST-routes naast MCP.
- Ondersteunde executabletypen en beheer van procesbomen per OS.
- Naamgeving van documenten conform de opleidingsafspraken.

