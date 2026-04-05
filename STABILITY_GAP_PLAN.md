# Stabilitäts- und Realitäts-Plan (Capabilities vs. Realität)

## Ziel

Dieses Dokument beantwortet drei Fragen:
1. **Was behauptet das Projekt zu können?**
2. **Was kann es aktuell nach Code- und Testlage wirklich?**
3. **Wie bringen wir den Stand in einen stabilen, glaubwürdigen Zustand?**

---

## 1) Kurzfazit

- Die Basis (CLI, OpenRouter-Anbindung, Kern-Tools) ist funktionsfähig.
- Es gibt jedoch **Feature-Claims**, die aktuell nur teilweise gedeckt sind (z. B. MCP/REPL als Platzhalter, Memory-Load nur rudimentär).
- Ein Teil der Tests war bisher nur Placeholder. Das ist ein Stabilitäts- und Vertrauensrisiko.

**Wichtig:** „Stable Release“ sollte nur für den Bereich gelten, der durch echte Tests + reproduzierbare Akzeptanzkriterien belegt ist.

---

## 2) Claims vs. Ist-Stand (Audit)

### A) Klar belegt (Code + Verhalten vorhanden)

- CLI-Kommandos (`chat`, `ask`, `run`, `review`, `status`, `init`) sind strukturell implementiert.
- OpenRouter-Client inkl. Auth-Flow und Modellauflösung ist vorhanden.
- Viele FS/Bash/Git/Net-Tools existieren und sind aufrufbar.

### B) Teilweise belegt / eingeschränkt

- Security-Härtungen sind vorhanden, aber Wirksamkeit muss über mehr negative Tests abgesichert werden.
- Memory-Modul hat Persistenzstruktur, aber Parsing/Reload ist noch minimal.
- Tool-Liste ist umfangreich, aber „voll produktiv“ ist je Tool unterschiedlich.

### C) Nicht belegt / Platzhalter

- MCP-Integration: explizit Placeholder.
- REPL: eval derzeit nur synthetische Antwort, keine echte Ausführung.
- Teile der bisherigen Tool-Tests waren Platzhalter statt echter Assertions.

---

## 3) Stabilitätsziele (Definition of Stable)

Ein Feature darf den Status **stable** nur tragen, wenn:

1. **Semantik klar dokumentiert** (inkl. Grenzen/Nicht-Ziele)
2. **Mind. 1 Erfolgstest + 1 Fehlertest** automatisiert
3. **Deterministisches Verhalten** in CI (keine flaky Assertions)
4. **Sicherheitskritische Pfade** mit Negativtests abgedeckt
5. **Claim in README passt exakt** zum tatsächlichen Implementierungsgrad

---

## 4) Umsetzungsplan (priorisiert)

## P0 – Vertrauenslücke schließen (sofort)

1. Placeholder-Tests entfernen/ersetzen (echte Assertions).
2. README/Architekturtexte auf „implemented vs planned“ trennen.
3. Platzhalter-Features (MCP/REPL) klar als experimental markieren.
4. CI-Gate ergänzen: keine Placeholder-Tests erlaubt.

## P1 – Sicherheits- und Robustheitsniveau erhöhen

1. Negativtests für Path-/Command-Validierung erweitern.
2. Error-Contexts vereinheitlichen (`context(...)`) in kritischen Pfaden.
3. Timeouts/Retry-Kanten systematisch testen.

## P2 – Produktreife pro Modul

1. Memory: echtes Reload/Parsing + Konsistenztests.
2. MCP: entweder minimal real integrieren oder aus Stable-Claim entfernen.
3. REPL: echte Evaluations-Engine oder klare Deaktivierung im Stable-Profil.

---

## 5) Messbare KPIs

- **0 Placeholder-Tests** in `tests/`.
- **>= 1 Negativtest** pro sicherheitskritischem Modul (bash/filesystem/auth).
- **100% Claim-zu-Code-Mapping** für README-Featureliste.
- **CI grün** auf clean checkout (`cargo fmt --check`, `cargo clippy`, `cargo test`).

---

## 6) Kommunikationsregel nach außen

Bis P0 abgeschlossen ist, sollte nach außen kommuniziert werden:

- „Core ist stabil“
- „Einige Advanced-Module sind experimental/in progress“

So bleibt das Projekt glaubwürdig und reduziert Nutzerfrust durch überzogene Erwartungen.
