# Requirements: Local-First AI Assistant

> **Template Oorsprong**: Officieel | **ArcKit Versie**: 4.3.1 | **Commando**: `/arckit:requirements`

## Documentbeheer

| Veld | Waarde |
|------|--------|
| **Document ID** | ARC-000-REQS-v1.0 |
| **Document Type** | Requirements |
| **Project** | Local-First AI Assistant (Gemeente Leiden & Utrecht) |
| **Classificatie** | PUBLIC |
| **Status** | DRAFT |
| **Versie** | 1.0 |
| **Aanmaakdatum** | 2026-05-07 |
| **Laatste Wijziging** | 2026-05-07 |
| **Review Cyclus** | Per Sprint |
| **Volgende Review Datum** | 2026-05-21 |
| **Eigenaar** | Product Owner |
| **Beoordeeld Door** | PENDING |
| **Goedgekeurd Door** | PENDING |
| **Distributie** | Projectteam, Stakeholders |

## Revisiegeschiedenis

| Versie | Datum | Auteur | Wijzigingen | Goedgekeurd Door | Goedkeuringsdatum |
|--------|-------|--------|-------------|------------------|-------------------|
| 1.0 | 2026-05-07 | ArcKit AI | Eerste aanmaak vanuit `/arckit:requirements` commando | PENDING | PENDING |

---

## Uitvoerende Samenvatting

### Overzicht

Dit document definieert alle requirements voor het local-first AI assistant project van Gemeente Leiden en Gemeente Utrecht. Requirements zijn gecategoriseerd als functionele, non-functionele, en technische requirements.

**Requirements Overzicht**:
- **Totaal Requirements**: [AANTAL]
- **Functioneel**: [AANTAL]
- **Non-Functioneel**: [AANTAL]
- **Technisch**: [AANTAL]
- **Compliance**: [AANTAL]

### Stakeholder Mapping

| Stakeholder | Primaire Requirements |
|-------------|----------------------|
| **Gemeente Leiden** | EFF-001 t/m EFF-010, COM-001 t/m COM-005 |
| **Gemeente Utrecht** | EFF-011 t/m EFF-020, COM-001 t/m COM-005 |
| **Beide Gemeenten** | NFR-001 t/m NFR-015, TECH-001 t/m TECH-010 |

---

## A. Functionele Requirements

### EFF-001: Chat Gesprekken

**ID**: EFF-001
**Prioriteit**: HOOG
**Type**: Functioneel
**Stakeholder**: Gebruiker (Ambtenaar)

**Beschrijving**:
De applicatie MOET gebruikers in staat stellen om chatgesprekken te voeren met een AI assistant. Gesprekken MOETEN context bewaren en gebruikers MOETEN eerdere gesprekken kunnen bekijken en hervatten.

**Acceptatiecriteria**:
- [ ] Gebruiker kan een nieuw gesprek starten
- [ ] Gebruiker kan eerdere gesprekken zien in een lijst
- [ ] Gebruiker kan een eerdere gesprek hervatten
- [ ] Gesprekken hebben een titel (auto-generated of handmatig)
- [ ] Gesprekken tonen een tijdstip van laatste activiteit

**Sprint**: 1

---

### EFF-002: Berichten Verzenden

**ID**: EFF-002
**Prioriteit**: HOOG
**Type**: Functioneel
**Stakeholder**: Gebruiker (Ambtenaar)

**Beschrijving**:
Gebruikers MOETEN berichten kunnen verzenden naar de AI assistant en MOETEN antwoorden ontvangen. Berichten MOETEN verschillende rollen ondersteunen (gebruiker, assistant, systeem).

**Acceptatiecriteria**:
- [ ] Gebruiker kan tekst berichten versturen
- [ ] Berichten verschijnen in de juiste volgorde
- [ ] Systeemberichten worden onderscheiden van gebruikersberichten
- [ ] Typ indicatoren tonen tijdens AI verwerking
- [ ] Berichten hebben tijdstempels

