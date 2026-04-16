<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="86">
</p>


<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>Um monitor de sistema para a barra de menus/bandeja do macOS.</strong>
</p>

<!-- README-LANG-START -->

<p align="center">
  <a href="README.md">English</a> •
  <a href="README.es.md">Español</a> •
  Português (Brasil) •
  <a href="README.zh-cn.md">简体中文</a>
</p>

<!-- README-LANG-END -->


<p align="center">
  <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/appstore-pt-br.webp" alt="Baixar na Mac App Store" width="270" height="65"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases/download/v1.1.0/Better.Resource.Monitor_1.1.0_aarch64.dmg" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/macos-pt-br.webp" alt="Baixar para macOS no GitHub Releases" width="270" height="65"></a>
</p>

## Por que usar

Sem senha de administrador. Sem helpers privilegiados. Sem APIs privadas. Sem ícone no Dock.

CPU, memória, GPU e rede na barra de menus. Roda em sandbox. Na Mac App Store com todos os recursos.

## Comparação

<table>
  <thead>
    <tr>
      <th width="20%">Recurso</th>
      <th width="20%">Better Resource Monitor</th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pt-br/vs-stats">Stats</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pt-br/vs-eul">Eul</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pt-br/vs-istat-menus">iStat Menus</a></th>
    </tr>
  </thead>
  <tbody>
    <tr><th scope="row">Mac App Store</th><td align="center">Sim (recursos completos)</td><td align="center">Não</td><td align="center">Limitado</td><td align="center">Limitado</td></tr>
    <tr><th scope="row">Senha de Admin / Privilégios</th><td align="center">Nenhum (sandboxed)</td><td align="center">Requer helper root</td><td align="center">Nenhum</td><td align="center">Requer helper root</td></tr>
    <tr><th scope="row">Estabilidade da API de GPU</th><td align="center">API Pública</td><td align="center">API Privada</td><td align="center">API Privada</td><td align="center">Proprietário</td></tr>
    <tr><th scope="row">Consumo de Memória</th><td align="center">~15 MB</td><td align="center">~50 MB</td><td align="center">~40 MB</td><td align="center">~100+ MB</td></tr>
    <tr><th scope="row">Impacto na CPU / Energia</th><td align="center">&lt; 0.1%</td><td align="center">~1%</td><td align="center">Alto (série M)</td><td align="center">~1%</td></tr>
    <tr><th scope="row">Tamanho do App</th><td align="center">&lt; 7 MB</td><td align="center">~25 MB</td><td align="center">~5 MB</td><td align="center">~65 MB</td></tr>
    <tr><th scope="row">Privacidade/Telemetria</th><td align="center">100% offline</td><td align="center">Offline</td><td align="center">Offline</td><td align="center">Analytics</td></tr>
    <tr><th scope="row">Status</th><td align="center">Ativo</td><td align="center">Ativo</td><td align="center">Sem manutenção</td><td align="center">Ativo</td></tr>
    <tr><th scope="row">Linguagem</th><td align="center">Rust</td><td align="center">Swift / C++</td><td align="center">Swift</td><td align="center">Obj-C / Swift</td></tr>
    <tr><th scope="row">Preço</th><td align="center">Grátis</td><td align="center">Grátis</td><td align="center">Grátis</td><td align="center">$14.99</td></tr>
    <tr><th scope="row">Licença</th><td align="center">MIT</td><td align="center">MIT</td><td align="center">MIT</td><td align="center">Proprietário</td></tr>
  </tbody>
</table>

> Os números de terceiros são estimativas aproximadas. Sua experiência pode variar.

## Instalação

Obtenha na <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> (inclui atualizações automáticas) ou baixe o `.dmg` no <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank">GitHub Releases</a>.

### Compatibilidade

Funciona em Macs Intel e Apple Silicon com macOS Ventura 13 ou mais recente.

### Status do Linux

As versões oficiais para Linux estão em pausa por enquanto. No Ubuntu/GNOME Wayland, a cadeia upstream de appindicator vaza texturas do compositor quando o ícone da bandeja é atualizado, e isso pode degradar toda a área de trabalho com o tempo. Este app já reduz a frequência das atualizações do ícone da bandeja, mas isso apenas desacelera o vazamento. Compilações a partir do código-fonte ainda funcionam no Linux se você quiser experimentar, mas publicar novas versões `.deb` seria irresponsável até que o Tauri adicione suporte a KSNI. Veja a <a href="https://github.com/alexx855/better-resource-monitor/issues/10" target="_blank">issue #10</a> e <a href="https://github.com/tauri-apps/tauri/issues/11293" target="_blank">tauri-apps/tauri#11293</a> para mais detalhes.

### Compilar a partir do código-fonte

Você precisará dos <a href="https://v2.tauri.app/start/prerequisites/" target="_blank">pré-requisitos do Tauri v2</a> e do <a href="https://pnpm.io/" target="_blank">pnpm</a>.

```bash
git clone https://github.com/alexx855/better-resource-monitor.git
cd better-resource-monitor
pnpm install
pnpm tauri build
```

### Desenvolvimento

```bash
# Executar em modo de desenvolvimento com hot reload
pnpm tauri dev

# Executar testes
cd src-tauri && cargo test

# Executar testes com cobertura (requer cargo-llvm-cov)
cargo install cargo-llvm-cov
cd src-tauri && cargo llvm-cov --lib --html --output-dir coverage/
```

## Créditos


- <a href="https://github.com/phosphor-icons" target="_blank">Phosphor Icons</a> - Conjunto de ícones usado na bandeja
- <a href="https://alexpedersen.dev/" target="_blank">Alex Pedersen</a> - Mantenedor
