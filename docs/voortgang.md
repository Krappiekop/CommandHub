# Huidige voortgang

Bijgewerkt: 2026-10-20.

## Actief

F00 staat klaar voor review. De aparte proefserver werkt via Streamable HTTP; MCP Inspector CLI heeft de vaste tool `hello` ontdekt en aangeroepen. De vereiste Cargo-controles zijn op Windows voor zowel de hoofdcrate als de proefcrate geslaagd.

## Aanwezig

Documentatieopzet, featureplannen, notities per feature, minimale Axum-statusroute met logging en de geïsoleerde F00-proef onder `prototypes/f00-mcp-probe`. De hoofdapplicatie heeft nog geen toolconfiguratie, executor of MCP-integratie.

## Nog open

- F00: review en PR-merge door gebruiker. De proef onderzoekt `tool_not_found` en een extra argument; bredere fout- en transportsecuritytests horen bij F05 waar relevant.
- F01: lokale statusroute, logging en Ctrl+C controleren; formatter en Clippy uitvoeren; repository-inrichting nalopen.
- Daarna F02 verder uitwerken.

## Volgende sessie

Review en sync F00 zelf en merge de PR naar `develop`. Daarna F01 lokaal controleren en vervolgens F02 uitwerken.

## Verificatie

Op Windows zijn `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` en `cargo test --locked` uitgevoerd vanuit de repositoryroot en `prototypes/f00-mcp-probe`; alle controles slaagden. Beide `cargo test`-runs vonden 0 tests, en Linux is niet gecontroleerd. MCP Inspector CLI 2.11.0 meldde `tool_not_found` voor een onbekende tool; een extra argument aan parameterloze `hello` werd niet afgewezen. Zie [F00-notitie](../notities/F00-mcp-proef.md) en [ADR-002](beslissingen/ADR-002-mcp-proef.md). Dit verifieert de basis en proef op Windows, maar niet de functionaliteit van de hoofdapplicatie of platformondersteuning op Linux.