**Sprint**: 1

---

### EFF-003: Document Upload

**ID**: EFF-003
**Prioriteit**: HOOG
**Type**: Functioneel
**Stakeholder**: Gebruiker (Ambtenaar)

**Beschrijving**:
Gebruikers MOETEN documenten kunnen uploaden (PDF, Word, tekstbestanden) die door de AI assistant geanalyseerd kunnen worden. Geüploade documenten MOETEN worden verwerkt en ontsloten voor de gesprekscontext.

**Acceptatiecriteria**:
- [ ] Gebruiker kan bestand selecteren via file picker
- [ ] Gebruiker kan bestand slepen (drag & drop)
- [ ] Ondersteunde formaten: PDF, DOCX, DOC, TXT, MD
- [ ] Upload voortgang wordt getoond
- [ ] Geüploade documenten verschijnen in gesprek
- [ ] Document wordt geanalyseerd en samengevat

**Sprint**: 2

---

### EFF-004: PII Detectie en Redactie

**ID**: EFF-004
**Prioriteit**: KRITISCH
**Type**: Functioneel
**Stakeholder**: DPO, Security Officer

**Beschrijving**:
De applicatie MOET automatisch persoonsgegevens (PII) detecteren in berichten en documenten. Gedetecteerde PII MOET worden gemarkeerd en, indien verzonden naar externe AI providers, geredigeerd worden.

**Acceptatiecriteria**:
- [ ] PII detectie werkt op: namen, email, telefoonnummers, adressen, BSN
- [ ] Gedetecteerde PII wordt visueel gemarkeerd in de UI
- [ ] Gebruiker kan PII detectie in-/uitschakelen
- [ ] PII log wordt bijgehouden voor compliance
- [ ] Externe calls gebruiken geredigeerde content

**Sprint**: 2

---

### EFF-005: Local-First Mode

**ID**: EFF-005
**Prioriteit**: KRITISCH
**Type**: Functioneel
**Stakeholder**: CIO, DPO

**Beschrijving**:
De applicatie MOET standaard werken in local-first modus, waarbij alle verwerking lokaal plaatsvindt en geen data naar externe servers wordt verzonden, tenzij de gebruiker expliciet kiest voor een externe provider.

**Acceptatiecriteria**:
- [ ] Default modus is local-only
- [ ] Gebruiker kan kiezen tussen lokaal en externe modellen
- [ ] Visuele indicatie toont welke modus actief is
- [ ] External modi toont waarschuwing voor privacy
- [ ] Lokaal beschikbare modellen worden getoond

**Sprint**: 1

---

### EFF-006: AI Provider Selectie

**ID**: EFF-006
**Prioriteit**: HOOG
**Type**: Functioneel
**Stakeholder**: CIO, Architect

**Beschrijving**:
Gebruikers MOETEN kunnen kiezen tussen verschillende AI providers. De applicatie MOET Europese providers voorrang geven (Mistral, Aleph Alpha) en niet-Europese providers als optie tonen met waarschuwing.

**Acceptatiecriteria**:
- [ ] Lijst van beschikbare providers wordt getoond
- [ ] Europese providers worden eerst getoond (Mistral, Aleph Alpha)
- [ ] Niet-Europese providers hebben EU-soevereiniteit waarschuwing
- [ ] Gebruiker kan default provider instellen
- [ ] Provider wisseling is mogelijk per gesprek

**Sprint**: 1

---

### EFF-007: Gesprek Zoeken

**ID**: EFF-007
**Prioriteit**: MIDDEN
**Type**: Functioneel
**Stakeholder**: Gebruiker (Ambtenaar)

**Beschrijving**:
Gebruikers MOETEN kunnen zoeken in hun gespreksgeschiedenis om eerlijke gesprekken en informatie snel terug te vinden.

