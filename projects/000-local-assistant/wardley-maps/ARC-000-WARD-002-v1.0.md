# Wardley Map: Local-First AI Assistant - Strategische Component Positionering (Update)

> **Template Oorsprong**: Officieel | **ArcKit Versie**: 4.3.1 | **Command**: `/arckit:wardley`

## Documentbeheer

| Veld | Waarde |
|------|--------|
| **Document ID** | ARC-000-WARD-002-v1.0 |
| **Document Type** | Wardley Map |
| **Project** | Local-First AI Assistant (Project 000) |
| **Classificatie** | PUBLIC |
| **Status** | IN_REVIEW |
| **Versie** | 1.0 |
| **Aanmaakdatum** | 2026-05-07 |
| **Laatste Wijziging** | 2026-05-07 |
| **Review Cyclus** | Per Kwartaal |
| **Volgende Review Datum** | 2026-08-07 |
| **Eigenaar** | Enterprise Architect |
| **Beoordeeld Door** | PENDING |
| **Goedgekeurd Door** | PENDING |
| **Distributie** | Projectteam, Stakeholders, Architecture Board |

## Revisiegeschiedenis

| Versie | Datum | Auteur | Wijzigingen | Goedgekeurd Door | Goedkeuringsdatum |
|--------|-------|--------|-------------|------------------|-------------------|
| 1.0 | 2026-05-07 | ArcKit AI | Update op basis van HLD review (ARC-000-HLDR-v1.0) - Local-First strategie gecorrigeerd, Expert Mesh components toegevoegd | PENDING | PENDING |

---

## Map Visualisatie

