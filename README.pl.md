<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor-storage.png" alt="Better Resource Monitor" width="1039" height="86">
</p>


<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>Monitoruj CPU, pamięć, dysk, GPU i sieć z paska menu swojego Maca.</strong>
</p>

<!-- README-LANG-START -->

<p align="center">
  <a href="README.md">English</a> •
  <a href="README.es.md">Español</a> •
  Polski •
  <a href="README.pt-br.md">Português (Brasil)</a> •
  <a href="README.zh-cn.md">简体中文</a>
</p>

<!-- README-LANG-END -->


<p align="center">
  <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/appstore-pl.webp" alt="Pobierz z Mac App Store" width="270" height="65"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/macos-pl.webp" alt="Pobierz wersję na macOS z GitHub Releases" width="270" height="65"></a>
</p>

## Dlaczego

Better Resource Monitor jest dla osób, które chcą po prostu szybko i bez zbędnego szumu mieć oko na swojego Maca.

Pokazuje użycie CPU, pamięci, dysku, GPU i sieci bezpośrednio na pasku menu, dzięki czemu zauważysz nietypowe obciążenie bez przerywania pracy i bez otwierania Monitora aktywności.

Został zaprojektowany tak, by działać przez cały dzień i nie stać się częścią problemu: poniżej 0,1% CPU na Apple Silicon, około 15 MB pamięci i zero żądań sieciowych.

Jeśli szukasz alternatywy dla iStat Menus, to dobre miejsce: lekki monitor codziennych wskaźników, z niskim narzutem i bez telemetrii.

## Dla kogo

Wybierz Better Resource Monitor, jeśli chcesz lekkiego monitora na pasku menu Maca, który na co dzień szybko pokaże użycie CPU, pamięci, dysku, GPU i sieci. Jest darmowy, ma otwarty kod źródłowy, działa w piaskownicy (sandbox) i offline, a do tego nie wymaga hasła administratora ani pomocnika z uprawnieniami roota. Działa na Macach z procesorami Intel i Apple Silicon z systemem macOS 13 lub nowszym.

Wybierz Stats lub iStat Menus, jeśli potrzebujesz danych o wentylatorach, temperaturach i baterii, szczegółowej historii czujników lub szerszej kontroli nad sprzętem. Better Resource Monitor celowo skupia się na podstawowych wskaźnikach, które sprawdza się codziennie.

## FAQ: alternatywa dla iStat Menus

### Czy Better Resource Monitor może zastąpić iStat Menus?

Tak, jeśli zależy Ci na widoczności i codziennej stabilności. iStat Menus świetnie sprawdza się przy dogłębnej kontroli systemu; Better Resource Monitor służy do tego, by najważniejsze liczby były czytelne przez cały dzień przy mniejszej konfiguracji.

### Czy jest darmowy?

Tak. Better Resource Monitor jest darmowy i udostępniony na licencji MIT. Obie dystrybucje mają te same funkcje monitorowania; różnią się sposobem aktualizacji.

### Czy wysyła telemetrię lub zbiera dane o użyciu?

Nie. Better Resource Monitor nie wykonuje żadnych żądań sieciowych. Nie wysyła żadnej analityki ani telemetrii.

### Czy nie obciąża baterii?

Nie. Został zaprojektowany tak, by działać w tle przy bardzo niskim wpływie na system i pozostać użyteczny podczas codziennej pracy.

## Alerty

Gdy opcja **Pokaż kolory ostrzeżeń** jest włączona, ikona na pasku menu zmienia kolor na pomarańczowy, kiedy widoczne użycie CPU, pamięci lub GPU osiągnie **81%** albo gdy ilość wolnego miejsca na dysku spadnie poniżej **10 GB**. Wraca do normalnego wyglądu, gdy wskaźniki procentowe są poniżej 81%, a na dysku jest co najmniej 10 GB wolnego miejsca.

Ostrzeżenie o wolnym miejscu dotyczy dysku wybranego w podmenu **Dysk**. Domyślnie monitorowany jest dysk zawierający folder domowy użytkownika.

Alerty są wyłącznie wizualne. Prędkość sieci i ukryte wskaźniki ich nie wywołują. Próg jest stały, ale kolory ostrzeżeń można wyłączyć w menu na pasku.


## Instalacja

Pobierz aplikację z <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> (z automatycznymi aktualizacjami) lub pobierz plik `.dmg` z <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank">GitHub Releases</a> (bez automatycznych aktualizacji; każdą nową wersję trzeba pobrać i zainstalować ręcznie).

### Homebrew