**Acceptatiecriteria**:
- [ ] Zoekbalk beschikbaar in gesprekken lijst
- [ ] Zoekt in titels en bericht content
- [ ] Resultaten worden gefilterd terwijl typt
- [ ] Zoekresultaten highlighten de zoekterm
- [ ] Gebruiker kan filteren op datum range

**Sprint**: 3

---

### EFF-008: Gebruikersinstellingen

**ID**: EFF-008
**Prioriteit**: MIDDEN
**Type**: Functioneel
**Stakeholder**: Gebruiker (Ambtenaar)

**Beschrijving**:
Gebruikers MOETEN hun voorkeuren kunnen configureren, inclusief thema, taal, default provider, en privacy instellingen.

**Acceptatiecriteria**:
- [ ] Instellingen scherm beschikbaar
- [ ] Thema keuze: light/dark/system
- [ ] Taal keuze: Nederlands/Engels
- [ ] Default provider keuze
- [ ] PII detectie aan/uit
- [ ] Local-only modus aan/uit
- [ ] Telemetry aan/uit

**Sprint**: 1

---

### EFF-009: Data Export

**ID**: EFF-009
**Prioriteit**: HOOG
**Type**: Functioneel
**Stakeholder**: DPO, Gebruiker

**Beschrijving**:
Gebruikers MOETEN hun data kunnen exporteren voor AVG/GDPR "recht op overdraagbaarheid". De export MOET alle gesprekken, instellingen en geüploade documenten bevatten.

**Acceptatiecriteria**:
- [ ] Export knop beschikbaar in instellingen
- [ ] Export formaat is JSON (machine-readable) + bestanden (ZIP)
- [ ] Export bevat alle gesprekken en berichten
- [ ] Export bevat instellingen
- [ ] Export bevat document metadata
- [ ] Export toont voortgangsindicatie

**Sprint**: 3

---

### EFF-010: Data Verwijderen

**ID**: EFF-010
**Prioriteit**: KRITISCH
**Type**: Functioneel
**Stakeholder**: DPO, Gebruiker

**Beschrijving**:
Gebruikers MOETEN hun data kunnen verwijderen voor AVG/GDPR "recht op vergetelheid". Dit omvat gesprekken, documenten en instellingen.

**Acceptatiecriteria**:
- [ ] "Verwijder alle data" knop beschikbaar
- [ ] Bevestiging vereist (2-stap proces)
- [ ] Verwijdering omvat alle gesprekken
- [ ] Verwijdering omvat geüploade documenten
- [ ] Verwijdering omvat instellingen
- [ ] Bevestiging na succesvolle verwijdering

**Sprint**: 3

---

### EFF-011: Gemeente Leiden Branding

**ID**: EFF-011
**Prioriteit**: MIDDEN
**Type**: Functioneel
**Stakeholder**: Gemeente Leiden (Communicatie)

**Beschrijving**:
De applicatie MOET gebrand worden met Gemeente Leiden huisstijl wanneer gebruikt door Leiden ambtenaren.

**Acceptatiecriteria**:
- [ ] Logo van Gemeente Leiden zichtbaar
- [ ] Kleuren volgen Leiden huisstijl
- [ ] App naam is configureerbaar
- [ ] Branding is configureerbaar per installatie

**Sprint**: 4

---

### EFF-012: Gemeente Utrecht Branding

**ID**: EFF-012
**Prioriteit**: MIDDEN
**Type**: Functioneel
**Stakeholder**: Gemeente Utrecht (Communicatie)

**Beschrijving**:
De applicatie MOET gebrand worden met Gemeente Utrecht huisstijl wanneer gebruikt door Utrecht ambtenaren.

**Acceptatiecriteria**:
- [ ] Logo van Gemeente Utrecht zichtbaar
- [ ] Kleuren volgen Utrecht huisstijl
- [ ] App naam is configureerbaar
- [ ] Branding is configureerbaar per installatie

**Sprint**: 4

