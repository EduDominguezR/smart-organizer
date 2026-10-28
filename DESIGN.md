# Dirección visual

Sale de las dos capturas que entregó el dueño del producto. Este archivo solo
transcribe esa dirección: no inventa identidad ni agrega elementos nuevos.

```
Dial: ENERGY 1 / RHYTHM 1 / MOTION 1
```

## Identidad

Herramienta de escritorio para alguien que descarga archivos al escritorio y
quiere que se ordenen sin miedo a perder nada. No es un panel de administración:
es una ventana que dice la verdad sobre lo que va a pasar y deja deshacer.

Personalidad: sobria, directa, sin adornos. La app no celebra ni anima; informa y
confirma. Cuando algo falla, explica qué pasó, qué se hizo para proteger los
archivos y qué puede hacer la persona.

## Por qué los diales en 1

La tarea principal es escanear listas y confirmar movimientos. La uniformidad
entre pantallas hace que la segunda visita no requiera reaprender dónde está cada
cosa. ENERGY 1 y RHYTHM 1 son una decisión, no un descuido: en una utilidad que se
abre por segundos, mover la jerarquía de sitio cuesta más de lo que aporta.

MOTION 1 significa estados de hover y foco, nada más. Sin pulsos, sin bucles, sin
animaciones de entrada. El repintado ocurre cuando llega un evento o cuando la
persona interactúa.

## Paleta

Los ratios están calculados con la fórmula WCAG 2.x, no estimados a ojo. Todos los
pares de texto pasan AA (4,5:1) y los bordes de control pasan 3:1. El color de
acento se puede cambiar; el texto del acento se calcula contra la superficie más
clara del tema para no bajar de 4,5:1.

| Rol | Oscuro | Claro | Ratio |
| --- | --- | --- | --- |
| Fondo de app | `#171717` | `#f7f7f8` | |
| Superficie (tarjeta, tabla) | `#1f1f1f` | `#ffffff` | |
| Superficie alternativa | `#262626` | `#efeff1` | |
| Texto principal | `#e8e8e8` | `#171717` | 14,63 / 16,75 |
| Texto secundario | `#a3a3a3` | `#525252` | 6,53 / 7,81 |
| Acento, relleno | `#2563eb` con texto blanco | igual | 5,17 |
| Acento, texto y enlaces | `#60a5fa` | `#1d4ed8` | 5,95 / 5,84 |
| Error, texto | `#f87171` | `#b91c1c` | 5,96 / 6,47 |
| Éxito, texto | `#22c55e` | `#15803d` | 7,23 / 5,02 |
| Borde de control | `#8a8a8a` | `#737373` | 4,38 / 4,13 |
| Anillo de foco | `#93c5fd` sobre azul | igual | 9,94 |

Dos ajustes sobre las capturas, obligados por contraste: el azul `#2563eb` sirve
como relleno pero no como texto, y el rojo de texto sube a `#f87171` porque
`#dc2626` no llega a AA.

Uso del acento: botón primario, pestaña activa y barra de progreso. Nada más. Si
el acento aparece en todos los bordes, deja de señalar el siguiente paso. Los
colores de categoría son solo para el gráfico y la leyenda; no cambian el color
de los botones.

## Tipografía

Tipografía del sistema. En Windows se carga Segoe UI desde `C:\Windows\Fonts`; si
no está, egui usa su fuente incluida. En macOS y Linux se usa la que resuelva el
entorno. Razón: la app tiene que verse nativa en la máquina donde vive, y una
familia empaquetada suma peso al binario sin mejorar la lectura de tablas densas.

Tamaño base 13 px, títulos de sección 22 px, encabezados de tarjeta 18 px,
etiquetas de tabla 12 px. Las micro-etiquetas de sección van en mayúsculas a
11 px, sin tracking amplio. Sin monoespaciada decorativa; la monoespaciada solo
aparece en rutas y datos técnicos.

## Forma y espacio

Radio de 4 px en tarjetas, campos, píldoras y botones; 6 px solo en el botón
primario. Un solo radio en todo produciría el efecto pastilla que se ve en
cualquier plantilla. Bordes de 1 px, sin sombras: la jerarquía la hace el color de
superficie, no la elevación.

Escala de espacio 4, 8, 12, 16, 24, 32. Las filas de tabla miden 32 px en densidad
compacta y 42 px en cómoda. La ventana abre a 1100 x 600 y no baja de 860 x 520.

## Estados

- Hover: cambia el relleno de la superficie, no el tamaño.
- Foco: anillo visible de 2 px, también con teclado.
- Deshabilitado: texto secundario y sin relleno de acento. Nunca se oculta una
  acción que existe; se explica por qué no está disponible.
- Error: texto rojo, borde de 1 px y una frase con qué pasó y qué hacer. No se
  usan iconos de alerta ni animaciones.
- Éxito: se anuncia en la barra de estado, sin ventanas emergentes.

## Iconos

Glifos dibujados con `egui::Painter` en trazo de 1,5 px. Sin fuente de emoji y sin
librería externa. Solo aparecen donde reemplazan una palabra repetida (papelera,
carpeta, deshacer); el resto del texto va sin icono.

## Decisiones tomadas con el dueño

- Carpetas destino con tilde: `Imagenes` pasa a `Imágenes`, `Musica` a `Música`.
  Las variantes sin tilde se aceptan como alias al leer configuración vieja.
- La flecha de la barra inferior se quita: `Ver detalle` ya lleva al mismo sitio.
- El tema oscuro es el de por defecto porque la app se abre muchas veces al día
  junto a otras ventanas oscuras, y no porque lo oscuro se vea técnico. El tema
  claro existe y se verifica con la misma tabla de contraste.
- El selector de idioma cambia toda la interfaz, incluidos avisos y errores, y se
  guarda en `config.toml`.
- El tamaño de texto y la densidad son globales; no se mezclan tamaños distintos
  en la misma pantalla.
- El botón primario solo aparece cuando hay algo que confirmar. Si la vista previa
  está vacía, se muestra deshabilitado y con el motivo en la barra de estado.
