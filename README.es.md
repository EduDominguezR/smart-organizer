# Smart Organizer

[English](README.md) | Español

Smart Organizer mantiene ordenada una carpeta como Descargas o el Escritorio.
Antes de tocar nada enseña a dónde iría cada archivo, mueve solo lo que apruebas
y anota cada lote para poder deshacerlo.

Es una aplicación nativa de Windows escrita en Rust. Funciona sin conexión, no
tiene cuenta y no envía nada a ninguna parte.

![Inicio de Smart Organizer en español, con una carpeta Descargas, 24 archivos organizados y 9 listos](docs/screenshots/inicio.es.png)

## Capturas

Las capturas usan una carpeta ficticia, `C:\Users\Public\Smart Organizer Demo`.
No aparecen rutas ni documentos personales. El menú de la bandeja lo dibuja
Windows con el tema del sistema; la aplicación sí es oscura en todas las
capturas.

| Español | English |
| --- | --- |
| ![Inicio en español con 24 archivos organizados y 9 listos](docs/screenshots/inicio.es.png) | ![Inicio en inglés con 24 archivos organizados y 9 listos](docs/screenshots/inicio.en.png) |
| ![Vista previa antes de mover seis archivos, con las carpetas de destino](docs/screenshots/vista-previa.es.png) | ![Vista previa antes de mover seis archivos, con las carpetas de destino](docs/screenshots/vista-previa.en.png) |
| ![Historial de un lote organizado con el botón para guardar la lista](docs/screenshots/historial.es.png) | ![Historial de un lote organizado con el botón para guardar la lista](docs/screenshots/historial.en.png) |
| ![Ajustes con idioma, avisos y perfiles disponibles](docs/screenshots/ajustes.es.png) | ![Ajustes con idioma, avisos y perfiles disponibles](docs/screenshots/ajustes.en.png) |
| ![Reglas con el constructor de frases y reglas activas](docs/screenshots/reglas.es.png) | ![Reglas con el constructor de frases y reglas activas](docs/screenshots/reglas.en.png) |
| ![Grupos de duplicados con rutas, fechas, opción de conservar y Enviar a la papelera](docs/screenshots/duplicados.es.png) | ![Grupos de duplicados con rutas, fechas, opción de conservar y Enviar a la papelera](docs/screenshots/duplicados.en.png) |
| ![Menú de la bandeja con la vigilancia activa; Windows dibuja el menú con el tema del sistema](docs/screenshots/bandeja.es.png) | ![Menú de la bandeja con la vigilancia activa; Windows dibuja el menú con el tema del sistema](docs/screenshots/bandeja.en.png) |
| ![Guía de primer arranque para elegir carpeta, con atajos a Descargas, Escritorio y Documentos](docs/screenshots/primer-arranque.es.png) | ![Guía de primer arranque para elegir carpeta, con atajos a Descargas, Escritorio y Documentos](docs/screenshots/primer-arranque.en.png) |

### Estado vacío y apariencia

