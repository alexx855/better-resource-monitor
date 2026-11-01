---
title: "Política de privacidad | Better Resource Monitor"
description: "Lee cómo Better Resource Monitor gestiona la privacidad: sin recopilación de datos personales, analíticas ni telemetría."
ogImage: "/images/og-privacy-es.png"
---

# Política de privacidad

**Última actualización: 30 de octubre de 2026**

## Descripción general

Better Resource Monitor ("la App") es una app de escritorio que muestra información de recursos del sistema en la barra de menús. Esta política de privacidad explica cómo gestionamos tus datos.

## Recopilación de datos

**No recopilamos ningún dato personal.**

La App funciona completamente de forma local en tu dispositivo. Toda la información del sistema (uso de CPU, uso de memoria, uso de almacenamiento, actividad de red, estadísticas de GPU) es:
- Leída directamente desde tu sistema operativo
- Mostrada solo en tu pantalla local
- Nunca transmitida a servidores externos
- Nunca almacenada en ninguna base de datos externa

El uso de almacenamiento se calcula con la capacidad total y libre que informa el sistema operativo para el volumen local seleccionado; por defecto, el que contiene tu carpeta de usuario. Para ofrecer opciones de disco, la App lee los nombres, puntos de montaje y capacidad de los volúmenes locales montados compatibles, incluidos los discos externos. El punto de montaje elegido se guarda localmente como preferencia. La App no lee nombres ni contenidos de archivos.

## Almacenamiento de datos

La App almacena tus preferencias (como qué métricas mostrar y ajustes de apariencia) localmente en tu dispositivo, mediante el almacenamiento de configuración estándar de tu sistema operativo.

En macOS, la App también añade entradas de diagnóstico con marcas de tiempo a archivos locales `autostart.log` en directorios de registros de Library, incluidas ubicaciones dentro del contenedor de la App, para ayudar a resolver problemas de inicio, apertura al iniciar sesión y funcionamiento de la barra de menús. Estos registros pueden incluir la versión de la App y el identificador del commit de compilación, el estado de registro del ítem de inicio de sesión, los ajustes seleccionados, eventos de inicio y del icono de la barra de menús, mensajes de error y rutas de archivos locales (como las del ejecutable, el recibo de App Store y el bloqueo de procesos). Las preferencias y los registros de diagnóstico permanecen en tu dispositivo; la App no los transmite.

## Servicios de terceros

La App no:
- Utiliza servicios de analíticas
- Incluye publicidad
- Se conecta a APIs externas
- Comparte datos con terceros

## Acceso a la red

La App supervisa tu actividad de red para mostrar velocidades de subida y descarga. No:
- Realiza ninguna solicitud de red saliente
- Envía ningún dato a través de internet
- Accede al contenido de tu tráfico de red

## Código abierto

Better Resource Monitor es software de código abierto. Puedes revisar el código fuente completo en [GitHub](https://github.com/alexx855/better-resource-monitor) para comprobar estas prácticas de privacidad.

## Cambios en esta política

Si realizamos cambios en esta política de privacidad, actualizaremos la fecha de "Última actualización" anterior.

## Contacto

Si tienes preguntas sobre esta política de privacidad, abre una incidencia en nuestro [repositorio de GitHub](https://github.com/alexx855/better-resource-monitor/issues).