---

### EFF-013: Schalingsvalidatie (Utrecht)

**ID**: EFF-013
**Prioriteit**: HOOG
**Type**: Functioneel
**Stakeholder**: Gemeente Utrecht (Management)

**Beschrijving**:
De applicatie MOET grotere volumes aankunnen zoals verwacht door de grootste gebruikersbase (Utrecht: ~6000 medewerkers).

**Acceptatiecriteria**:
- [ ] Ondersteunt 1000+ gelijktijdige gebruikers
- [ ] Ondersteunt documenten van 500+ pagina's
- [ ] Respons blijft <3 seconden onder belasting
- [ ] Database schaalt naar grootere volumes
- [ ] Memory usage blijft binnen grenzen

**Sprint**: 5

---

### EFF-014: Geavanceerde Document Processing

**ID**: EFF-014
**Prioriteit**: MIDDEN
**Type**: Functioneel
**Stakeholder**: Gebruiker (Ambtenaar)

**Beschrijving**:
De applicatie MOET geavanceerde document processing ondersteunen, zoals multi-document upload, samenvatting van meerdere documenten, en extractie van specifieke informatie.

**Acceptatiecriteria**:
- [ ] Meerdere documenten tegelijk uploaden
- [ ] Documenten worden samengevat
- [ ] Vragen over specifieke documenten mogelijk
- [ ] Cross-document referenties mogelijk
- [ ] Document chunks voor RAG

**Sprint**: 4

---

### EFF-015: Prompt Templates

**ID**: EFF-015
**Prioriteit**: LAAG
**Type**: Functioneel
**Stakeholder**: Gebruiker (Ambtenaar)

**Beschrijving**:
Gebruikers MOETEN gebruik kunnen maken van vooraf gedefinieerde prompt templates voor veelvoorkomende taken.

**Acceptatiecriteria**:
- [ ] Bibliotheek van templates beschikbaar
- [ ] Templates configureerbaar door beheerder
- [ ] Gebruiker kan template selecteren en aanpassen
- [ ] Variabelen in templates (bijv. naam, datum)
- [ ] Templates zijn te delen tussen gebruikers

**Sprint**: 6 (Nice-to-have)

---

## B. Non-Functionele Requirements

### NFR-001: Europese AI Modellen (KRITISCH)

**ID**: NFR-001
**Prioriteit**: KRITISCH
**Type**: Non-Functioneel (Compliance)
**Stakeholder**: CIO, DPO, College van B&W

**Beschrijving**:
Alle AI modellen en dienstverlening MOETEN voorkeur geven aan Europese, open source oplossingen. Niet-Europese of gesloten modellen zijn alleen toegestaan met expliciete rechtvaardiging en uitzondering.

**Requirements**:
- [ ] Default AI provider is Europese (Mistral AI of vergelijkbaar)
- [ ] Data processing vindt plaats binnen EU
- [ ] Niet-Europese providers vereisen expliciete goedkeuring
- [ ] Uitzonderingen worden gedocumenteerd
- [ ] Exit strategy is gedefinieerd voor elke AI afhankelijkheid

**Validatie**:
- [ ] AI provider lijst is goedgekeurd door CTO en DPO
- [ ] Data residatie is geverifieerd (EU compliance)
- [ ] Uitzonderingen hebben documentatie

**Sprint**: 1

---

### NFR-002: AVG/GDPR Compliance

**ID**: NFR-002
**Prioriteit**: KRITISCH
**Type**: Non-Functioneel (Compliance)
**Stakeholder**: DPO

**Beschrijving**:
De applicatie MOET volledig voldoen aan AVG/GDPR, inclusief privacy by design, recht op vergetelheid, data portabiliteit, en data minimization.