| Vista | Captura |
| --- | --- |
| Inicio vacío en español, con acento rojo y atajos a Descargas, Escritorio y Documentos | ![Inicio vacío en español, con acento rojo y atajos a Descargas, Escritorio y Documentos](docs/screenshots/inicio-vacio.es.png) |
| Ajustes > Apariencia en español, con acento rojo `#ff0000` y aviso de contraste AA | ![Ajustes > Apariencia en español, con acento rojo #ff0000 y aviso de contraste AA](docs/screenshots/ajustes-apariencia.es.png) |

## Funciones

- **Primero la vista previa.** La app prepara un plan con el destino de cada
  archivo. Puedes incluir o excluir filas; nada se mueve hasta pulsar
  **Organizar ahora**.
- **Deshacer.** Cada lote se anota en un diario. Inicio e Historial pueden
  deshacer el último lote, incluidos los archivos que una regla mandó a la
  papelera.
- **Ocho grupos iniciales.** Documentos, Programas, Comprimidos, Imágenes,
  Vídeos, Música, Proyectos y Otros. Cada grupo se puede renombrar, reordenar,
  cambiar de color, tener su propia carpeta de destino y editar sus tipos.
- **Constructor de reglas en forma de frase.** Las reglas se leen y se escriben
  así: `Cuando [el nombre contiene] [factura] → mover a [Documentos]`.
  El modo Avanzado añade más condiciones, varias condiciones por regla, reglas
  por carpeta, acciones de mover/copiar/ignorar/papelera y subcarpetas por tipo.
- **Perfiles.** Los perfiles General, Estudiante, Desarrollador y Fotografía
  añaden solo los grupos y reglas que faltan. Antes enseñan un resumen, nunca
  sobrescriben un grupo y se pueden deshacer.
- **Atajos y arrastrar y soltar.** Añade Descargas, Escritorio o Documentos con
  un clic, suelta una carpeta en cualquier parte de la ventana o suelta un
  archivo para añadir su carpeta contenedora. Los atajos respetan la redirección
  de OneDrive.
- **Buscador de duplicados.** Analiza las carpetas elegidas cuando se lo pides,
  agrupa por tamaño, compara el principio y el final de los candidatos y calcula
  un hash BLAKE3 completo solo para los candidatos reales. Tú eliges qué copia
  conservar; el resto puede ir a la papelera y se puede deshacer.
- **Vigilante, bandeja e inicio automático.** Un proceso aparte vigila los
  archivos nuevos. Espera el debounce, comprueba que el tamaño se estabilice y
  organiza solo lo que cambió. Al cerrar la ventana con la vigilancia activa,
  queda en la bandeja. El inicio automático arranca el vigilante oculto, no la
  ventana. Si una pasada falla, se captura y se anota en vez de terminar el
  vigilante. La interfaz reinicia un vigilante caído con esperas de 1 s, 2 s,
  4 s, 8 s y 16 s, como máximo cinco intentos en diez minutos; después muestra
  un aviso y deja de reintentar. **Reiniciar vigilancia** está en Ajustes >
  Vigilancia, Inicio, la vista sencilla y la bandeja.
- **Instancia única.** Un segundo arranque enfoca la ventana existente y sale.
  Un segundo vigilante sale en silencio. El bloqueo depende de la carpeta de
  datos, así que una copia instalada y una portable pueden convivir.
- **Interfaz bilingüe y accesible.** Español e inglés, tema oscuro y claro, color
  de acento, tres tamaños de texto y dos densidades. Todos los textos visibles
  pasan por `src/i18n.rs`; los controles tienen foco de teclado y estado visible.
  Los interruptores propios exponen el rol de interruptor y su estado, y las
  filas plegables de grupos exponen el rol de divulgación y si están desplegadas.
  El foco usa el acento elegido y los controles se alcanzan con Tab.
- **Importar, exportar, copia y reinicio.** Los ajustes se exportan e importan
  desde Ajustes. La carpeta de datos, el diario y el archivo de configuración son
  archivos locales normales. El reinicio es explícito y se puede recuperar con
  una copia de seguridad.

## Seguridad

- La acción normal es **mover**, no borrar. Lo que no encaja en ningún grupo se
  queda donde está.
- La papelera se usa solo si eliges una regla con la acción Papelera o si envías
  duplicados elegidos a la papelera. No hay un botón de borrado definitivo.
- Antes de mandar duplicados a la papelera, la app comprueba que la copia que
  conservas sigue existiendo y que su hash completo coincide con cada copia.
- Se omiten archivos ocultos, restos de descargas (`.part`, `.crdownload`,
  `.tmp`), enlaces simbólicos y carpetas protegidas del sistema. El escáner no
  sale de la carpeta que añadiste.
- El análisis de duplicados ignora los marcadores de OneDrive no descargados y
  los archivos de 0 bytes, para no forzar descargas ni crear grupos sin sentido.
- El diario se escribe antes de mover los archivos. Al arrancar, cada entrada
  pendiente se comprueba contra el disco. Si falta el origen y existe el destino,
  el movimiento se marca como completado. Si falta el destino, se marca como
  interrumpido. Si existen los dos, la app compara tamaño y BLAKE3, marca la
  entrada como **necesita revisión**, no borra nada y avisa en Historial.
- Una pasada fallida del vigilante se anota en `error.log`; el diario se repara
  antes de la siguiente pasada. Un pánico al arrancar la interfaz o `--watch`
  también queda allí con su traza.
- Los ajustes se escriben en `config.toml.tmp`, se sincronizan y sustituyen
  `config.toml` de forma atómica. Antes se copia el archivo actual a
  `config.toml.bak`. Antes de una migración de versión se crea una vez
  `config.toml.v{version}.bak`, que nunca se sobrescribe.
- La app no hace peticiones de red y no tiene telemetría.

## Empezar en tres pasos

1. **Añade una carpeta.** Pulsa **Añadir carpeta**, elige Descargas o Escritorio,
   o suelta una carpeta en la ventana. Los tres atajos usan la carpeta real de
   Windows, incluida la redirección de OneDrive.
2. **Revisa la vista previa.** Abre **Vista previa**, mira el origen, el destino y
   el tamaño de cada archivo y desmarca lo que quieras dejar quieto.
3. **Organiza y comprueba.** Pulsa **Organizar ahora**. El lote aparece en
   **Historial**; si algo no está bien, pulsa **Deshacer** antes de seguir.

La guía de primer arranque ofrece el mismo camino y se puede saltar.

## Descarga e instalación

La versión actual está en [`dist/`](dist/):

- `Smart Organizer.exe`: interfaz gráfica (y modo vigilante oculto).
- `smart-organizer-cli.exe`: versión de consola para automatizar.
- `SHA256SUMS.txt`: SHA-256 de cada archivo de la versión.
- `README.txt` / `LEEME.txt`: instrucciones breves en inglés y español.

No hay instalador ni hace falta ser administrador. Copia el ejecutable a una
carpeta tuya y ábrelo. Para guardar los ajustes junto al ejecutable, crea una
carpeta llamada `portable` a su lado antes de arrancar; entonces `config.toml` y
`journal.jsonl` se guardan ahí. Consulta
[Datos, modo portable, copia y reinicio](#datos-modo-portable-copia-y-reinicio).

El ejecutable todavía no está firmado. Windows puede mostrar un aviso de
SmartScreen; comprueba el hash antes de elegir **Más información > Ejecutar de
todos modos**. Un SHA-256 coincidente demuestra que el archivo es el mismo cuyo
hash publicó el autor y que no se corrompió al descargarlo; **no** demuestra que
el programa sea seguro ni que el autor sea de confianza.

**TODO:** añadir la URL pública de descarga cuando se publique el proyecto.

### SHA-256

```text
77CF5BB77DBAC252056A7E793EF7C4E366025C688D6068497A9CF1EA1D4B767E  Smart Organizer.exe
8E31EEE27371FE261D5A1F54CB5DC8E3BD4BF82FE5A9E1CF8F4DCC9F90BCC1FD  smart-organizer-cli.exe
```

`dist/SHA256SUMS.txt` es la lista autorizada de la versión actual. Compruébala
en PowerShell con:

```powershell
Get-FileHash ".\Smart Organizer.exe" -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

### SmartScreen

La versión no tiene firma Authenticode ni reputación de editor todavía, así que
SmartScreen puede mostrar «Windows protegió tu PC». Es lo normal en un
ejecutable nuevo sin firmar. Comprueba primero el SHA-256 y usa **Más
información** y **Ejecutar de todos modos** solo si confías en el archivo y el
hash coincide. No desactives SmartScreen de forma global.

El script de compilación permite firmar cuando hay un certificado; consulta
[Compilar, firmar y probar](#compilar-firmar-y-probar). La firma elimina el aviso
de editor desconocido, pero un certificado nuevo todavía tiene que ganar
reputación.

## Reglas con ejemplos

Las reglas se aplican antes que los grupos por tipo de archivo. En el
constructor sencillo, la frase es la regla:

- `Cuando [el nombre contiene] [factura] → mover a [Documentos]`
- `Cuando [el tipo es] [imagen] → mover a [Imagenes]`
- `Cuando [el tamaño es mayor que] [100 MB] → mover a [Otros]`
- `Cuando [tiene más de] [30] días → mover a [Otros]`

En el modo Avanzado puedes combinar condiciones, limitar una regla a una carpeta
añadida, elegir **mover**, **copiar**, **ignorar** o **papelera**, y añadir una
subcarpeta por tipo. Las reglas se evalúan en orden; gana la primera que coincide.
Una regla sin destino válido se rechaza antes de mover nada.

La limpieza de duplicados es aparte: abre **Duplicados**, pulsa **Buscar
duplicados**, revisa cada grupo, elige la copia que conservas y pulsa **Enviar a
la papelera**.

## Vigilante, bandeja, inicio automático e instancia única

- **Vigilancia.** Arranca un proceso aparte con `--watch`. Escucha eventos del
  sistema de archivos en vez de recorrer la carpeta cada segundo, espera el
  debounce y la estabilidad de tamaño, y solo vuelve a mirar las rutas que
  cambiaron.
- **Bandeja.** Con la vigilancia activa, cerrar la ventana la oculta. El menú de
  la bandeja permite abrir, pausar o reanudar la vigilancia, organizar ahora o
  salir. Salir detiene el vigilante.
- **Inicio automático.** Ajustes > Vigilancia > **Iniciar con Windows** escribe
  una entrada en `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` para el
  ejecutable actual con `--watch --hidden`. No abre la ventana al entrar en
  Windows. Al desactivar el interruptor se borra la entrada.
- **Instancia única.** La app usa un mutex con nombre derivado de la carpeta de
  datos. El segundo arranque de la interfaz restaura y enfoca la primera ventana
  y sale con código 0. Un segundo vigilante sale en silencio. Una copia portable
  y una instalada tienen carpetas distintas y pueden convivir.
- **Fallo y reinicio.** La interfaz distingue si arrancó ella el vigilante y si
  la parada fue intencionada. Una salida inesperada se reintenta tras 1, 2, 4, 8
  y 16 segundos, con un máximo de cinco intentos en diez minutos. Tras el quinto
  fallo muestra un aviso persistente y no vuelve a intentarlo. Reiniciar desde la
  bandeja, Ajustes, Inicio o la barra de estado reinicia esa cuenta. Un vigilante
  arrancado por el inicio de Windows se detiene con `watch.stop` y el nuevo
  proceso espera a que se libere su mutex antes de arrancar.
- **Prioridad de fondo.** Solo el proceso `--watch` pide a Windows EcoQoS
  (`ProcessPowerThrottling`, velocidad de ejecución). Si el sistema no lo admite,
  la llamada falla sin afectar al funcionamiento. La interfaz nunca lo aplica.
- **Después de actualizar.** Sustituye el ejecutable con la app cerrada. Los
  ajustes y el diario viven fuera del ejecutable y se conservan. Si el vigilante
  está activo, sal primero desde la bandeja.

## Recursos medidos

Las cifras siguientes son mediciones históricas de la versión anterior de
6.545.920 bytes. Se conservan para comparar y **no** son mediciones de la versión
actual, que usa `panic = "unwind"` e incluye la recuperación y los reinicios
nuevos. Las mediciones nuevas están pendientes.

Medido en Windows 11, 1920x1080 al 125 %, en `DESKTOP-9U3MDFQ`. La CPU es el
porcentaje medio de un núcleo lógico; la memoria es el conjunto de trabajo. Cada
estado se muestreó una vez por segundo durante 60 segundos con
[`scripts/measure.ps1`](scripts/measure.ps1), tras un calentamiento para dejar
terminar el arranque y el primer repintado.

| Estado | CPU antes | Memoria antes | CPU final anterior | Memoria final anterior |
| --- | ---: | ---: | ---: | ---: |
| GUI visible, en reposo | 0,12 % | 96,2 MB | 0,00 % | 97,0 MB |
| GUI minimizada | 0,07 % | 96,3 MB | 0,00 % | 97,0 MB |
| GUI en bandeja + vigilante | 0,52 % | 106,5 MB | 0,02 % | 108,8 MB |
| Vigilante solo (`--watch`) | 0,35 % | 10,4 MB | 0,00 % | 10,6 MB |

Mediciones pendientes de la versión actual (rellena cada celda con la muestra
real; no copies las cifras históricas):

| Estado | CPU % | Memoria MB | Equipo | Versión de Windows | Fecha |
| --- | ---: | ---: | --- | --- | --- |
| GUI visible, en reposo | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir |
| GUI minimizada | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir |
| GUI en bandeja + vigilante | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir |
| Vigilante solo (`--watch`) | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir | TODO: Medir |

La versión final anterior promedió 0,00 % de CPU con la ventana visible,
minimizada y con el vigilante solo, y 0,02 % con la ventana oculta en la bandeja.
Una muestra suelta de 1 segundo todavía puede marcar 1 %, que el Administrador de
tareas redondea a 0 %. El ejecutable era un 19,9 % más pequeño que la compilación
con `opt-level = 3`.

Las muestras, el entorno y el método completo están en
[`docs/rendimiento.md`](docs/rendimiento.md). La tabla vacía de la versión actual
también está en [`docs/checklist-pruebas.md`](docs/checklist-pruebas.md).

## Datos, modo portable, copia y reinicio

Carpeta de datos normal en Windows:

```text
%APPDATA%\smart-organizer\
  config.toml
  config.toml.bak                    (ajustes anteriores, copiados antes de guardar)
  config.toml.v{version}.bak         (copia permanente previa a migrar, creada una vez)
  config.toml.tmp                    (solo durante un guardado; se borra o se reemplaza)
  journal.jsonl
  error.log                          (fallos y trazas de pánico; rota a 1 MB)
  error.log.1                        (registro anterior tras la rotación)
  watch.stop                         (solo mientras se detiene un vigilante)
```

Ábrela desde **Ajustes > Datos > Abrir carpeta de datos**. El diario es JSON
Lines, una operación por línea; de ahí sale la opción Deshacer. Al guardar, la app
copia el archivo actual a `config.toml.bak`, escribe y sincroniza
`config.toml.tmp` y sustituye `config.toml` en una sola operación. Si el
reemplazo falla, el `config.toml` anterior sigue siendo válido. La copia
`config.toml.v{version}.bak` de antes de migrar nunca se sobrescribe.

**Modo portable.** Crea una carpeta llamada `portable` junto a
`Smart Organizer.exe`. En el siguiente arranque, `config.toml` y `journal.jsonl`
se crean al lado del ejecutable. Borra la carpeta `portable` para volver a la
carpeta de datos normal. Al mover una copia portable, lleva la carpeta completa.

**Copia de seguridad.** Cierra la app (también el vigilante) y copia
`config.toml` y `journal.jsonl`. Si restauras los dos, conservas el historial y
puedes seguir deshaciendo. Los archivos `.bak` son copias automáticas y no
sustituyen a una copia de seguridad. Exportar ajustes escribe una copia legible
de la configuración, pero no incluye el diario.

**Reinicio.** Ajustes > Datos > Restablecer ajustes reinicia la configuración
pero deja el diario. Para borrar todo el estado, cierra la app y elimina la
carpeta de datos. Eso quita el historial y la posibilidad de deshacer lotes
antiguos para siempre; nunca toca tus documentos. Haz una copia antes si puedes
necesitar Deshacer.

## Preguntas frecuentes

**¿Borra archivos?** No. Mueve archivos o los manda a la papelera cuando lo
eliges explícitamente. Puedes restaurarlos desde la papelera o con Deshacer en
Historial.

**¿Por qué se quedó un archivo donde estaba?** Puede ser un archivo oculto, un
resto de descarga, un enlace simbólico, estar dentro de una carpeta omitida, ya
estar en su destino o no coincidir con ningún grupo. La Vista previa indica el
motivo.

**¿Organiza también las subcarpetas?** Sí, hasta ocho niveles por debajo de cada
carpeta añadida, omitiendo carpetas ocultas y las conocidas del sistema o de
categorías. Una subcarpeta sin permisos se salta sin detener el resto.

**¿El buscador de duplicados lee todo el disco?** No. Mira solo las carpetas que
añadiste e ignora carpetas del sistema o de destino, enlaces simbólicos, archivos
vacíos y marcadores de OneDrive no descargados. Lee los candidatos en local y no
sube nada.

**¿Se puede deshacer una limpieza de duplicados?** Sí. Las operaciones de
papelera se anotan como lotes; usa Deshacer en Historial. Además, los archivos
están en la papelera de Windows y se pueden restaurar.

**¿Funciona con OneDrive?** Los atajos usan las carpetas conocidas reales de
Windows, así que encuentran Descargas, Escritorio y Documentos redirigidos. El
análisis de duplicados omite los archivos que no están descargados en local en
vez de forzar la descarga.

**¿Qué pasa si arranco dos copias?** La segunda ventana enfoca la primera y sale.
El vigilante tiene su propio bloqueo y sale en silencio. Una copia portable usa
otra carpeta de datos, así que no bloquea a la instalada.

**¿Envía datos a internet?** No. No hay telemetría, cuenta, actualizador ni
cliente de red en la app.

**¿Y si no se abre la ventana?** En Escritorio Remoto, una máquina virtual o un
equipo sin aceleración gráfica, la interfaz puede no arrancar. La app escribe el
motivo en `error.log`, muestra un mensaje localizado y sale sin tocar archivos.
Si ya hay carpetas configuradas, ejecuta `Smart Organizer.exe --watch` para
seguir vigilando. El mensaje en español es: «No pudimos iniciar la interfaz
gráfica. Es probable que estés en Escritorio Remoto, una máquina virtual o que
falte aceleración gráfica. Si ya configuraste carpetas, la vigilancia puede seguir
funcionando con el modo --watch.»

## Pruebas manuales recomendadas

1. Escritorio/Documentos de OneDrive y archivos solo en la nube.
2. 5.000 archivos a la vez.
3. Escalado de pantalla al 125 % y al 150 %.
4. Matar el proceso en mitad de un movimiento e inspeccionar Historial.
5. Abrir la app dos veces mientras vigila.
6. Soltar una carpeta, una carpeta del sistema y un archivo suelto.
7. Duplicados con una copia, tres copias y un archivo solo en la nube.
8. Escritorio Remoto o una máquina virtual.
9. Windows 10 y Windows 11 recién instalados.
10. Navegación solo con teclado y con Narrador.

## Compilar, firmar y probar

Requisitos: Rust 1.85 o superior, el toolchain GNU o MSVC de Windows y el SDK de
Windows solo si quieres firmar. Las dependencias están fijadas en `Cargo.lock`.

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
.\build-release.ps1
```

`build-release.ps1` compila la interfaz y la CLI, quita las rutas absolutas de
compilación, copia ambos ejecutables a `dist/` y escribe `SHA256SUMS.txt`. No
toca tus documentos.

La firma opcional usa `signtool.exe` del SDK de Windows, disponible en PATH o en
`SIGNTOOL_PATH`:

```powershell
$env:SIGN_CERT_PATH = "C:\keys\smart-organizer.pfx"
$env:SIGN_CERT_PASSWORD = "..."
$env:SIGN_TIMESTAMP_URL = "http://timestamp.digicert.com"  # opcional
.\build-release.ps1
```

También se puede elegir un certificado ya instalado en el almacén de Windows con
`SIGN_CERT_THUMBPRINT` en lugar de `SIGN_CERT_PATH`. Si no encuentra un
certificado o `signtool.exe`, el script avisa y genera igualmente una versión sin
firmar con sus hashes. Firmar no cambia la privacidad ni el comportamiento
descrito.

Las pruebas cubren agrupación, selección y verificación de duplicados, filtrado
de eventos del vigilante y límites del debounce, recuperación de entradas
pendientes del diario y copia y guardado atómico de la configuración. La suite
actual tiene 57 pruebas. La versión medida histórica usó `opt-level = "s"` y LTO;
el perfil está en `Cargo.toml`.

La versión actual sin firmar ocupa 7.063.040 bytes para la GUI y 1.846.272 bytes
para el CLI. Cambiar `panic = "abort"` por `panic = "unwind"` y añadir la
recuperación y los reinicios aumentó el ejecutable de la GUI en 517.120 bytes
(7,9 %) frente a la versión anterior de 6.545.920 bytes.

Ejemplos de CLI:

```powershell
.\smart-organizer-cli.exe --organize "C:\Users\me\Downloads" --dry-run
.\smart-organizer-cli.exe --organize "C:\Users\me\Downloads"
.\smart-organizer-cli.exe --undo-last
.\smart-organizer-cli.exe --watch
```

## Privacidad

Smart Organizer lee solo las carpetas que añades. Las lecturas de hash y los
metadatos se quedan en el equipo. No hay telemetría, analítica, anuncios,
cuentas, servicio en la nube ni conexión de red. Los únicos datos que escribe son
`config.toml`, sus copias `.bak` y temporales, `journal.jsonl`, `error.log` y
`watch.stop` (temporal) en la carpeta de datos. Los temporales se reconocen por el
sufijo `.tmp` y se borran o se reemplazan. Las comprobaciones de SmartScreen las
hace Windows, no esta app.

## Licencia y estado del proyecto

Los metadatos del paquete declaran **MIT**. Todavía no hay un archivo `LICENSE`
ni un titular de derechos en esta copia.

**TODO:** añadir un archivo `LICENSE` y el titular de los derechos antes de
publicar.

**TODO:** añadir la URL del repositorio público.

**TODO:** añadir un contacto de soporte o un gestor de incidencias.

Hasta entonces, el ejecutable y la documentación se ofrecen tal cual, sin
compromiso de soporte. La app funciona sin conexión y no depende de un
repositorio, servidor de licencias ni servicio de soporte.
