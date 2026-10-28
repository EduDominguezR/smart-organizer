# Rendimiento medido

Windows 11, `DESKTOP-9U3MDFQ`, 1920x1080 al 125 %. Una muestra por segundo
durante 60 s por estado. CPU en porcentaje de un núcleo lógico; memoria en MB de
conjunto de trabajo. Los números son de la versión final anterior sin firmar
(`opt-level = "s"`, LTO, `codegen-units = 1`, `strip`, `panic = "abort"`). No es
una prueba de batería ni una medida de GPU.

La versión actual cambia a `panic = "unwind"`, añade recuperación del diario y
reinicio supervisado, y ocupa 7.063.040 bytes. **No hay mediciones nuevas**: las
cifras de abajo son históricas y no deben presentarse como las de la versión
actual. La tabla vacía está en [`checklist-pruebas.md`](checklist-pruebas.md).

## Antes y después

El estado «antes» es la versión de referencia de 7.941.632 bytes. El estado
«final anterior» es `dist\Smart Organizer.exe`, de 6.545.920 bytes.

| Estado | CPU antes | Mem. antes | CPU final anterior | Mem. final anterior |
| --- | ---: | ---: | ---: | ---: |
| GUI visible, en reposo | 0,12 % | 96,2 MB | 0,00 % | 97,0 MB |
| GUI minimizada | 0,07 % | 96,3 MB | 0,00 % | 97,0 MB |
| GUI en bandeja + vigilante | 0,52 % | 106,5 MB | 0,02 % | 108,8 MB |
| Vigilante solo (`--watch`) | 0,35 % | 10,4 MB | 0,00 % | 10,6 MB |

Los máximos de la versión final anterior fueron 0 % en la GUI visible y en el
vigilante solo, y 1 % en la bandeja con el vigilante. El detalle por segundo está en
`rendimiento-antes-muestras.csv` y `rendimiento-final-muestras.csv`.

La GUI visible y minimizada quedan por debajo del 1 % que muestra el Administrador
de tareas; las diferencias de centésimas con la referencia entran dentro del
ruido de una muestra por segundo. Lo relevante es el cambio en segundo plano:

- El vigilante pasó de un bucle de sondeo fijo a eventos del sistema de archivos
  con debounce. Su CPU bajó de 0,35 % a 0,00 % y quedó con prioridad baja.
- La bandeja con el vigilante en marcha bajó de 0,52 % a 0,02 %.
- El escaneo del vigilante es incremental: solo vuelve a mirar las rutas que
  cambiaron, no el árbol completo.
- `--watch --hidden` no crea contexto OpenGL ni carga fuentes de la interfaz.

La memoria no baja: la GUI se queda en 97 MB, la bandeja en 108,8 MB y el
proceso del vigilante en 10,6 MB. La versión final anterior tiene más funciones;
la reducción importante está en el tamaño y en la CPU de fondo.

## Tamaño del ejecutable

| Compilación | GUI | CLI |
| --- | ---: | ---: |
| `opt-level = 3` | 8.167.936 B | 2.161.664 B |
| `opt-level = "s"`, `panic = "abort"` (final anterior) | 6.545.920 B | 1.675.776 B |
| Ahorro histórico | 1.622.016 B (19,9 %) | 485.888 B (22,5 %) |
| Actual, `panic = "unwind"` | 7.063.040 B | 1.846.272 B |

La comparación de `opt-level` usa la misma cadena de publicación (LTO,
`codegen-units = 1`, `strip`). El cambio a `"s"` no modificó el comportamiento en
reposo; la diferencia de CPU entre ambas compilaciones está dentro del muestreo.

## Método

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\measure.ps1 `
  -Exe ".\dist\Smart Organizer.exe" -Label final
```

El script usa una carpeta portable temporal, crea 80 archivos de prueba y mide
cuatro estados:

1. GUI visible, sin vigilante.
2. GUI minimizada, sin vigilante.
3. GUI oculta en la bandeja con el vigilante hijo en marcha (se suman ambos
   procesos).
4. Solo el proceso oculto `--watch --hidden`.

Cada estado deja pasar el arranque (15 s para la GUI y al menos 8 s tras
minimizar u ocultar), muestrea 60 veces y descarta la medición si no queda ningún
proceso vivo. Los CSV se guardan junto al informe.

## Límites

- CPU es un porcentaje de un núcleo, no del equipo completo.
- La memoria de la bandeja suma GUI y vigilante; la del vigilante solo cuenta ese
  proceso.
- El equipo tenía el escritorio y el navegador abiertos; otros procesos pueden
  añadir ruido.
- Las capturas de pantalla y la documentación no forman parte de la medición.
