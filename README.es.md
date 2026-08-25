<p align="center">
  <img src="https://better-resource-monitor.alexpedersen.dev/better-resource-monitor.png" alt="Better Resource Monitor" width="830" height="86">
</p>


<h1 align="center">Better Resource Monitor</h1>

<p align="center">
  <strong>Supervisa CPU, memoria, almacenamiento, GPU y red desde la barra de menús de tu Mac.</strong>
</p>

<!-- README-LANG-START -->

<p align="center">
  <a href="README.md">English</a> •
  Español •
  <a href="README.pt-br.md">Português (Brasil)</a> •
  <a href="README.zh-cn.md">简体中文</a>
</p>

<!-- README-LANG-END -->


<p align="center">
  <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/appstore-es.webp" alt="Descargar en la Mac App Store" width="270" height="65"></a>
  <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank"><img src="https://better-resource-monitor.alexpedersen.dev/badges/macos-es.webp" alt="Descargar macOS en GitHub Releases" width="270" height="65"></a>
</p>

## Por qué

Better Resource Monitor es para quienes solo quieren una forma rápida y discreta de vigilar su Mac.

Mantiene el uso de CPU, memoria, almacenamiento, GPU y red directamente en la barra de menús, para que puedas detectar cargas inusuales sin interrumpir lo que haces ni abrir el Monitor de Actividad.

Está pensado para quedarse abierto todo el día sin convertirse en parte del problema: menos del 0,1% de CPU en Apple Silicon, unos 15 MB de memoria y cero solicitudes de red.

Si estás evaluando una alternativa a iStat Menus, aquí la tienes: monitoreo diario ligero, con bajo consumo y sin telemetría.

## Para quién es

Elige Better Resource Monitor si quieres un monitor ligero para la barra de menús de Mac y una vista rápida diaria del uso de CPU, memoria, almacenamiento, GPU y red. Es gratuito, de código abierto, funciona en sandbox y sin conexión, y no necesita contraseña de administrador ni un helper con permisos root. Funciona en Macs Intel y Apple Silicon con macOS 13 o posterior.

Elige Stats o iStat Menus si necesitas ventiladores, temperaturas, batería, historial detallado de sensores o más controles de hardware. Better Resource Monitor se mantiene enfocado en las métricas esenciales que se consultan a diario.

## FAQ: alternativa a iStat Menus

### ¿Puede reemplazar a iStat Menus?

Sí, cuando buscas visibilidad y estabilidad diaria. iStat Menus destaca en control profundo del sistema; Better Resource Monitor está pensado para mantener métricas clave simples y visibles todo el día con menos configuración.

### ¿Es gratis?

Sí. Better Resource Monitor es gratuito y de licencia MIT. La versión de la Mac App Store y la de GitHub son la misma aplicación.

### ¿Recopila datos o envía telemetría?

No. Better Resource Monitor no hace solicitudes de red. No hay analíticas ni telemetría enviadas.

### ¿Funciona sin consumir batería?

Sí. Está diseñado para permanecer en segundo plano con bajo impacto y resultar útil durante el trabajo diario.

## Alertas

Con **Mostrar colores de advertencia** activado, la barra se vuelve naranja cuando el uso visible de CPU, memoria, almacenamiento o GPU llega al **81%**. Recupera su aspecto normal cuando todos bajan del 81%.

Las alertas son solo visuales. La velocidad de red y las métricas ocultas no las activan. El umbral es fijo, pero los colores se pueden desactivar desde el menú de la barra.


## Instalación

Descárgalo desde la <a href="https://apps.apple.com/app/better-resource-monitor/id6758237306" target="_blank">Mac App Store</a> (incluye actualizaciones automáticas) o descarga el `.dmg` desde <a href="https://github.com/alexx855/better-resource-monitor/releases" target="_blank">GitHub Releases</a> (sin actualizaciones automáticas; descarga e instala manualmente cada nueva versión).

### Compatibilidad

Funciona en Mac con Intel y Apple Silicon con macOS Ventura 13 o posterior.

### Compilar desde el código fuente