**Requirements**:
- [ ] Privacy by design en by default
- [ ] Recht op toegang (data export)
- [ ] Recht op rectificatie (gegevens wijzigen)
- [ ] Recht op vergetelheid (data verwijderen)
- [ ] Recht op portabiliteit (data export)
- [ ] Data minimization (minimale data opslaan)
- [ ] DPIA uitgevoerd voor productie

**Validatie**:
- [ ] DPIA goedgekeurd door DPO
- [ ] Privacy impact assessment voltooid
- [ ] Data retention policy gedefinieerd

**Sprint**: 1

---

### NFR-003: Performance

**ID**: NFR-003
**Prioriteit**: HOOG
**Type**: Non-Functioneel (Kwaliteit)
**Stakeholder**: Gebruiker

**Beschrijving**:
De applicatie MOET responsief zijn en snelle feedback geven aan gebruikers, zelfs bij grote documenten of complexe vragen.

**Requirements**:
- [ ] Tijd tot eerste token: <2 seconden
- [ ] Bericht generation snelheid: >20 tokens/seconde
- [ ] UI responsiviteit: <100ms voor interacties
- [ ] Document processing: <10 seconden per 100 pagina's
- [ ] Zoekresultaten: <500ms voor 1000 gesprekken

**Validatie**:
- [ ] Performance tests uitgevoerd
- [ ] Load tests met 100+ concurrente gebruikers
- [ ] Profiling uitgevoerd voor hotspots

**Sprint**: 2

---

### NFR-004: Beschikbaarheid

**ID**: NFR-004
**Prioriteit**: HOOG
**Type**: Non-Functioneel (Kwaliteit)
**Stakeholder**: CIO

**Beschrijving**:
De applicatie MOET betrouwbaar beschikbaar zijn voor gemeente ambtenaren tijdens werkuren, met minimale downtime.

**Requirements**:
- [ ] Uptime target: 99% (excl. gepland onderhoud)
- [ ] Graceful degradation bij externe provider uitval
- [ ] Automatische herstel mechanismen
- [ ] Backup en recovery procedure
- [ ] Gepland onderhoud buiten kantooruren

**Validatie**:
- [ ] Availability monitoring geïmplementeerd
- [ ] Failover getest
- [ ] Backup/restore procedure getest

**Sprint**: 3

---

### NFR-005: Security

**ID**: NFR-005
**Prioriteit**: KRITISCH
**Type**: Non-Functioneel (Security)
**Stakeholder**: CISO

**Beschrijving**:
De applicatie MOET voldoen aan gemeentelijke security baseline en SUP (Standaard Urwerk) eisen.

**Requirements**:
- [ ] Encryptie at rest (AES-256)
- [ ] Encryptie in transit (TLS 1.3+)
- [ ] Secure storage voor API keys (keychain/credential manager)
- [ ] Logging van alle security relevante events
- [ ] Vulnerability scanning voor dependencies
- [ ] Security review voor release

**Validatie**:
- [ ] Penetration test uitgevoerd
- [ ] Security scan door CIO afdeling
- [ ] Code review door security expert

**Sprint**: 2

---

### NFR-006: Onderhoudbaarheid

**ID**: NFR-006
**Prioriteit**: MIDDEN
**Type**: Non-Functioneel (Kwaliteit)
**Stakeholder**: Ontwikkelteam

**Beschrijving**:
De applicatie MOET ontworpen zijn voor eenvoudig onderhoud en doorontwikkeling door gemeente development teams.

**Requirements**:
- [ ] Modulaire architectuur
- [ ] Documentatie (README, API docs)
- [ ] Code conventions (Rust/TypeScript standaarden)
- [ ] Automated testing (>70% coverage)
- [ ] CI/CD pipeline
- [ ] Logging en monitoring

**Validatie**:
- [ ] Code review door senior developer
- [ ] Documentation review
- [ ] Test coverage gemeten

**Sprint**: Continue

---

### NFR-007: Toegankelijkheid

**ID**: NFR-007
**Prioriteit**: MIDDEN
**Type**: Non-Functioneel (Inclusie)
**Stakeholder**: Communicatie, Diversiteit

