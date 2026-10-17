# Cinemática — Guía UTN (Física 1)

<!--
CC BY-SA 4.0 — Grafito Estudio. Podés copiar, anotar y compartir esta guía
citando la fuente. Los valores numéricos coinciden con los comandos
verificados de Grafito (`oleada2.rs`); si un comando cambia su salida, el
test `banco_valores_pineados` avisa.
Quiz asociado: pestaña `+` → `Quiz · Cinemática` (10 preguntas).
Plan: `+` → `Mi examen` (el quiz alimenta tu dominio por tema).
-->

## 1. Mapa de la unidad

- **MRU**: velocidad constante. `x = x₀ + v·t`.
- **MRUV**: aceleración constante. `v = v₀ + a·t`, `x = x₀ + v₀·t + ½·a·t²`, `v² = v₀² + 2·a·Δx`.
- **Caída libre / tiro vertical**: MRUV con `a = -g` (`g = 9.81 m/s²`).
- **Tiro oblicuo**: MRU en `x` + MRUV en `y`, independientes.
- **MCU**: `a_c = v²/r`, período `T = 2πr/v`.
- **Cantidad de movimiento**: `p = m·v` (se conserva si no hay impulso externo).
- **Choques elásticos 1D**: se conservan `p` y energía cinética.
- **Sonido**: `v ≈ 331.3·√(T/273.15)` m/s con `T` en kelvin.

## 2. Fórmulas que sí o sí van al examen

| Situación | Fórmula | Comando Grafito |
|---|---|---|
| Alcance tiro oblicuo | `R = v²·sin(2θ)/g` | `AlcanceProyectil[20, 45]` → `40.775` m |
| Aceleración centrípeta | `a = v²/r` | `AceleracionCentripeta[20, 50]` → `8.000` m/s² |
| Velocidad del sonido | `v = 331.3·√((T+273.15)/273.15)` | `VelocidadSonido[20]` → `343.215` m/s |
| Cantidad de movimiento | `p = m·v` | `CantidadMovimiento[1000, 20]` → `20000.000` kg·m/s |
| Choque elástico 1D | `v1′=((m1-m2)·v1+2·m2·v2)/(m1+m2)` | `ChoqueElastico[1, 3, 1, 0]` → `v1′=0.000, v2′=3.000` |
| Energía de un capacitor | `E = ½·C·U²` | `EnergiaCapacitor[0.001, 100]` → `5.000` J |

## 3. Errores típicos (los que restan puntos)

1. **Mezclar ejes en tiro oblicuo**: el tiempo de vuelo lo manda `y`, el alcance lo calcula `x`. Nunca uses el alcance para hallar el tiempo.
2. **Signos en caída libre**: elegí UN sentido positivo y sostenelo en `v₀`, `a` y `Δx`. El error más común es `g` con el signo cambiado a mitad del problema.
3. **`sin(2θ)` vs `2·sin θ`**: el alcance máximo es a 45° porque `sin(90°) = 1`.
4. **Unidades**: pasá todo a SI antes de reemplazar (`km/h ÷ 3.6 = m/s`).
5. **Choques**: elástico conserva energía; plástico no. Leé el enunciado antes de elegir la ecuación.
6. **Sonido**: la temperatura va en kelvin dentro de la raíz, no en Celsius.

## 4. Labs para tocar antes del parcial

1. `VelocidadOnda[2.0, 3.0]` — MRU puro: distancia = velocidad × tiempo.
2. `AlcanceProyectil[20, 45]` y después `[20, 30]` → `35.312` m (verificá que 45° da más).
3. `AceleracionCentripeta[20, 50]` — ¿qué pasa si duplicás `v`? (×4, no ×2).
4. `ChoqueElastico[1, 3, 1, 0]` — la de 1 kg frena en seco y la otra sale a 3 m/s.
5. `VelocidadSonido[0]` → `331.300` m/s — el "cero grados" de referencia.

## 5. Checklist pre-parcial

- [ ] Resuelvo MRUV sin mirar las 3 ecuaciones.
- [ ] Descompongo tiro oblicuo en `x`/`y` sin dudar.
- [ ] Verifico unidades en cada reemplazo.
- [ ] Hice el `Quiz · Cinemática` y mi dominio por tema supera 70%.
- [ ] Mi plan en `Mi examen` dice **Llegás**.

> Regla honesta: el porcentaje de `Mi examen` es probabilidad medida (BKT
> sobre tus respuestas reales), no temas leídos. Si dice "sin datos",
> todavía no rendiste ningún quiz de esta materia.