```bash
brew tap alexx855/better-resource-monitor https://github.com/alexx855/better-resource-monitor
brew install --cask alexx855/better-resource-monitor/better-resource-monitor
```

`brew upgrade --cask better-resource-monitor` instaluje nowe wersje po aktualizacji definicji cask.

### Zgodność

Działa na Macach z procesorami Intel i Apple Silicon z systemem macOS Ventura 13 lub nowszym.

### Kompilacja ze źródeł

Potrzebujesz <a href="https://v2.tauri.app/start/prerequisites/" target="_blank">wymagań wstępnych Tauri v2</a> oraz <a href="https://pnpm.io/" target="_blank">pnpm</a>.

```bash
git clone https://github.com/alexx855/better-resource-monitor.git
cd better-resource-monitor
pnpm install
pnpm tauri build
```

Do testów lokalnych użyj `pnpm dev`. Niepodpisany pakiet macOS zbudowany lokalnie nie jest wspieraną wersją do instalacji.

Informacje o rozwoju i weryfikacji znajdziesz w [przewodniku dla współtwórców](https://github.com/alexx855/better-resource-monitor/blob/main/CONTRIBUTING.md).

## Porównanie

Na szybki start:

- [Better Resource Monitor vs iStat Menus](https://better-resource-monitor.alexpedersen.dev/pl/comparison/vs-istat-menus/)
- [Better Resource Monitor vs Stats](https://better-resource-monitor.alexpedersen.dev/pl/comparison/vs-stats/)
- [Better Resource Monitor vs Eul](https://better-resource-monitor.alexpedersen.dev/pl/comparison/vs-eul/)

<table>
  <thead>
    <tr>
      <th width="20%">Funkcja</th>
      <th width="20%">Better Resource Monitor</th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pl/comparison/vs-stats/">Stats</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pl/comparison/vs-eul/">Eul</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pl/comparison/vs-istat-menus/">iStat Menus</a></th>
    </tr>
  </thead>
  <tbody>
    <tr><th scope="row">Mac App Store</th><td align="center">Tak (pełne funkcje)</td><td align="center">Nie</td><td align="center">Ograniczone</td><td align="center">Ograniczone</td></tr>
    <tr><th scope="row">Hasło administratora / uprawnienia</th><td align="center">Brak (sandbox)</td><td align="center">Wymaga pomocnika z uprawnieniami roota</td><td align="center">Brak</td><td align="center">Wymaga pomocnika z uprawnieniami roota</td></tr>
    <tr><th scope="row">Stabilność API GPU</th><td align="center">Publiczne API</td><td align="center">Prywatne API</td><td align="center">Prywatne API</td><td align="center">Własnościowe</td></tr>
    <tr><th scope="row">Zużycie pamięci</th><td align="center">~15 MB</td><td align="center">~50 MB</td><td align="center">~40 MB</td><td align="center">~100+ MB</td></tr>
    <tr><th scope="row">Wpływ na CPU / energię</th><td align="center">&lt; 0,1%</td><td align="center">~1%</td><td align="center">Wysoki (seria M)</td><td align="center">~1%</td></tr>
    <tr><th scope="row">Rozmiar aplikacji</th><td align="center">&lt; 7 MB</td><td align="center">~25 MB</td><td align="center">~5 MB</td><td align="center">~65 MB</td></tr>
    <tr><th scope="row">Prywatność/telemetria</th><td align="center">100% offline</td><td align="center">Offline</td><td align="center">Offline</td><td align="center">Analityka</td></tr>
    <tr><th scope="row">Status</th><td align="center">Aktywny</td><td align="center">Aktywny</td><td align="center">Nierozwijany</td><td align="center">Aktywny</td></tr>
    <tr><th scope="row">Język</th><td align="center">Rust</td><td align="center">Swift / C++</td><td align="center">Swift</td><td align="center">Obj-C / Swift</td></tr>
    <tr><th scope="row">Cena</th><td align="center">Za darmo</td><td align="center">Za darmo</td><td align="center">Za darmo</td><td align="center">$14.99</td></tr>
    <tr><th scope="row">Licencja</th><td align="center">MIT</td><td align="center">MIT</td><td align="center">MIT</td><td align="center">Własnościowa</td></tr>
  </tbody>
</table>

> Dane innych aplikacji to przybliżone szacunki. Wyniki mogą się różnić.

## Podziękowania


- <a href="https://github.com/phosphor-icons" target="_blank">Phosphor Icons</a> - Zestaw ikon używany na pasku menu
- <a href="https://alexpedersen.dev/" target="_blank">Alex Pedersen</a> - Opiekun projektu
