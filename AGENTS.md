# Werken aan CommandHub

Lees eerst README.md, docs/voortgang.md, docs/requirements.md, docs/architectuur.md en het actieve featurebestand. Lees vervolgens de bijbehorende notitie in notities/.

- Dit is een leerproject. Leg relevante Rust-keuzes begrijpelijk uit.
- Controleer de actuele code en tests; notities zijn geen bewijs dat iets werkt.
- Implementeer alleen de afgesproken feature. Werk in kleine, controleerbare stappen.
- Featurebestanden zijn de bron voor gedetailleerde todo's en acceptatiecriteria.
- Noteer uitleg, keuzes, problemen en verificatie in de notitie van de feature.
- Werk docs/voortgang.md bij met huidige situatie, open punten en volgende stap.
- Markeer alleen werkelijk geverifieerd werk als klaar.
- Houd HTTP/MCP buiten de kern van configuratie, validatie en procesuitvoering.
- Geen shellstrings opbouwen uit invoer. Programma's en argumenten blijven gescheiden.
- De client kiest geen executable; beheerdersconfiguratie bepaalt toegestane tools.
- Log geen geheimen, volledige parameterwaarden of procesoutput.
- Voer passende controles uit: cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings en cargo test --locked.
- Meld wat niet getest kon worden en waarom.
- Geen commit, push, merge of externe publicatie zonder opdracht van de gebruiker.