**Beschrijving**:
De applicatie MOET toegankelijk zijn voor alle ambtenaren, inclusief那些 met visuele beperkingen.

**Requirements**:
- [ ] Keyboard navigatie mogelijk
- [ ] Screen reader support
- [ ] Contrast ratio voldoet aan WCAG AA
- [ ] Font sizes instelbaar
- [ ] Ondersteuning voor donker modus

**Validatie**:
- [ ] Accessibility audit uitgevoerd
- [ ] Getest met screen readers
- [ ] Contrast gemeten

**Sprint**: 4

---

### NFR-008: Audit Trail

**ID**: NFR-008
**Prioriteit**: HOOG
**Type**: Non-Functioneel (Compliance)
**Stakeholder**: DPO, Auditor

**Beschrijving**:
De applicatie MOET een volledige audit trail bijhouden van alle relevante acties voor compliance en debugging doeleinden.

**Requirements**:
- [ ] Logging van alle gebruikersacties
- [ ] Logging van PII detectie events
- [ ] Logging van AI provider calls
- [ ] Logging van data export en deletie
- [ ] Log retention: 6 maanden (security), 30 dagen (application)
- [ ] Logs zijn zoekbaar en filterbaar

**Validatie**:
- [ ] Audit trail review door DPO
- [ ] Log retention policy gedefinieerd
- [ ] Log access controls gedefinieerd

**Sprint**: 3

---

## C. Technische Requirements

### TECH-001: Local-First Architectuur

**ID**: TECH-001
**Prioriteit**: KRITISCH
**Type**: Technisch (Architectuur)
**Stakeholder**: Architect

**Beschrijving**:
De applicatie MOET een local-first architectuur volgen waarbij alle data standaard lokaal wordt opgeslagen en verwerkt.

**Requirements**:
- [ ] Local database (SQLite of vergelijkbaar)
- [ ] Local file storage voor documenten
- [ ] Geen cloud synchronisatie standaard
- [ ] Optionele integratie met gemeente systemen (later)
- [ ] Offline operatie mogelijk

**Validatie**:
- [ ] Architectuur review goedgekeurd
- [ ] Local storage geïmplementeerd
- [ ] Offline operatie getest

**Sprint**: 1

---

### TECH-002: Multi-Platform Support

**ID**: TECH-002
**Prioriteit**: HOOG
**Type**: Technisch (Platform)
**Stakeholder**: CIO

**Beschrijving**:
De applicatie MOET beschikbaar zijn op gemeente werkstations (Windows, macOS) en mogelijk op mobiel.

**Requirements**:
- [ ] Windows 10/11 support
- [ ] macOS 12+ support
- [ ] Linux support (optioneel)
- [ ] Consistente ervaring over platforms
- [ ] Platform-specifieke optimalisaties

**Validatie**:
- [ ] Getest op Windows en macOS
- [ ] Performance gelijkwaardig over platforms
- [ ] Feature parity over platforms

**Sprint**: 2

---

### TECH-003: AI Provider Abstraktie

**ID**: TECH-003
**Prioriteit**: HOOG
**Type**: Technisch (Architectuur)
**Stakeholder**: Architect

**Beschrijving**:
De applicatie MOET een abstractielaag hebben voor AI providers, waardoor eenvoudig van provider gewisseld kan worden zonder code wijzigingen.

**Requirements**:
- [ ] Uniform interface voor alle providers
- [ ] Provider configuratie via settings
- [ ] Provider-specifieke parameters
- [ ] Fallback mechanisme bij provider uitval
- [ ] Provider switching zonder data verlies

**Validatie**:
- [ ] Meerdere providers geïmplementeerd
- [ ] Provider switching getest
- [ ] Fallback mechanisme getest

**Sprint**: 1

---

### TECH-004: Gemeente Integraties

