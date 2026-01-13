# Sitemap Strategy for Silicon Monitor

## Project Context
*   **Project Type**: Product Landing Page & Documentation Site
*   **Target Audience**: macOS users (specifically Apple Silicon M1/M2/M3 owners), Developers, Power Users looking for system monitoring tools.
*   **Primary Goal**: Software Installation / User Acquisition

## 1. Hierarchical Structure

The sitemap is designed to keep all content within 3 clicks of the homepage.

```markdown
*   **Home** (`/`)
    *   *Goal*: Immediate understanding of value proposition + Download CTA.
*   **Features** (`/features`)
    *   **CPU Monitoring** (`/features/cpu`)
    *   **GPU Monitoring** (`/features/gpu`)
    *   **Memory & Swap** (`/features/memory`)
    *   **Network Activity** (`/features/network`)
    *   **Battery Life** (`/features/battery`)
*   **Documentation** (`/docs`)
    *   **Getting Started** (`/docs/getting-started`)
        *   Installation (`/docs/getting-started/installation`)
        *   First Run (`/docs/getting-started/first-run`)
    *   **Configuration** (`/docs/configuration`)
        *   Menu Bar Settings (`/docs/configuration/menu-bar`)
        *   Theme Customization (`/docs/configuration/themes`)
    *   **Troubleshooting** (`/docs/troubleshooting`)
        *   Common Issues (`/docs/troubleshooting/common-issues`)
        *   FAQ (`/docs/troubleshooting/faq`)
*   **Comparisons** (`/comparisons`)
    *   **Silicon Monitor vs Stats** (`/comparisons/silicon-monitor-vs-stats`)
    *   **Silicon Monitor vs iStat Menus** (`/comparisons/silicon-monitor-vs-istat-menus`)
*   **Changelog** (`/changelog`)
```

## 2. Justification (Search Intent Logic)

*   **Home**: Targets high-level queries like "mac menu bar monitor", "free mac system monitor". The page should satisfy the intent of "what is the best free monitor?" and "where can I download it?".
*   **Features (`/features/*`)**: Targets specific long-tail keywords.
    *   *GPU Monitoring*: Targets "monitor m1 gpu usage", "apple silicon gpu tool". This is a unique selling point (USP) as many tools fail here.
    *   *Memory*: Targets "mac memory monitor", "swap usage tracker".
*   **Documentation (`/docs/*`)**: Targets informational/support queries.
    *   *Installation/Configuration*: Targets existing users needing help ("how to configure silicon monitor").
    *   *Troubleshooting*: Targets problem-solving queries ("silicon monitor not showing gpu").
*   **Comparisons (`/comparisons/*`)**: Targets consideration-stage users comparing options.
    *   "Silicon Monitor vs iStat Menus" is a high-intent keyword for users looking for free alternatives to paid software.
    *   "Silicon Monitor vs Stats" targets users looking for the most lightweight option.

## 3. Cross-Linking Strategy

To enhance authority and user journey:

*   **From Features to Docs**: Each feature page (e.g., *GPU Monitoring*) should link to the relevant configuration guide (e.g., *Menu Bar Settings*) to show users how to enable it immediately.
*   **From Comparisons to Features**: Comparison pages should deep-link to specific Feature pages to substantiate claims (e.g., "Unlike X, we track native residency" -> link to *GPU Monitoring*).
*   **From Docs to Support**: Troubleshooting pages should link to the GitHub Issues page or a contact form for unresolved issues.
*   **Breadcrumbs**: Implement breadcrumb navigation on all deep pages (e.g., `Home > Docs > Configuration > Menu Bar`) to reinforce hierarchy.

## 4. Labeling Strategy

Labels are chosen to match the user's mental model:

*   **"Features"** over "Capabilities": "Features" is the standard term users look for when evaluating software.
*   **"Docs"** over "Knowledge Base": The audience includes developers; "Docs" is familiar and implies technical detail.
*   **"Comparisons"** over "Alternatives": Users often search for "vs", so "Comparisons" sets the right expectation for a direct head-to-head analysis.
*   **"Get Started"** over "Onboarding": Action-oriented language encourages the user to begin.