**Bekijk deze map**: Plak de onderstaande map code in [https://create.wardleymaps.ai](https://create.wardleymaps.ai)

```wardley
title Local-First AI Assistant - Strategische Component Positionering (Update)
anchor Ambtenaar [0.95, 0.30]
annotation 1 [0.82, 0.38] Expert Mesh: Concurrentievoordeel - Bouw
annotation 2 [0.75, 0.40] PII Pipeline: AVG/GDPR verplicht - Bouw
annotation 3 [0.22, 0.90] Commodity: Gebruik standaard oplossingen
annotation 4 [0.48, 0.52] AI Abstraction: Commoditiserend
note Local-First = Primary (Ollama), Cloud = Fallback (Mistral/Aleph Alpha) [0.20, 0.20]

component Ambtenaar [0.95, 0.30]
component AI_Assistentie_Behoefte [0.92, 0.25]
component Expert_Mesh_Systeem [0.82, 0.38]
component Entry_Actor [0.78, 0.42]
component Registry_Actor [0.72, 0.45]
component Document_Orchestrator [0.68, 0.40]
component Document_Improver [0.65, 0.38]
component PII_Detection_Pipeline [0.75, 0.40]
component Research_Expert [0.62, 0.42]
component Frontend_Expert [0.58, 0.38]
component Rust_Expert [0.55, 0.35]
component Reviewer_Expert [0.57, 0.40]
component PII_Stripper_Expert [0.52, 0.35]
component Schrijver_Expert [0.60, 0.38]
component Document_Experts [0.63, 0.40]
component AI_Provider_Abstraction [0.48, 0.52]
component Web_UI [0.35, 0.68]
component Desktop_App_Shell [0.28, 0.72]
component Local_Database_SQLite [0.22, 0.90]
component Local_File_Storage [0.18, 0.92]
component Ollama_Local_AI [0.25, 0.65]
component Mistral_AI [0.30, 0.72]
component Aleph_Alpha [0.28, 0.70]

Ambtenaar -> AI_Assistentie_Behoefte
AI_Assistentie_Behoefte -> Expert_Mesh_Systeem
AI_Assistentie_Behoefte -> PII_Detection_Pipeline
Expert_Mesh_Systeem -> Entry_Actor
Expert_Mesh_Systeem -> Registry_Actor
Expert_Mesh_Systeem -> Document_Orchestrator
Expert_Mesh_Systeem -> Document_Improver
Entry_Actor -> Research_Expert
Entry_Actor -> Frontend_Expert
Entry_Actor -> Rust_Expert
Entry_Actor -> Reviewer_Expert
Entry_Actor -> PII_Stripper_Expert
Entry_Actor -> Schrijver_Expert
Entry_Actor -> Document_Experts
Document_Orchestrator -> Research_Expert
Document_Orchestrator -> Schrijver_Expert
Document_Orchestrator -> Reviewer_Expert
Document_Improver -> Reviewer_Expert
PII_Detection_Pipeline -> PII_Stripper_Expert
Expert_Mesh_Systeem -> AI_Provider_Abstraction
Research_Expert -> AI_Provider_Abstraction
Frontend_Expert -> Web_UI
Rust_Expert -> Desktop_App_Shell
Document_Orchestrator -> Local_Database_SQLite
Document_Orchestrator -> Local_File_Storage
AI_Provider_Abstraction -> Ollama_Local_AI
AI_Provider_Abstraction -> Mistral_AI
AI_Provider_Abstraction -> Aleph_Alpha
Web_UI -> Desktop_App_Shell
Local_Database_SQLite -> Local_File_Storage

pipeline AI_Assistentie_Behoefte [0.92, 0.25, 0.55]

evolve AI_Provider_Abstraction 0.68 label Commoditiserend door standaardisatie
evolve Expert_Mesh_Systeem 0.50 label Mogelijke productisatie in 18-24 maanden
evolve Ollama_Local_AI 0.72 label Local model hosting wordt mainstream

style wardley
```

<details>
<summary>Mermaid Wardley Map (werkt in GitHub, VS Code en andere Mermaid-viewers)</summary>

```mermaid
wardley-beta
title Local-First AI Assistant - Strategische Component Positionering (Update)
size [1100, 800]

anchor Ambtenaar [0.95, 0.30]

component AI_Assistentie_Behoefte [0.92, 0.25] (build)
component Expert_Mesh_Systeem [0.82, 0.38] (build)
component Entry_Actor [0.78, 0.42] (build)
component Registry_Actor [0.72, 0.45] (build)
component Document_Orchestrator [0.68, 0.40] (build)
component Document_Improver [0.65, 0.38] (build)
component PII_Detection_Pipeline [0.75, 0.40] (build)
component Research_Expert [0.62, 0.42] (build)
component Frontend_Expert [0.58, 0.38] (build)
component Rust_Expert [0.55, 0.35] (build)
component Reviewer_Expert [0.57, 0.40] (build)
component PII_Stripper_Expert [0.52, 0.35] (build)
component Schrijver_Expert [0.60, 0.38] (build)
component Document_Experts [0.63, 0.40] (build)
component AI_Provider_Abstraction [0.48, 0.52] (build)
component Web_UI [0.35, 0.68] (buy)
component Desktop_App_Shell [0.28, 0.72] (buy)
component Local_Database_SQLite [0.22, 0.90] (buy)
component Local_File_Storage [0.18, 0.92] (buy)
component Ollama_Local_AI [0.25, 0.65] (buy)
component Mistral_AI [0.30, 0.72] (buy)
component Aleph_Alpha [0.28, 0.70] (buy)

Ambtenaar -> AI_Assistentie_Behoefte
AI_Assistentie_Behoefte -> Expert_Mesh_Systeem
AI_Assistentie_Behoefte -> PII_Detection_Pipeline
Expert_Mesh_Systeem -> Entry_Actor
Expert_Mesh_Systeem -> Registry_Actor
Expert_Mesh_Systeem -> Document_Orchestrator
Expert_Mesh_Systeem -> Document_Improver
Entry_Actor -> Research_Expert
Entry_Actor -> Frontend_Expert
Entry_Actor -> Rust_Expert
Entry_Actor -> Reviewer_Expert
Entry_Actor -> PII_Stripper_Expert
Entry_Actor -> Schrijver_Expert
Entry_Actor -> Document_Experts
Document_Orchestrator -> Research_Expert
Document_Orchestrator -> Schrijver_Expert
Document_Orchestrator -> Reviewer_Expert
Document_Improver -> Reviewer_Expert
PII_Detection_Pipeline -> PII_Stripper_Expert
Expert_Mesh_Systeem -> AI_Provider_Abstraction
Research_Expert -> AI_Provider_Abstraction
Frontend_Expert -> Web_UI
Rust_Expert -> Desktop_App_Shell
Document_Orchestrator -> Local_Database_SQLite
Document_Orchestrator -> Local_File_Storage
AI_Provider_Abstraction -> Ollama_Local_AI
AI_Provider_Abstraction -> Mistral_AI
AI_Provider_Abstraction -> Aleph_Alpha
Web_UI -> Desktop_App_Shell
Local_Database_SQLite -> Local_File_Storage

pipeline AI_Assistentie_Behoefte {
  component "Tekst-based Assistentie" [0.25]
  component "Conversational AI Assistentie" [0.55]
}

evolve AI_Provider_Abstraction 0.68
evolve Expert_Mesh_Systeem 0.50
evolve Ollama_Local_AI 0.72

note "Local-First = Primary, Cloud = Fallback" [0.20, 0.20]
note "Expert Mesh: Concurrentievoordeel" [0.82, 0.38]
note "PII Pipeline: AVG/GDPR verplicht" [0.75, 0.40]
note "Commodity: Gebruik standaarden" [0.22, 0.90]

annotations [0.05, 0.05]
annotation 1,[0.48, 0.52] "AI Abstraction commoditiserend"
annotation 2,[0.78, 0.42] "Entry Actor: Gateway naar experts"
annotation 3,[0.72, 0.45] "Registry: Capability discovery"
annotation 4,[0.68, 0.40] "Document Orchestrator: Multi-expert workflows"
```

**Decorator Gids**:
- `(build)` — Genesis/Custom componenten die in-house gebouwd worden (driehoek marker)
- `(buy)` — Product/Commodity componenten van de markt (diamant marker)
- `(outsource)` — Componenten uitbesteed aan leveranciers (vierkant marker)
- `(inertia)` — Componenten met weerstand tegen verandering (verticale lijn)

</details>

---

## Evolutie Stages Referentie

| Stage | Volwassenheid | Kenmerken | Strategische Acties |
|-------|--------------|------------|---------------------|
| **Genesis** (0.00 - 0.25) | Nieuw, onzeker, snel veranderend | - Uniek en zeldzaam<br>- Slecht begrepen<br>- Snelle verandering<br>- Hoge onzekerheid<br>- Toekomstige waarde onzeker | - R&D focus<br>- Accepteer falen<br>- Experimenteer en verken<br>- Bouw in-house als strategisch |
| **Custom** (0.25 - 0.50) | Ontluikend, groeiend begrip | - Ambachtelijke oplossingen<br>- Custom ontwikkeling<br>- Concurrentievoordeel<br>- Vereist significante skills<br>- Evolueert nog snel | - Investeer in differentiatie<br>- Bouw custom bij concurrentievoordeel<br>- Bescherm IP (patenten) |
| **Product** (0.50 - 0.75) | Rijpende, goede huur diensten | - Producten met features<br>- Huur modellen<br>- Tragere evolutie<br>- Gedefinieerde practices | - Koop producten<br>- Vergelijk features<br>- Standaardiseer waar mogelijk |
| **Commodity** (0.75 - 1.00) | Geïndustrialiseerd, utility | - Gestandaardiseerd<br>- Volume operaties<br>- Hoge afwijkingskosten<br>- Utility diensten | - Gebruik commodity/utility<br>- Cloud diensten<br>- Focus op kosten efficiëntie |

---

## Component Inventaris

### User Needs (Bovenkant Map - Hoge Zichtbaarheid)

| Component | Zichtbaarheid | Evolutie | Stage | Beschrijving | Strategische Notities |
|-----------|---------------|----------|-------|--------------|------------------------|
| Ambtenaar | 0.95 | 0.30 | Genesis | Eindgebruiker van AI assistentie | Primaire stakeholder voor overheidsorganisaties |
| AI_Assistentie_Behoefte | 0.92 | 0.25 | Genesis | Behoefte aan AI-ondersteuning bij dagelijkse taken | Nieuwe user need in overheidscontext - unieke gelegenheid |

### Expert Mesh Componenten (Midden-Niveau Zichtbaarheid)

| Component | Zichtbaarheid | Evolutie | Stage | Beschrijving | Strategische Notities |
|-----------|---------------|----------|-------|--------------|------------------------|
| Expert_Mesh_Systeem | 0.82 | 0.38 | Custom | Uitbreidbaar expert agent systeem met capability discovery | **Concurrentievoordeel** - uniek voor overheid |
| Entry_Actor | 0.78 | 0.42 | Custom | Gateway voor externe requests naar expert mesh | Triage en delegation logic |
| Registry_Actor | 0.72 | 0.45 | Custom | Capability discovery en health monitoring | Dynamic expert discovery |
| Document_Orchestrator | 0.68 | 0.40 | Custom | Multi-expert workflow voor document creatie | Coördinatie van Research → Schrijver → Reviewer |
| Document_Improver | 0.65 | 0.38 | Custom | Document verbetering workflow | Kwaliteitsverbetering loops |

### Expert Agents (Specialiseerde Capabilities)

| Component | Zichtbaarheid | Evolutie | Stage | Beschrijving | Strategische Notities |
|-----------|---------------|----------|-------|--------------|------------------------|
| Research_Expert | 0.62 | 0.42 | Custom | Web research en data extraction expert | Domein expertise vereist |
| Frontend_Expert | 0.58 | 0.38 | Custom | Frontend technical expertise (UI/UX) | Web UI specialisatie |
| Rust_Expert | 0.55 | 0.35 | Custom | Rust systems programming expertise | Performance optimalisatie |
| Reviewer_Expert | 0.57 | 0.40 | Custom | Content review en feedback expert | Kwaliteitsborging |
| PII_Stripper_Expert | 0.52 | 0.35 | Custom | PII detectie en redactie | AVG/GDPR compliance differentiatie |
| Schrijver_Expert | 0.60 | 0.38 | Custom | Nederlandse content creatie expert | Lokalisatie differentiatie |
| Document_Experts | 0.63 | 0.40 | Custom | Geaggregeerde document expertise | Document workflow specialisatie |

### Ondersteunende Capabilities

| Component | Zichtbaarheid | Evolutie | Stage | Beschrijving | Strategische Notities |
|-----------|---------------|----------|-------|--------------|------------------------|
| PII_Detection_Pipeline | 0.75 | 0.40 | Custom | PII detectie en redactie voor AVG/GDPR | **Compliance verplichting** - onderscheidend |
| AI_Provider_Abstraction | 0.48 | 0.52 | Custom→Product | Uniforme interface voor alle AI providers | Beweegt naar product door standaardisatie |

### Infrastructuur Componenten (Lage Zichtbaarheid)

| Component | Zichtbaarheid | Evolutie | Stage | Beschrijving | Strategische Notities |
|-----------|---------------|----------|-------|--------------|------------------------|
| Web_UI | 0.35 | 0.68 | Product | React/TypeScript gebruikersinterface | Standaard web technologie |
| Desktop_App_Shell | 0.28 | 0.72 | Product | Tauri cross-platform desktop shell | Mature desktop framework |
| Local_Database_SQLite | 0.22 | 0.90 | Commodity | Embedded database voor lokale opslag | Gebruik commodity oplossing |
| Local_File_Storage | 0.18 | 0.92 | Commodity | Bestandssysteem voor document opslag | Native OS opslagsysteem |

### Externe Afhankelijkheden (Local-First Strategie)

| Component | Zichtbaarheid | Evolutie | Stage | Beschrijving | Strategische Notities |
|-----------|---------------|----------|-------|--------------|------------------------|
| Ollama_Local_AI | 0.25 | 0.65 | Product | **Primary** local model hosting voor volledige isolatie | Local-first strategie - primary provider |
| Mistral_AI | 0.30 | 0.72 | Product | Europese AI provider (Frankrijk) - **Performance fallback #1** | Performance fallback bij local limits |
| Aleph_Alpha | 0.28 | 0.70 | Product | Europese AI provider (Duitsland) - **Secondary fallback #2** | Enterprise fallback optie |

---

## Evolutie Analyse

### Componenten in Genesis (0.00 - 0.25)

**Nieuw, onbewezen, hoge onzekerheid**

| Component | Huidige Positie | Risico | Opportuniteit | Actie |
|-----------|------------------|--------|---------------|-------|
| AI_Assistentie_Behoefte | 0.25 | Medium - nieuwe user need in overheidscontext | Eerst mover advantage in overheids-AI | Accepteer iteratie, focus op user feedback |

**Strategische Aanbevelingen**:

- [x] Accepteer hoge faalratio bij experimenten
- [x] Investeer in R&D en iteratieve ontwikkeling
- [x] Bouw in-house als strategisch differentiator
- [x] Vermijd uitbesteden van core innovatie

### Componenten in Custom (0.25 - 0.50)

**Ontluikende practices, concurrentievoordeel**

| Component | Huidige Positie | Concurrentievoordeel? | Actie |
|-----------|------------------|------------------------|--------|
| Expert_Mesh_Systeem | 0.38 | **Ja** - uniek voor overheidscontext | Bouw en bescherm IP |
| Entry_Actor | 0.42 | **Ja** - gateway naar expert mesh | Bouw voor uitbreidbaarheid |
| Registry_Actor | 0.45 | **Ja** - capability discovery differentiatie | Bouw voor dynamic discovery |
| Document_Orchestrator | 0.40 | **Ja** - workflow coördinatie | Bouw voor domein-specifieke workflows |
| Document_Improver | 0.38 | **Ja** - kwaliteitsverbetering | Bouw voor differentiatie |
| PII_Detection_Pipeline | 0.40 | **Ja** - AVG/GDPR compliance | Bouw compliance-gericht |
| Research_Expert | 0.42 | **Ja** - web research expertise | Bouw met specialisten |
| Frontend_Expert | 0.38 | Nee - generieke frontend skills | Overweeg buy vs build |
| Rust_Expert | 0.35 | **Ja** - performance optimalisatie | Bouw voor lokale efficiëntie |
| Reviewer_Expert | 0.40 | Nee - generieke review skills | Overweeg hergebruik |
| PII_Stripper_Expert | 0.35 | **Ja** - privacy compliance | Bouw voor AVG/GDPR |
| Schrijver_Expert | 0.38 | **Ja** - Nederlandse taal expertise | Bouw voor lokalisatie |
| Document_Experts | 0.40 | **Ja** - document domein expertise | Bouw voor specialisatie |
| AI_Provider_Abstraction | 0.52 | Nee - aan het commoditiseren | Prepareer voor vendor lock-in ontwijking |

**Strategische Aanbevelingen**:

- [x] Bouw custom bij concurrentievoordeel
- [x] Investeer in specialistische skills
- [x] Bescherm IP (patenten, trade secrets)
- [x] Monitor evolutie snelheid - kan snel naar product bewegen
- [x] Build vs Buy beslissing kritiek hier

### Componenten in Product (0.50 - 0.75)

**Rijpende markt, feature differentiatie**

| Component | Huidige Positie | Markt Opties | Actie |
|-----------|------------------|---------------|--------|
| Web_UI | 0.68 | React, Vue, Angular ecosystem | Gebruik standaard frameworks |
| Desktop_App_Shell | 0.72 | Tauri, Electron (overweeg trade-offs) | Tauri gekozen voor kleinere binaries |
| Ollama_Local_AI | 0.65 | Ollama (enige echte speler) | Product-marktpositie |
| Mistral_AI | 0.72 | Mistral API, Azure, AWS | Multi-provider strategie |
| Aleph_Alpha | 0.70 | Aleph Alpha API direct | Europese enterprise optie |

**Strategische Aanbevelingen**:

- [x] Procureer van markt leiders
- [x] Vergelijk feature sets en prijzen
- [x] Standaardiseer op gemeenschappelijke platformen
- [x] Vermijd custom ontwikkeling tenzij kritisch

### Componenten in Commodity (0.75 - 1.00)

**Geïndustrialiseerd, utility diensten**

| Component | Huidige Positie | Commodity Provider | Actie |
|-----------|------------------|-------------------|--------|
| Local_Database_SQLite | 0.90 | SQLite (open source) | Gebruik embedded oplossing |
| Local_File_Storage | 0.92 | Native file systems | Gebruik OS native opslag |

**Strategische Aanbevelingen**:

- [x] Gebruik commodity/utility services
- [x] Focus op kosten efficiëntie, niet features
- [x] Vermijd custom ontwikkeling tegen alle kosten

---

## Build vs Buy Analyse

### Build (In-House Ontwikkeling)

**Kandidaten voor Bouwen**:

| Component | Evolutie Stage | Rationale | Risico | Investering |
|-----------|----------------|-----------|--------|--------------|
| Expert_Mesh_Systeem | Custom (0.38) | Uniek concurrentievoordeel voor overheid | Medium - skills beschikbaar | Reeds geïmplementeerd |
| Entry_Actor | Custom (0.42) | Gateway differentiatie | Low - uitbreidbaarheid | Reeds geïmplementeerd |
| Registry_Actor | Custom (0.45) | Dynamic discovery differentiatie | Low - standard pattern | Reeds geïmplementeerd |
| Document_Orchestrator | Custom (0.40) | Multi-expert workflows | Low - coördinatie logica | Reeds geïmplementeerd |
| Document_Improver | Custom (0.38) | Kwaliteitsverbetering differentiatie | Low - iteratief | Reeds geïmplementeerd |
| PII_Detection_Pipeline | Custom (0.40) | AVG/GDPR compliance verplichting | Medium - privacy expertise nodig | Reeds geïmplementeerd |
| Rust_Expert | Custom (0.35) | Performance differentiatie | Low - Rust team beschikbaar | Reeds geïmplementeerd |
| Schrijver_Expert | Custom (0.38) | Nederlandse taal expertise | Low - taal expertise beschikbaar | Reeds geïmplementeerd |
| AI_Provider_Abstraction | Custom→Product (0.52) | Vendor lock-in ontwijking | Medium - abstraction complexity | Reeds geïmplementeerd |

**Build Criteria**:

- ✅ Genesis/Custom stage (< 0.50 evolutie)
- ✅ Biedt concurrentievoordeel
- ✅ Core van business differentiator
- ✅ Geen geschikte markt alternatieven
- ✅ Skills beschikbaar of verwervbaar
- ✅ Strategisch IP eigendom belangrijk

### Buy (Procurement)

**Kandidaten voor Kopen**:

| Component | Evolutie Stage | Markt Opties | Rationale | Procurement Route |
|-----------|----------------|--------------|-----------|-------------------|
| Web_UI libraries | Product (0.68) | MUI, Tailwind, shadcn/ui | Snelheid tot markt | Open source (gratis) |
| Desktop_App_Shell | Product (0.72) | Tauri, Electron | Mature frameworks | Open source (gratis) |
| Ollama_Local_AI | Product (0.65) | Ollama project | Enige local model hoster | Open source (gratis) |
| Mistral_AI | Product (0.72) | Mistral API, Azure, AWS | Europese provider fallback | Direct contract |
| Aleph_Alpha | Product (0.70) | Aleph Alpha direct | Enterprise fallback optie EU | Direct contract |

**Buy Criteria**:

- ✅ Product/Commodity stage (> 0.50 evolutie)
- ✅ Volwassen markt met meerdere vendors
- ✅ Geen concurrentie differentiator
- ✅ Kosten van bouwen > kosten van kopen
- ✅ Time to market kritiek

---

## Inertia en Barrières

**Inertia** = weerstand tegen evolutie door bestaande practices, skills of investeringen

| Component | Huidige Evolutie | Gewenste Evolutie | Inertia Factor | Barrier Beschrijving | Mitigatie Strategie |
|-----------|-------------------|--------------------|----------------|---------------------|---------------------|
| Expert_Mesh_Systeem | 0.38 | 0.50 | **Medium** | Nieuw architectuur patroon voor team | Training, proof-of-concept |
| Local_First_Architectuur | 0.25 | 0.45 | **Hoog** | Cultuur verschuiving van cloud naar local | Change management, pilot projecten |
| PII_Detection_Pipeline | 0.40 | 0.55 | **Low** | Compliance is driver | Minimal - regelgeving dwingt verandering |
| AI_Provider_Abstraction | 0.52 | 0.68 | **Low** | Standaardisatie wordt omarmd | Multi-provider strategie behouden |

**Inertia Bronnen**:

- **Skills inertia**: Team expertise in cloud-native technieken (niet local-first)
- **Proces inertia**: Gevestigde workflows voor cloud development
- **Vendor lock-in**: Bestaande contracten met cloud providers
- **Culturele inertia**: "Cloud first" standaard in overheids-IT
- **Kapitaal investering**: Reeds geïnvesteerd in cloud infrastructuur

**De-risking Strategieën**:

- [x] Upskilling programs voor Rust en local-first architectuur
- [x] Pilot projecten om local-first aanpak te bewijzen
- [x] Gefaseerde migratie om risico te verminderen
- [x] Verandermanaging en communicatie

---

## Beweging en Evolutie Voorspellingen

**Evolutie Snelheid** = hoe snel componenten verwacht worden langs de evolutie as te bewegen

### Volgende 12 Maanden

| Component | Huidig | Voorspeld (12m) | Snelheid | Impact | Actie Vereist |
|-----------|---------|-----------------|----------|--------|----------------|
| AI_Provider_Abstraction | 0.52 | 0.58 | **Medium** | Provider standaardisatie neemt toe | Monitor markt voor AI abstractie lagen |
| Expert_Mesh_Systeem | 0.38 | 0.42 | **Langzaam** | Domein expertise blijft relevant | Continue investering in differentiatie |
| PII_Detection_Pipeline | 0.40 | 0.45 | **Langzaam** | Privacy compliance blijft maatwerk | Domein expertise uitbouwen |
| Ollama_Local_AI | 0.65 | 0.72 | **Medium** | Local model hosting wordt mainstream | Alternative providers evalueren |
| Registry_Actor | 0.45 | 0.50 | **Medium** | Capability discovery patroon verspreidt | Open source bijdrage overwegen |

### Volgende 24 Maanden

| Component | Huidig | Voorspeld (24m) | Snelheid | Impact | Actie Vereist |
|-----------|---------|-----------------|----------|--------|----------------|
| AI_Provider_Abstraction | 0.52 | 0.68 | **Snel** | Commoditiserend naar product | Voorbereiden op lock-in ontwijking |
| Expert_Mesh_Systeem | 0.38 | 0.50 | **Medium** | Mogelijke productisatie | IP bescherming, ecosystem bouwen |
| Mistral_AI | 0.72 | 0.80 | **Medium** | LLM markt commoditiseert | Multi-provider strategie behouden |
| Document_Orchestrator | 0.40 | 0.52 | **Medium** | Workflow orchestration standaardiseert | Prepareer voor markt adoptie |

**Strategische Implicaties**:

- [ ] Componenten bewegen Genesis → Custom: Investeer nu in R&D
- [ ] Componenten bewegen Custom → Product: Bereid je voor op buy vs build
- [ ] Componenten bewegen Product → Commodity: Plan cloud migratie
- [ ] Componenten met hoge snelheid: Monitor markt naulettend
- [ ] Componenten met inertia: Plan verandermanaging vroeg

---

## Nederlandse Overheid Context

### Europese AI Compliance

**Europa-First Strategie** (PRIN-004, bijgewerkt volgens HLD review):

| Provider | Prioriteit | EU-gebaseerd | Open Source | Data Residatie | Type | Uitzondering Vereist? |
|----------|------------|--------------|-------------|-----------------|------|---------------------|
| **Ollama (Local)** | **1e keuze (Primary)** | ✅ Local (EU) | ✅ | ✅ Local | Self-hosted | Nee |
| **Mistral AI** | **2e keuze (Fallback)** | ✅ FR | ✅ | ✅ EU (Parijs) | Cloud - Performance | Nee |
| **Aleph Alpha** | **3e keuze (Fallback)** | ✅ DE | ❌ | ✅ EU (Duitsland) | Cloud - Enterprise | Nee |
| OpenAI | Laatste | ❌ VS | ❌ | ⚠️ VS (EU compliant?) | Cloud | Ja (CTO+DPO) |
| Anthropic | Laatste | ❌ VS | ❌ | ⚠️ VS (EU compliant?) | Cloud | Ja (CTO+DPO) |

### Local-First Decision Tree

```
┌─────────────────┐
│ User Request    │
└────────┬────────┘
         │
         v
┌─────────────────┐     YES    ┌──────────────┐
│ Local model     │────────────▶│ Use Ollama   │
│ available?      │             │ (Primary)    │
└────────┬────────┘             └──────────────┘
         │ NO
         v
┌─────────────────┐     YES    ┌──────────────┐
│ Performance     │────────────▶│ Use Mistral  │
│ threshold      │             │ (Fallback 1)  │
│ exceeded?      │             └──────────────┘
└────────┬────────┘
         │ NO
         v
┌─────────────────┐             ┌──────────────┐
│ Fallback to     │────────────▶│ Use Aleph    │
│ secondary EU    │             │ (Fallback 2)  │
└─────────────────┘             └──────────────┘
```

### AVG/GDPR Compliance Mapping

| AVG Principe | Gerelateerde Componenten | Compliance Status | Gap Analyse |
|--------------|--------------------------|--------------------|--------------|
| Privacy by Design | PII_Detection_Pipeline, Local_Database_SQLite | ✅ | Pipeline geïmplementeerd |
| Recht op Vergetelheid | Local_Database_SQLite, Local_File_Storage | ✅ | EFF-010 geïmplementeerd |
| Recht op Portabiliteit | Export functionaliteit | ⚠️ | EFF-009 moet geïmplementeerd worden |
| Data Minimisation | PII_Detection_Pipeline | ✅ | Pipeline detecte en redigeert |
| Recht op Toegang | Local_Database_SQLite | ⚠️ | Data export functionaliteit nodig |

**HIGH-RISK AI Componenten**:

- [x] PII_Detection_Pipeline: AVG/GDPR compliance verplichting
- [x] Expert_Mesh_Systeem: Transparantie van AI beslissingen
- [x] AI_Provider_Abstraction: Provider transparantie
- [x] DPIA vereist voor productie deployment

---

## Dependencies en Value Chain

**Component Dependencies** (Update met Expert Mesh details):

```mermaid
flowchart TD
    AMB[Ambtenaar] --> AIB[AI_Assistentie_Behoefte<br/>Genesis - Bouw]
    AIB --> EMS[Expert_Mesh_Systeem<br/>Custom - Bouw]
    AIB --> PII[PII_Detection_Pipeline<br/>Custom - Bouw]
    
    EMS --> EA[Entry_Actor<br/>Custom - Bouw]
    EMS --> RA[Registry_Actor<br/>Custom - Bouw]
    EMS --> DOC[Document_Orchestrator<br/>Custom - Bouw]
    EMS --> IMP[Document_Improver<br/>Custom - Bouw]
    
    EA --> RE[Research_Expert<br/>Custom - Bouw]
    EA --> FE[Frontend_Expert<br/>Custom - Bouw]
    EA --> RustE[Rust_Expert<br/>Custom - Bouw]
    EA --> RV[Reviewer_Expert<br/>Custom - Bouw]
    EA --> PE[PII_Stripper_Expert<br/>Custom - Bouw]
    EA --> SE[Schrijver_Expert<br/>Custom - Bouw]
    
    EMS --> APA[AI_Provider_Abstraction<br/>Custom→Product - Bouw]
    
    APA --> OLA[Ollama_Local_AI<br/>Product - Kopen<br/>PRIMARY]
    APA --> MA[Mistral_AI<br/>Product - Kopen<br/>FALLBACK 1]
    APA --> AA[Aleph_Alpha<br/>Product - Kopen<br/>FALLBACK 2]
    
    EMS --> LDS[Local_Database_SQLite<br/>Commodity - Kopen]
    DOC --> LFS[Local_File_Storage<br/>Commodity - Kopen]
    
    style AMB fill:#FFE4B5
    style EMS fill:#E8F5E9
    style PII fill:#FFF3E0
    style LDS fill:#E3F2FD
    style LFS fill:#E3F2FD
    style OLA fill:#C8E6C9
```

**Expert Capability Mapping** (Update volgens codebase):

| Expert | Capabilities | Description | Status |
|--------|--------------|-------------|--------|
| FrontendExpert | "frontend", "css", "html", "design" | Frontend technical expertise | ✅ Live |
| ResearchExpert | "research", "http", "urls" | Web research en data extraction | ✅ Live |
| ReviewerExpert | "review", "critique", "quality" | Content review en feedback | ✅ Live |
| PIIStripperExpert | "pii", "privacy", "scrub" | PII detectie en redactie | ✅ Live |
| RustExpert | "rust", "systems", "memory" | Rust systems programming | ✅ Live |
| SchrijverExpert | "write", "content", "dutch" | Nederlandse content creatie | ✅ Live |

**Critical Path Analyse** (Update):

- [x] Entry_Actor is critical path naar alle expert functionaliteit
- [x] Registry_Actor is critical path voor capability discovery
- [x] AI_Provider_Abstraction is critical path naar AI verwerking
- [x] PII_Detection_Pipeline is critical path naar AVG/GDPR compliance
- [x] Document_Orchestrator is critical path voor multi-expert workflows
- [x] Single vendor risico: Mistral AI als fallback - gemitigeerd met Aleph Alpha

---

## Doctrine Assessments

### Doctrine Maturity Score (Update)

| Phase | Score | Assessment |
|-------|-------|------------|
| **I: Stop Self-Harm** | 7/10 | Common language aanwezig, maar documentation gap voor Expert Mesh |
| **II: Context Aware** | 6/10 | Open source bias aanwezig, inertia management nodig voor local-first |
| **III: Better for Less** | 5/10 | Flow optimalisatie nodig, observerbaarheid ontbreekt voor expert mesh |
| **IV: Continuously Evolving** | 4/10 | Ecosystem luisteren nodig, geen single culture |

**Gemiddelde Doctrine Maturity**: 5.5/10

### Verbeterpunten (Update)

**Phase I - Prioriteit: Hoog**
- [x] Common language: Expert mesh terminologie is niet universeel begrepen
- [x] Challenge assumptions: Local-first vs cloud-first assumptions uitdagen
- [x] Understand context: Stakeholder context beter documenteren

**Phase II - Prioriteit: Hoog**
- [x] Bias towards open: Expert_Mesh_Systeem als open source delen?
- [x] Manage inertia: Local-first cultuurverschillen adresseren
- [x] Move fast: Snellere iteratie tussen architectuur en implementatie
- [x] Use Standards: AI provider abstractie als standaard voorbereiden

**Phase III - Prioriteit: Medium**
- [x] Optimise flow: Expert workflow optimaliseren
- [x] Do better with less: Resource efficiëntie verbeteren
- [x] Set exceptional standards: Kwaliteitsstandaarden voor experts definieren

---

## Gameplay Patroon Analyse

### Toegepaste Gameplay Patterns

**Tower & Moat** (LG - Bescherm concurrentievoordeel):

- [x] **Expert_Mesh_Systeem**: Bouw custom, houd propriëtair
  - Unique waarde: Capability-based expert discovery voor overheidsdomeinen
  - Switching costs: Domein expertise integratie in experts
  - Ecosystem: Derde partijen kunnen eigen experts toevoegen (open source extensibility)

- [x] **PII_Detection_Pipeline**: Bouw compliance moat
  - Unique waarde: AVG/GDPR compliance voor Nederlandse context
  - Switching costs: Integratie met document workflows

**Open Source Play** (LG - Versnel commoditisatie):

- [x] **AI_Provider_Abstraction**: Open source specificatie publiceren
  - Doel: Voorkom vendor lock-in bij AI providers
  - Impact: Versnel adoptie van Europese providers

**Market Enablement** (LG - Marktactivering):

- [x] **Expert Registry**: Open capability discovery protocol
  - Doel: Laat derde partijen experts toevoegen
  - Impact: Ecosystem groei zonder centrale planning

**Sensing Engines** (Vroegtijdige waarschuwing):

- [x] Monitor LLM markt voor nieuwe Europese providers
- [x] Track local model hosting (Ollama concurrentie)
- [x] Watch voor AI agent framework standaardisatie

### Mogelijke Gameplay Patterns (Toekomst)

**Differentiation** (N):
- Nederlandse taal expertise (Schrijver_Expert) onderscheidt van US-centric AI tools
- Overheidsdocument domeinkennis als differentiator

**Co-operation** (N):
- Stand together met andere gemeenten voor Europese AI sourcing
- Shared expertise development voor overheids-AI use cases

---

## Climatic Patroon Analyse

### Externe Krachten die de Vorm Geven

**Alles Evolueert** (1.1):
- LLM markt evolueert snel van Genesis naar Product
- Local model hosting (Ollama) beweegt naar Product stage
- AI agent frameworks evolueren naar standaardisatie

**Evolutiesnelheden Varieren per Ecosysteem** (1.2):
- Consumer AI: Snelle evolutie (maanden)
- Overheids-AI: Trage evolutie (jaren) door regelgeving
- Impact: Onze Expert Mesh beweegt langzamer dan marktstandaard

**Co-evolutie** (1.6):
- Expert Mesh co-evolueert met LLM commoditisatie
- PII compliance eisen co-evolueren met AI regelgeving (EU AI Act)

**Innovatie Golven** (1.7):
- We zitten in tweede golf van AI adoptie in overheid
- Chasm: Productie use cases vereisen AVG/GDPR compliance
- Crossing strategy: PII Pipeline als compliance bridge

**Commoditisering ≠ Centralisatie** (1.8):
- Local AI hosting blijft gedecentraliseerd (per device)
- AI provider abstractie voorkomt centrale vendor lock-in

**Jevons Paradox** (2.2):
- Efficiëntere local AI → meer AI use cases → hoger totaal gebruik
- Implicatie: Plan voor schaalvergroting, niet kostenreductie

**Kapitaal Stroom** (2.3):
- VC kapitaal stroomt naar AI agents en orchestration
- Impact: Expert Mesh concurrentie neemt toe in 12-24 maanden

**Creatieve Destructie** (2.4):
- LLM commoditisering kan custom expert agents minder relevant maken
- Mitigatie: Focus op domein expertise niet algemene intelligence

---

## Risico Analyse

### High-Risk Gebieden (Update)

| Risico | Component(en) | Waarschijnlijkheid | Impact | Mitigatie |
|--------|---------------|-------------------|--------|------------|
| **Single vendor dependency** | Mistral_AI | Medium | High | Multi-provider strategie (Ollama primary, Aleph Alpha secondary) |
| **Genesis component failure** | Expert_Mesh_Systeem | Low | High | Iteratieve ontwikkeling, fail-fast, fallback naar direct AI calls |
| **Rapid commoditization** | AI_Provider_Abstraction | High | Medium | Open source specificatie, vendor agnostisch ontwerp |
| **Skills gap** | Rust expertise | Medium | Medium | Training, hiring, knowledge sharing |
| **Regulatory change** | AVG/GDPR wijzigingen, EU AI Act | Low | High | Continue monitoring, flexibele architectuur |
| **Expert orchestration complexity** | Document_Orchestrator | Medium | High | Unit testing, integration testing, monitoring |
| **Local model performance** | Ollama_Local_AI | Medium | High | Fallback strategy, performance thresholds |

### Opportuniteiten (Update)

| Opportuniteit | Componenten | Potentiële Waarde | Investering Vereist | Actie Plan |
|----------------|--------------|--------------------|---------------------|-------------|
| Europese AI leiderschap | Expert Mesh + EU providers | High | Medium | Positioneer als EU-first AI oplossing |
| Gemeente ecosystem | Expert Mesh extensies | High | Low | Derde partijen enablen via open registry |
| Open source bijdragen | AI_Provider_Abstraction | Medium | Low | Publiceer als standaard |
| Overheidsdocument expertise | Document_Experts | High | Medium | Specialiseer in overheidsformats |
| Local-first movement | Local-First architectuur | High | High | Pioneer in overheid local-first AI |

---

## Aanbevelingen

### Directe Acties (0-3 maanden) - Update naar HLD Review

1. **Update Container Diagram (DIAG-002)**
   - **Component**: Expert_Mesh_Systeem, Entry_Actor, Registry_Actor
   - **Rationale**: Architectuurdocumentatie loopt achter op implementatie
   - **Investering**: 1-2 dagen architectuur werk
   - **Eigenaar**: Enterprise Architect
   - **Succescriteria**: DIAG-002 bevat volledige Expert Mesh architectuur

2. **Documenteer Registry Pattern**
   - **Component**: Registry_Actor, capability discovery
   - **Rationale**: Expert discovery mechanisme niet gedocumenteerd
   - **Investering**: 2-3 dagen documentatie
   - **Eigenaar**: Architect
   - **Succescriteria**: Registry protocol gedocumenteerd in ADR

3. **Implementeer Observability Strategy**
   - **Component**: Expert_Mesh_Systeem (alle experts)
   - **Rationale**: Monitoring ontbreekt voor distributed expert systeem
   - **Investering**: 4-6 weken ontwikkeling
   - **Eigenaar**: SRE/DevOps team
   - **Succescriteria**: Distributed tracing, metrics, logging voor expert mesh

4. **Definieer Performance Thresholds**
   - **Component**: AI_Provider_Abstraction (fallback logic)
   - **Rationale**: Local-first fallback criteria niet gedefinieerd
   - **Investering**: 1 week specificatie + implementatie
   - **Eigenaar**: Architect
   - **Succescriteria**: Thresholds documenteerd en geïmplementeerd

### Korte Termijn Acties (3-12 maanden)

1. **PII Pipeline Validatie**
   - **Component**: PII_Detection_Pipeline
   - **Rationale**: AVG/GDPR compliance moet gevalideerd worden
   - **Investering**: 4-8 weken + DPO review
   - **Eigenaar**: Security Architect
   - **Succescriteria**: DPIA goedgekeurd, accuracy >90%

2. **Expert Ecosystem Uitbreiding**
   - **Component**: Expert_Mesh_Systeem
   - **Rationale**: Maximaliseer concurrentievoordeel via derde partijen
   - **Investering**: 8-12 weken ontwikkeling + documentatie
   - **Eigenaar**: Product Owner
   - **Succescriteria**: Externe developer onboarding docs live

3. **Utrecht Schalingsvalidatie**
   - **Component**: Alle componenten (load testing)
   - **Rationale**: Ondersteuning voor 6000 medewerkers onzeker
   - **Investering**: 4-6 weken testing + optimalisatie
   - **Eigenaar**: QA team
   - **Succescriteria**: Load test resultaten show 1000+ concurrent users

### Lange Termijn Strategische Acties (12-24 maanden)

1. **AI Abstraction Productisatie**
   - **Component**: AI_Provider_Abstraction
   - **Rationale**: Component commoditiseert, prepareer voor marktpositie
   - **Investering**: 12-18 maanden (indien gekozen)
   - **Eigenaar**: Product Management
   - **Succescriteria**: Open source release, community adoptie

2. **Multi-Instance Expert Mesh**
   - **Component**: Expert_Mesh_Systeem
   - **Rationale**: Ondersteun grotere gemeentes (Utrecht schaling)
   - **Investering**: 6-12 maanden architectuur werk
   - **Eigenaar**: Architect
   - **Succescriteria**: Distributed mesh over meerdere instanties

3. **EU AI Act Compliance**
   - **Component**: Alle AI componenten
   - **Rationale**: Voorbereiding op aankomende EU AI Act regelgeving
   - **Investering**: Continue monitoring + aanpassingen
   - **Eigenaar**: DPO + Architect
   - **Succescriteria**: Compliance assessment voltooid voor implementation deadline

---

## HLD Review Follow-up

### Toegepaste Correcties vanuit ARC-000-HLDR-v1.0

| Correctie | Status | Impact |
|-----------|--------|--------|
| BLOCKING-01: Expert Mesh in container diagram | ⚠️ Open | Nog niet toegepast in DIAG-002 |
| BLOCKING-02: Local-First strategy gecorrigeerd | ✅ Toegepast | Ollama = primary, Mistral = fallback |
| BLOCKING-03: Registry pattern gedocumenteerd | ⚠️ Open | Wordt gedocumenteerd in deze Wardley Map |
| ADVISORY-01: Observability strategy | ⚠️ Open | Aanbevolen in directe acties |
| ADVISORY-02: Performance thresholds | ⚠️ Open | Aanbevolen in directe acties |
| ADVISORY-03: Expert lifecycle management | ✅ Gedocumenteerd | Expert registry pattern beschreven |

### Overgebleven Acties naar Detailed Design

1. **Observability Container toevoegen** aan architectuur
2. **Performance Thresholds definiëren** voor cloud fallback
3. **Expert Lifecycle documenteren** (spawn/terminate/recovery)
4. **Sequence Diagrams** maken voor expert workflows

---

## Map Versiebeheer

**Versie Geschiedenis**:

| Versie | Datum | Auteur | Wijzigingen | Rationale |
|---------|-------|--------|-------------|-----------|
| v1.0 (WARD-001) | 2026-05-07 | ArcKit AI | Initiële map | Strategische positie van Local-First AI Assistant |
| v1.0 (WARD-002) | 2026-05-07 | ArcKit AI | Update met Expert Mesh details, Local-First correctie, Doctrine/Gameplay/Climatic analyse | HLD review findings verwerken |

**Volgende Review Datum**: 2026-08-07

**Review Frequentie**: Per kwartaal

---

## Externe Referenties

| Document | Type | Bron | Kernextracties | Pad |
|----------|------|------|-----------------|-----|
| ARC-000-PRIN-v1.0.md | Architecture Principles | ArcKit | Europa-first AI, Security by Design | projects/000-global/ |
| ARC-000-REQS-v1.0.md | Requirements | ArcKit | Functionele en non-functionele eisen | projects/000-global/ |
| ARC-000-HLDR-v1.0.md | HLD Review | ArcKit | Expert mesh implementatie status, Local-First correctie | projects/000-local-assistant/ |
| ARC-000-DIAG-002-v1.0.md | Container Diagram | ArcKit | Component architectuur | projects/000-global/diagrams/ |
| ARC-000-DATA-v1.0.md | Data Model | ArcKit | Data componenten | projects/000-global/ |

---

**Gegenereerd door**: ArcKit `/arckit:wardley` command
**Gegenereerd op**: 2026-05-07
**ArcKit Versie**: 4.3.1
**Project**: Local-First AI Assistant (Project 000)
**AI Model**: Claude Opus 4.7
**Generatie Context**: Wardley Map update gegenereerd op basis van principles (ARC-000-PRIN), requirements (ARC-000-REQS), HLD review (ARC-000-HLDR), en doctrine/gameplay/climatic pattern referenties