**ID**: TECH-004
**Prioriteit**: MIDDEN
**Type**: Technisch (Integratie)
**Stakeholder**: Enterprise Architect

**Beschrijving**:
De applicatie MOET mogelijkheden bieden voor integratie met gemeente systemen (zoals Zaaksysteem, DMS, etc.).

**Requirements**:
- [ ] API beschikbaar voor integraties
- [ ] Webhook support voor events
- [ ] OAuth/OIDC support voor authenticatie
- [ ] Gemeente SSO integratie (toekomst)
- [ ] Audit log export naar SIEM (toekomst)

**Validatie**:
- [ ] API documentatie beschikbaar
- [ ] Integratie testomgeving
- [ ] Security review van API

**Sprint**: 5 (Nice-to-have)

---

### TECH-005: Ollama Integration

**ID**: TECH-005
**Prioriteit**: HOOG
**Type**: Technisch (AI Provider)
**Stakeholder**: Architect

**Beschrijving**:
De applicatie MOET integratie met Ollama voor local AI model hosting, waardoor gemeenten eigen modellen kunnen draaien.

**Requirements**:
- [ ] Ollama API integratie
- [ ] Model download en management
- [ ] Local model discovery
- [ ] Fallback naar Ollama bij API uitval
- [ ] Model versioning support

**Validatie**:
- [ ] Ollama integratie getest
- [ ] Local modellen getest
- [ ] Performance getest

**Sprint**: 2

---

### TECH-006: Mistral AI Integration

**ID**: TECH-006
**Prioriteit**: HOOG
**Type**: Technisch (AI Provider)
**Stakeholder**: Architect

**Beschrijving**:
De applicatie MOET integratie met Mistral AI (Europese provider) als primary AI provider.

**Requirements**:
- [ ] Mistral API integratie
- [ ] Support voor Mixtral modellen
- [ ] Support voor Mistral modellen
- [ ] API key management
- [ ] Rate limiting handling

**Validatie**:
- [ ] Mistral API integratie getest
- [ ] Model performance getest
- [ ] Error handling getest

**Sprint**: 1

---

### TECH-007: Document Processing Pipeline

**ID**: TECH-007
**Prioriteit**: HOOG
**Type**: Technisch (Document)
**Stakeholder**: Architect

**Beschrijving**:
De applicatie MOET een pipeline hebben voor document processing (upload → extract → chunk → embed).

**Requirements**:
- [ ] PDF parsing en tekst extractie
- [ ] DOCX/DOC parsing
- [ ] Tekst chunking met overlap
- [ ] Embedding generation (optioneel)
- [ ] Progress feedback aan gebruiker

