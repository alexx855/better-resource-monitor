# Sitemap Strategy for Silicon Monitor

## 1. Hierarchical Structure

This sitemap is designed to capture high-intent traffic from users looking for macOS system monitoring tools, specifically for Apple Silicon.

```markdown
Homepage (monitor.alexpedersen.dev)
├── Features
│   ├── GPU Monitoring (Deep dive into M-Series GPU residency vs utilization)
│   ├── Performance & Efficiency (Rust architecture, memory usage, battery impact)
│   └── Customization (Theming, configuring stats)
├── Comparisons
│   ├── Silicon Monitor vs iStat Menus (Free vs Paid, Open Source vs Closed)
│   └── Silicon Monitor vs Electron Apps (Native resource usage vs Web technologies)
└── Guide
    ├── Installation (Brew, Binary, Build from source)
    └── Understanding Metrics (Explaining "GPU Residency", IOReport)
```

## 2. Strategic Justification (Search Intent)

*   **Features / GPU Monitoring**:
    *   **Intent**: Informational/Transactional. Users specifically searching for "how to monitor M1 GPU" or "mac task manager gpu". The unique selling point of Silicon Monitor is the `IOReport` implementation for accurate residency data, which addresses a specific gap in existing free tools.
*   **Features / Performance & Efficiency**:
    *   **Intent**: Informational. Users frustrated with heavy background apps ("why is activity monitor using so much cpu"). highlighting "Rust" and "Native" appeals to developer/power-user audience.
*   **Comparisons / Vs iStat Menus**:
    *   **Intent**: Commercial Investigation. Users looking for a free alternative to the market leader. Captures "istat menus alternative" or "free mac system monitor" queries.
*   **Comparisons / Vs Electron Apps**:
    *   **Intent**: Informational/Educational. Educates users on the "Electron Tax" and justifies switching from other popular but heavy tools.
*   **Guide / Understanding Metrics**:
    *   **Intent**: Informational. specific queries about "what is gpu residency" or technical users validating the tool's accuracy.

## 3. Cross-Linking Strategy

To distribute link equity and guide the user journey towards the "Download" (Conversion) goal:

*   **Contextual Feature Linking**: The **"Comparisons / Vs iStat Menus"** page should link directly to the **"Features / GPU Monitoring"** page when discussing GPU stats. This validates the comparison claim with deep technical content.
*   **Efficiency Proof**: The **"Comparisons / Vs Electron Apps"** page should link to **"Features / Performance"** to provide data backing up the "bloat" claims.
*   **Download Anchors**: Every deep page (Features, Comparisons) must have a persistent "Download" or "Get Started" CTA in the header/footer or as a sticky element, linking to the **"Guide / Installation"** page or the GitHub release directly.
*   **Breadcrumbs**: Implement breadcrumb navigation (e.g., `Home > Comparisons > Vs iStat Menus`) to allow users to easily navigate back up the hierarchy (Mental Model: "I am deep in a specific topic, let me see other comparisons").

## 4. Labeling (Mental Model)

We use labels that reflect *what the user gets* rather than internal architectural terms.

| Internal Jargon | Proposed Navigation Label | Why? |
| :--- | :--- | :--- |
| `Capabilities` | **Features** | Standard term users look for to see "what it does". |
| `Architecture` | **Performance** | Users care about the *result* (fast/light), not the *method* (architecture). |
| `Competitors` | **Vs Others** or **Comparisons** | Direct and honest. "Vs Others" piques curiosity. |
| `Documentation` | **Guide** | "Documentation" sounds like a manual. "Guide" sounds helpful and approachable. |
| `IOReport Implementation` | **How it Works** | Technical users want to know the "secret sauce" but "IOReport" is too obscure for the nav bar. |
