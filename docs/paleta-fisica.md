# Paleta física de Grafito (estilo escandinavo didáctico)

Un solo lenguaje visual en los 46 laboratorios Dinámica: pocos colores,
cada uno con UN significado. Si un comando nuevo dibuja, usa estos.
Nada de arcoíris: el color explica, no decora.

## Colores semánticos (RGBA, mismos valores en todos los handlers)

| Rol | Color | RGB | Dónde |
|---|---|---|---|
| Peso / causa / resultado | rojo | `0.9, 0.2, 0.2` | P del plano, `a` del tiro, imagen de la lente |
| Dato inicial / objeto | verde | `0.2, 0.7, 0.2` | `v` del tiro, objeto de la lente, observador Minkowski |
| Componente tangencial | naranja | `0.9, 0.6, 0.1` | P∥ del plano, `at` del tiro, TIR del rayo |
| Componente normal | azul | `0.2, 0.4, 0.9` | N del plano, `an` del tiro |
| Luz / rayo | amarillo | `0.9, 0.8, 0.1` | cono de Minkowski, incidente, frentes Young |
| Soporte / estructura | gris | `0.5, 0.5, 0.5` | cuña, hilo, interfaz, lente |
| Magnitud continua | viridis (log) | — | campos E/B/g: `color_by_magnitude`, `t=(log10−lo)/(hi−lo)` |

## Reglas de dibujo

1. **Escala común honesta**: todas las flechas de un diagrama comparten
   escala (`|v|→0.25L`, `mg→0.25L`). Comparar longitudes = comparar física.
2. **Etiquetas cortas**: `P`, `N`, `v`, `a`, `at`, `an`, `Ppar`. Nada de
   párrafos sobre el canvas.
3. **Estructura gris, física en color**: lo que sostiene (cuña, hilo,
   interfaz) va gris; lo que enseña va en color semántico.
4. **Campos con magnitud**: viridis en escala log (E y Coulomb varían
   órdenes de magnitud); `scale_by_magnitude=false` por defecto para no
   mentir con flechas gigantes.
5. **Máximo 6 objetos nuevos por comando**: si necesita más, es receta
   (varios comandos), no un comando.

## Test que la blinda

`paleta_fisica_armonica_semantica` en `oleada2.rs`: corre
`PlanoInclinado` y `VectoresProyectil` y exige los colores exactos por
etiqueta. Cambiar un color rompe el test a propósito: la armonía se
discute, no se desliza.