Necesitarás los <a href="https://v2.tauri.app/start/prerequisites/" target="_blank">requisitos previos de Tauri v2</a> y <a href="https://pnpm.io/" target="_blank">pnpm</a>.

```bash
git clone https://github.com/alexx855/better-resource-monitor.git
cd better-resource-monitor
pnpm install
pnpm tauri build
```

### Desarrollo

```bash
# Ejecutar en modo desarrollo con recarga en caliente
pnpm tauri dev

# Ejecutar pruebas
cd src-tauri && cargo test

# Ejecutar pruebas con cobertura (requiere cargo-llvm-cov)
cargo install cargo-llvm-cov
cd src-tauri && cargo llvm-cov --lib --html --output-dir coverage/
```

## Comparación

Atajos rápidos:

- [Better Resource Monitor vs iStat Menus](https://better-resource-monitor.alexpedersen.dev/es/comparison/vs-istat-menus/)
- [Better Resource Monitor vs Stats](https://better-resource-monitor.alexpedersen.dev/es/comparison/vs-stats/)
- [Better Resource Monitor vs Eul](https://better-resource-monitor.alexpedersen.dev/es/comparison/vs-eul/)

<table>
  <thead>
    <tr>
      <th width="20%">Característica</th>
      <th width="20%">Better Resource Monitor</th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/es/comparison/vs-stats/">Stats</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/es/comparison/vs-eul/">Eul</a></th>
      <th width="20%"><a href="https://better-resource-monitor.alexpedersen.dev/es/comparison/vs-istat-menus/">iStat Menus</a></th>
    </tr>
  </thead>
  <tbody>
    <tr><th scope="row">Mac App Store</th><td align="center">Sí (funciones completas)</td><td align="center">No</td><td align="center">Limitado</td><td align="center">Limitado</td></tr>
    <tr><th scope="row">Contraseña de administrador / privilegios</th><td align="center">Ninguno (sandbox)</td><td align="center">Requiere helper con permisos root</td><td align="center">Ninguno</td><td align="center">Requiere helper con permisos root</td></tr>
    <tr><th scope="row">Estabilidad de API de GPU</th><td align="center">API pública</td><td align="center">API privada</td><td align="center">API privada</td><td align="center">Propietario</td></tr>
    <tr><th scope="row">Uso de memoria</th><td align="center">~15 MB</td><td align="center">~50 MB</td><td align="center">~40 MB</td><td align="center">~100+ MB</td></tr>
    <tr><th scope="row">Impacto en CPU / energía</th><td align="center">&lt; 0,1%</td><td align="center">~1%</td><td align="center">Alto (serie M)</td><td align="center">~1%</td></tr>
    <tr><th scope="row">Tamaño de la app</th><td align="center">&lt; 7 MB</td><td align="center">~25 MB</td><td align="center">~5 MB</td><td align="center">~65 MB</td></tr>
    <tr><th scope="row">Privacidad/telemetría</th><td align="center">100% sin conexión</td><td align="center">Sin conexión</td><td align="center">Sin conexión</td><td align="center">Analíticas</td></tr>
    <tr><th scope="row">Estado</th><td align="center">Activo</td><td align="center">Activo</td><td align="center">Sin mantenimiento</td><td align="center">Activo</td></tr>
    <tr><th scope="row">Lenguaje</th><td align="center">Rust</td><td align="center">Swift / C++</td><td align="center">Swift</td><td align="center">Obj-C / Swift</td></tr>
    <tr><th scope="row">Precio</th><td align="center">Gratis</td><td align="center">Gratis</td><td align="center">Gratis</td><td align="center">$14.99</td></tr>
    <tr><th scope="row">Licencia</th><td align="center">MIT</td><td align="center">MIT</td><td align="center">MIT</td><td align="center">Propietario</td></tr>
  </tbody>
</table>

> Los números de terceros son estimaciones aproximadas. Tu experiencia puede variar.

## Créditos


- <a href="https://github.com/phosphor-icons" target="_blank">Phosphor Icons</a> - Conjunto de iconos usado en la bandeja
- <a href="https://alexpedersen.dev/" target="_blank">Alex Pedersen</a> - Responsable del mantenimiento