**Validatie**:
- [ ] Verscheidene document formats getest
- [ ] Grote documenten getest (>100 pagina's)
- [ ] Error handling getest

**Sprint**: 2

---

### TECH-008: PII Detection Pipeline

**ID**: TECH-008
**Prioriteit**: KRITISCH
**Type**: Technisch (Security)
**Stakeholder**: DPO, CISO

**Beschrijving**:
De applicatie MOET een pipeline hebben voor PII detection en redactie voordat data naar externe AI providers wordt gestuurd.

**Requirements**:
- [ ] Named Entity Recognition (NER) voor PII
- [ ] Detectie van: naam, email, telefoon, adres, BSN
- [ ] Redactie strategie (mask/placeholder/remove)
- [ ] PII logging voor compliance
- [ ] Gebruiker controle over detectie

**Validatie**:
- [ ] PII detection accuracy >90%
- [ ] False positive rate <5%
- [ ] Performance impact minimal
- [ ] DPO review van pipeline

**Sprint**: 2

---

## D. Requirements Traceability Matrix

### Requirements per Sprint

| Sprint | Requirements | Focus |
|--------|--------------|-------|
| **Sprint 1** | EFF-001, EFF-002, EFF-005, EFF-006, EFF-008, EFF-010, NFR-001, NFR-002, NFR-005, TECH-001, TECH-003, TECH-006 | Core chat, EU AI, Compliance, Foundation |
| **Sprint 2** | EFF-003, EFF-004, EFF-009, NFR-003, NFR-005, TECH-002, TECH-005, TECH-007, TECH-008 | Documents, PII, Performance, Security |
| **Sprint 3** | EFF-007, EFF-010, NFR-004, NFR-008 | Search, Privacy, Availability, Audit |
| **Sprint 4** | EFF-011, EFF-012, EFF-014, NFR-007 | Branding, Advanced docs, Accessibility |
| **Sprint 5** | EFF-013, TECH-004 | Scaling, Integrations |
| **Sprint 6** | EFF-015 | Templates (Nice-to-have) |

### Requirements per Stakeholder

| Stakeholder | Requirements |
|-------------|--------------|
| **Ambtenaren (Gebruikers)** | EFF-001 t/m EFF-010, EFF-015, NFR-003, NFR-007 |
| **Gemeente Leiden** | EFF-011, EFF-013, NFR-001, NFR-002, NFR-005, TECH-004 |
| **Gemeente Utrecht** | EFF-012, EFF-013, NFR-001, NFR-002, NFR-005, TECH-004 |
| **DPO** | EFF-004, EFF-009, EFF-010, NFR-002, NFR-008, TECH-008 |
| **CIO** | EFF-005, EFF-006, NFR-001, NFR-004, NFR-005, TECH-001, TECH-004 |
| **CISO** | EFF-004, NFR-005, TECH-008 |
| **Enterprise Architect** | TECH-003, TECH-004, TECH-006, TECH-007 |

---

## E. Prioriteit Matrix

| Prioriteit | Aantal Requirements | IDs |
|------------|-------------------|-----|
| **KRITISCH** | 6 | EFF-004, EFF-005, EFF-010, NFR-001, NFR-002, NFR-005, TECH-001, TECH-008 |
| **HOOG** | 13 | EFF-001, EFF-002, EFF-003, EFF-006, EFF-009, NFR-003, NFR-004, NFR-008, TECH-002, TECH-003, TECH-005, TECH-006, TECH-007, EFF-013 |
| **MIDDEN** | 7 | EFF-007, EFF-008, EFF-011, EFF-012, EFF-014, NFR-006, NFR-007, TECH-004 |
| **LAAG** | 1 | EFF-015 |

---

## F. Mogelijke Requirements (Backlog)

De volgende requirements zijn geïdentificeerd maar nog niet geprioriteerd:

| ID | Requirement | Prioriteit | Beschrijving |
|----|------------|------------|--------------|
| EFF-101 | Voice input | LAAG | Spraak-naar-tekst voor berichten |
| EFF-102 | Multi-language | LAAG | Ondersteuning voor andere talen (naast NL/EN) |
| EFF-103 | Gesprek delen | MIDDEN | Gesprekken delen met collega's |
| EFF-104 | Analytics dashboard | MIDDEN | Usage analytics voor beheerders |
| EFF-105 | Custom instructions | LAAG | Gebruiker kan systeem prompts aanpassen |
| NFR-101 | Multi-user support | MIDDEN | Meerdere gebruikers per apparaat |
| TECH-101 | Mobile app | LAAG | Native iOS/Android apps |

---

**Document End**

*Requirements zijn een levend document en worden bijgewerkt na elke sprint review en stakeholder feedback.*

## Externe Referenties

| Document | Type | Bron | Kernextracties | Pad |
|----------|------|------|----------------|-----|
| *Geen voorzien* | — | — | — | — |

---

**Gegenereerd door**: ArcKit `/arckit:requirements` commando
**Gegenereerd op**: 2026-05-07
**ArcKit Versie**: 4.3.1
**Project**: Local-First AI Assistant (Gemeente Leiden & Utrecht)
**AI Model**: Claude Opus 4.7
