---
id: F01
prioriteit: Must
status: Backlog
milestone: "Fundament"
---

# F01 — Projectbasis

## Doel

Projectbasis. Dit is een werkpakket; gebruik waar nodig meerdere kleine issues/PR's.

## Afhankelijkheden

Geen.

## Requirements

R13, R15, R19.

## Scope

Zie onderstaande taken. Nieuwe uitbreidingen krijgen een apart issue; vergroot de scope niet stilzwijgend. Bij F01 is de minimale codebasis al aanwezig, maar lokale controle blijft nodig.

## Todo

- [x] Rust-project en minimale Axum-statusroute lokaal controleren.
- [x] Logging en Ctrl+C-afhandeling controleren.
- [ ] Cargo.lock opnemen in een commit (het bestand bestaat al).
- [x] Formatter, Clippy en build controleren; CI voor Windows/Linux voorbereiden.

## Acceptatiecriteria

- [x] GET /status geeft 200 met status en applicatienaam.
- [x] cargo fmt --check en Clippy slagen.
- [x] Documentatie maakt duidelijk wat wel en niet geïmplementeerd is.

## Bewijs en koppelingen

- GitHub issue: nog aan te maken.
- Branch: feature/F01-projectbasis.
- Pull request: nog niet aangemaakt.
- Tests/handmatige verificatie: nog vast te leggen.
- [Featurenotitie](../../notities/F01-projectbasis.md).

## Afronden

- [ ] Relevante controles geslaagd.
- [ ] Documentatie en notitie bijgewerkt.
- [ ] Voortgang bijgewerkt.
- [ ] PR gemerged; pas daarna status Klaar.

