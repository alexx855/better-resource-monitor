<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="86">
</p>


<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>Monitore CPU, memória, armazenamento, GPU e rede pela barra de menus do Mac.</strong>
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
  <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/macos-pt-br.webp" alt="Baixar para macOS no GitHub Releases" width="270" height="65"></a>
</p>

## Por que usar

Better Resource Monitor é para quem só quer uma forma simples de acompanhar o Mac.

Ele mostra CPU, memória, armazenamento, GPU e rede direto na barra de menus, para você perceber cargas incomuns rapidamente sem abrir o Monitor de Atividade nem fuçar ferramentas do sistema.

Também foi feito para ser leve, para que o próprio monitor não vire parte do problema.

Se você está procurando uma alternativa ao iStat Menus, aqui está ela: monitoramento diário leve, com baixo consumo e sem telemetria.

## FAQ: alternativa ao iStat Menus

### O Better Resource Monitor pode substituir o iStat Menus?

Sim, quando o objetivo é visibilidade estável no dia a dia. iStat Menus é melhor para controle profundo do sistema; Better Resource Monitor é feito para manter apenas os números essenciais visíveis durante todo o dia com menos configuração.

### É gratuito?

Sim. Better Resource Monitor é gratuito e com licença MIT. A versão da Mac App Store e a do GitHub são o mesmo app.

### Ele coleta dados ou envia telemetria?

Não. Better Resource Monitor não faz requisições de rede. Não há analytics nem telemetria enviada.

### Funciona sem pesar a bateria?

Sim. Foi construído para ficar ativo em segundo plano com baixo impacto e útil no dia a dia.


## Instalação

Obtenha na <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> (inclui atualizações automáticas) ou baixe o `.dmg` no <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank">GitHub Releases</a> (sem atualizações automáticas; baixe e atualize manualmente cada versão).

### Compatibilidade

Funciona em Macs Intel e Apple Silicon com macOS Ventura 13 ou mais recente.

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

## Comparação

Atalhos rápidos:

- [Better Resource Monitor vs iStat Menus](https://better-resource-monitor.alexpedersen.dev/pt-br/comparison/vs-istat-menus/)
- [Better Resource Monitor vs Stats](https://better-resource-monitor.alexpedersen.dev/pt-br/comparison/vs-stats/)
- [Better Resource Monitor vs Eul](https://better-resource-monitor.alexpedersen.dev/pt-br/comparison/vs-eul/)

<table>
  <thead>
    <tr>
      <th width="20%">Recurso</th>
      <th width="20%">Better Resource Monitor</th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pt-br/comparison/vs-stats/">Stats</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pt-br/comparison/vs-eul/">Eul</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/pt-br/comparison/vs-istat-menus/">iStat Menus</a></th>
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

## Créditos


- <a href="https://github.com/phosphor-icons" target="_blank">Phosphor Icons</a> - Conjunto de ícones usado na bandeja
- <a href="https://alexpedersen.dev/" target="_blank">Alex Pedersen</a> - Mantenedor
