---
id: F05
prioriteit: Must
status: Backlog
milestone: "MCP-keten"
---

# F05 — MCP via HTTP aan de kern koppelen

## Doel

MCP via HTTP aan de kern koppelen. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

F00, F04.

## Requirements

R11, R13, R15, R16.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Todo

- [ ] SDK en gekozen transport integreren.
- [ ] Registry vertalen naar MCP-tools en inputschema's.
- [ ] Toolcalls via dezelfde toolservice als REST uitvoeren.
- [ ] Kernfouten en procesresultaten naar MCP-resultaten vertalen.
- [ ] Toepasselijke transportbeveiliging en sessiegedrag implementeren.
- [ ] Met echte MCP-client en ongeldige protocolrequests testen.

## Acceptatiecriteria

- [ ] MCP-client ontdekt geconfigureerde tools en voert ze uit.
- [ ] Geen aparte executor of validatieregels in MCP-adapter.
- [ ] REST en MCP tonen consistent inhoudelijk gedrag.
- [ ] Protocol- en transportkeuzes zijn gedocumenteerd en getoetst.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F05-mcp-integratie.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: nog vast te leggen.
- [Featurenotitie](../../notities/F05-mcp-integratie.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

