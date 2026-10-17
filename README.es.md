<p align="center">
  <img src="assets/grafito-icon-256x256.png" alt="Grafito" width="128" />
</p>

<h1 align="center">Grafito</h1>

<p align="center">
  <a href="https://github.com/Diez111/Grafito/releases"><img src="https://img.shields.io/github/v/release/Diez111/Grafito?label=versi%C3%B3n&color=blue" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/licencia-GPLv3%2B-blue" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.92%2B-orange" /></a>
  <a href="https://github.com/Diez111/Grafito/stargazers"><img src="https://img.shields.io/github/stars/Diez111/Grafito?style=social" /></a>
</p>

<p align="center">
  <b>El laboratorio de matemática y física que explica antes de mostrar.</b><br />
  Geometría interactiva, álgebra computacional, estadística, cálculo y
  <b>67 laboratorios de física visual</b> + asistente tutor + pizarra —
  todo en Rust nativo, con estética minimalista escandinava.
</p>

---

## Filosofía (5 principios)

1. **Didáctico primero.** Cada comando de física enseña una idea (el signo
   antes que la fórmula, el veredicto antes que el número). Ver
   [Guía docente](#guía-docente-en-5-minutos) y `docs/fisica-animada.md` (64 recetas).
2. **Minimalista.** Interfaz escandinava: poco cromo, mucho contenido. La
   paleta agrupa por categoría y filtra por texto; nada satura.
3. **Honesto.** Si algo no se calcula, se dice (`Error` tipado, jamás un
   default silencioso). Lo congelado se etiqueta congelado.
4. **Nativo y liviano.** 100% Rust, GPU con WebGPU, GIF propio sin MP4
   pesados, recetas de código re-ejecutables en vez de videos caja negra.
5. **Para el profesor.** Recetas copiables por tema, exportación GIF para
   el aula, pizarra de 32 hojas, ejercicios deterministas con solución.

---

- [Instalación](#instalación)
- [Primeros pasos](#primeros-pasos)
- [Guía docente (en 5 minutos)](#guía-docente-en-5-minutos)
- [Laboratorio de física](#laboratorio-de-física-66--8)
- [Asistente tutor](#asistente-tutor)
- [Pizarra](#pizarra)
- [Animación liviana](#animación-liviana-gif--recetas)
- [Referencia de comandos](#referencia-de-comandos)
- [Canvas 2D](#canvas-2d)
- [Canvas 3D](#canvas-3d)
- [Arquitectura](#arquitectura)
- [Controles](#controles)
- [Desarrollo](#desarrollo)
- [Inspiración y créditos](#inspiración-y-créditos-esquema-legal)
- [Contribuir](#contribuir)
- [Licencia](#licencia)

---

## Instalación

<p><details><summary><b>Linux x86_64 (.tar.gz)</b></summary>

El archivo se construye y prueba en Ubuntu 22.04 x86_64 (glibc 2.35); requiere
glibc 2.35 o posterior, una sesión gráfica y un driver Vulkan u OpenGL
compatible con wgpu. En Debian/Ubuntu, instalá el runtime D-Bus antes de
ejecutarlo. La CI comprueba que el archivo extraído resuelva sus bibliotecas y
acepte `--help`; no prueba una sesión gráfica real:

```bash
sudo apt install libdbus-1-3
wget https://github.com/Diez111/Grafito/releases/latest/download/grafito-linux-x64.tar.gz
tar -xzf grafito-linux-x64.tar.gz
./grafito
```
</details></p>

Los enlaces `releases/latest` de esta sección apuntan solamente a la última
versión estable. Las versiones beta/alpha se publican marcadas como
*pre-release* y deben elegirse explícitamente desde [Releases](https://github.com/Diez111/Grafito/releases).

<p><details><summary><b>Windows x86_64 (.zip portátil)</b></summary>

1. Descargá [`grafito-windows-x64.zip`](https://github.com/Diez111/Grafito/releases/latest/download/grafito-windows-x64.zip).
2. Extraelo y ejecutá `grafito.exe`. No requiere instalación.
</details></p>

<p><details><summary><b>Compilar desde el código fuente</b></summary>

```bash
sudo apt install build-essential libgmp-dev libmpfr-dev libmpc-dev m4 pkg-config libdbus-1-dev
git clone https://github.com/Diez111/Grafito.git
cd Grafito
cargo run -p grafito-app --release
```
> Requiere Rust &ge; 1.92 y un compilador C (incluido en `build-essential` en Debian/Ubuntu). Los compute shaders GPU necesitan Vulkan, Metal o DX12.
</details></p>

---

## Primeros pasos

Al abrir Grafito ves un canvas con ejes cartesianos y una barra de entrada en el panel izquierdo. Tenés **dos formas** de crear cosas:

### Por comandos (escribiendo)

Escribí en la barra de entrada y presioná Enter. Grafito interpreta notación matemática natural:

```text
(x - 2)^2 + (y - 3)^2 = 25          # círculo centrado en (2,3) radio 5
y = sin(x)                           # función seno
r = 2*cos(3*theta)                   # curva polar (rosa de 3 pétalos)
(x(t), y(t)) = (cos(t), sin(t))      # curva paramétrica (círculo)
(2, 5)                               # punto en (2, 5)
Derivative[sin(x)]                   # derivada simbólica: cos(x)
Root[sin(x)]                         # raíces de la función
Taylor[exp(x), x, 0, 4]              # serie de Taylor: 1 + x + x²/2 + ...
```

### Por herramientas (haciendo clic)

La barra de herramientas (abajo) tiene íconos para dibujar directamente sobre el canvas:
- **F1** Seleccionar &bull; **F2** Punto &bull; **F3** Recta &bull; **F4** Círculo &bull; **F5** Polígono
- Cada grupo se despliega con &blacktriangledown; y muestra herramientas relacionadas

Probá: seleccioná **Círculo**, hacé clic en el canvas para el centro, después clic en otro punto cualquiera. El círculo se crea con radio igual a la distancia entre los dos puntos.

### Variables y sliders

Escribí un número o una expresión con una variable nueva y Grafito crea un **slider** automáticamente:

```text
k = 3                         # crea un slider "k" que va de 0 a 10
a*sin(x)                      # crea la variable "a" ligada a un slider
```

Arrastrá el slider en el panel izquierdo y la función `a*sin(x)` se actualiza en tiempo real. También podés animarlos (&blacktriangleright;).

### Enlace de expresiones

Casi cualquier parámetro de un objeto puede ligarse a una expresión. Escribí `PointExpr[...]`, `CircleExpr[...]`, etc., usando variables globales o sliders. El objeto se recalcula automáticamente cada vez que las variables cambian.

### Panel de análisis

Seleccioná una función desde el panel de Álgebra (pestaña **A**) y hacé clic en **Analizar** en la barra de herramientas o escribí `Analyze[f]`. Grafito calcula raíces, extremos, inflexiones, asíntotas e interceptos, y marca los puntos sobre el canvas.

---

## Guía docente (en 5 minutos)

1. **Elegí el tema.** Abrí `docs/fisica-animada.md`: 64 recetas copiables
   (tiro oblicuo, dipolo, RLC, Doppler, Young, Carnot, lente, dilatación,
   órbita, Lorentz, decibeles…).
2. **Pegá 2 líneas en la barra de entrada.** Cada receta son 1–3 comandos;
   el dibujo aparece con colores que codifican significado
   (verde = velocidad, rojo = aceleración, amarillo = luz…).
3. **Mové un slider.** `k = 3` + arrastrar = la clase pregunta "¿y si…?"
   y el canvas responde en vivo.
4. **Exportá el GIF.** Animación liviana nativa (12 fps, hasta 64 cuadros,
   tope 5 MiB): pesa KB, se pega en cualquier apunte o aula virtual.
5. **Evaluá con el tutor.** El asistente genera ejercicios deterministas
   con solución exacta (18 familias) para los programas UTN
   (AM1, AM2, Álgebra, Probabilidad, Física 1, Física 2, Discreta).
6. **Anotá en la pizarra.** 7 herramientas + goma WYSIWYG + libro de
   32 hojas guardado dentro del documento.

> Todo lo que ves se puede re-ejecutar: no hay videos caja negra ni
> MP4 pesados. Si un cálculo no existe, Grafito devuelve un error honesto
> en vez de inventar un número.

---

## Laboratorio de física (66 + 8)

La categoría **Dinámica** trae 67 laboratorios visuales agrupados por rama,
más 8 atractores caóticos clásicos (`Lorenz`, `Rossler`, `Thomas`, `Chen`,
`Chua`, `Aizawa`, `Dadras`, `Halvorsen`). Las 64 recetas didácticas viven en
[`docs/fisica-animada.md`](docs/fisica-animada.md) (§1–§63).

### Mecánica

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `TrayectoriaProyectil[v0, angulo, h0]` | Parábola del tiro (g = 9.8) |
| `VectoresProyectil[v0, angulo, h0, t]` | Punto + v + a + componentes tangencial/normal |
| `CaidaRozamiento[m, c, v0]` | v(t) con aire de Stokes + velocidad terminal |
| `Pendulo[L, theta0]` | Hilo + lenteja + período corregido hasta 60° |
| `OsciladorArmonico[A, omega, phi]` | MAS como función viva (derivable, integrable) |
| `BarraHooke[L0, k, F]` | Barra original + deformada (verde tracción / roja compresión) |
| `PlanoInclinado[m, theta]` | Cuña + P + P∥ + N con escala común (a = g·sinθ, sin rozamiento) |
| `MovimientoAcelerado[x0, v0, a]` | x(t) MRUV viva (a = 0 → MRU, sin comando aparte) |
| `EnergiaMecanica[m, v, h]` | Ec + Ep + Em (la cuenta que el plano promete) |
| `Trabajo[F, d]` | W = F·d (entra trabajo, queda energía; negativo frena) |
| `PotenciaMecanica[W, t]` | P = W/t (qué tan rápido; t = 0 → error, no infinito) |
| `FuerzaRozamiento[mu, N]` | fr = μN (el plano era ideal; con μ el bloque puede no deslizar) |
| `CantidadMovimiento[m, v]` | p = m·v en kg·m/s (lo que el choque interno conserva) |
| `ChoqueElastico[m1, v1, m2, v2]` | v₁′v₂′ del choque 1D (iguales intercambian; 10 vs 1 → 1.636 y 3.636) |
| `VelocidadSonido[T]` | v = 331.3·√(T/273.15) en aire (20 °C → 343.215 m/s; T ≥ −273.15) |
| `AlcanceProyectil[v, ang]` | R = v²·sin(2θ)/g sin rozamiento (20 m/s a 45° → 40.775 m) |
| `EnergiaCapacitor[C, U]` | E = ½·C·U² guardada (1 mF a 100 V → 5 J; el signo de U no importa) |
| `AceleracionCentripeta[v, r]` | a = v²/r hacia el centro (dobla sin frenar) |

### Electromagnetismo y circuitos

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `CampoCargas[q1, x1, y1, ...]` | Campo E con color log-viridis por magnitud |
| `EquipotencialCargas[q1, x1, y1, ...]` | 8 curvas V con V ⊥ E visible |
| `CampoHilo[I, x0, y0]` | B azimutal de Biot-Savart (regla mano derecha) |
| `FemInducida[N, B, A, omega]` | fem = NBAω·sin(ωt) como función viva |
| `LeyFaraday[N, dPhi, dt]` | fem = −N·ΔΦ/Δt discreta + Lenz (la espira generalizada) |
| `FlujoGauss[Q]` | Φ = Q/ε₀ de la carga encerrada (1 nC → 112.941 N·m²/C) |
| `CargaCapacitor[V, R, C]` | Carga RC exponencial (misma EDO que desintegración) |
| `ResonanciaRLC[R, L, C]` | Curva I(f) + f₀ + Q (bajar R estrecha la campana) |
| `FuerzaCoulomb[q1, q2, r]` | F = Kq₁q₂/r² + veredicto REPULSIÓN / ATRACCIÓN |
| `FuerzaLorentz[q, v, B]` | F = qvB + sentido mano derecha (el hilo dibuja B, este pone F) |
| `LeyOhm[V, R]` | I = V/R + P de regalo (R ≤ 0 → error, no infinito) |
| `CampoCargas3D[q1, x1, y1, z1, …]` | E 3D + esferas por carga (el dipolo real es volumen, no plano) |

### Gravitación

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `CampoGravitatorio[m1, x1, y1, ...]` | Campo g atractivo (hermano de Coulomb, signo opuesto) |
| `PotencialGravitatorio[m1, x1, y1, ...]` | 4 pozos V < 0 (eléctrico tenía ±, gravedad solo −) |
| `VelocidadEscape[M, R]` | v = √(2GM/R) con G real (Tierra → 11.186 km/s) |
| `OrbitaCircular[M, r]` | v = √(GM/r) + período (ISS → 7672 m/s, 92 min; √2 vs escape) |
| `SistemaOrbital[M, r, inclinacion]` | Órbita 3D inclinada + v/T reales (ISS a 51.6°; dibujo normalizado R = 5) |
| `VelocidadOnda[lambda, f]` | Onda viajera en 2 fotos (t = 0 y t = T/4) + v = λf (la gris avanza λ/4) |
| `TerceraKepler[M]` | k = T²/r³ = 4π²/GM (Tierra → 9.904e-14; verifica tu órbita) |
| `LeyGravitacion[m1, m2, r]` | F = G·m1·m2/r² con G real (Tierra–Luna → 1.980e20 N) |

### Fluidos

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `TuboVenturi[A1, A2, v1, p1, rho]` | Continuidad + Bernoulli dibujados (anchos ∝ A, flechas ∝ v) |
| `PresionHidrostatica[rho, h]` | p = ρgh + atmósferas (agua quieta pesa) |
| `EmpujeArquimedes[rho, V, m]` | E = ρgV vs peso + veredicto FLOTA / SE HUNDE |
| `NumeroReynolds[rho, v, L, mu]` | Re + régimen LAMINAR / TRANSICIÓN / TURBULENTO |
| `Densidad[m, V]` | ρ = m/V + FLOTA / SE HUNDE en agua (decide antes que Arquímedes) |

### Termodinámica y radiación

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `ConduccionTermica[T1, T2, L, k]` | Perfil T(x) lineal + flujo de Fourier (la pendiente ES el gradiente) |
| `GasIdeal[n, T]` | Isoterma p = nRT/V viva (dos T comparan Boyle, área = trabajo) |
| `CicloCarnot[Tc, Th]` | Banda entre isotermas + η techo (no promete el ciclo) |
| `LeyWien[T]` | λ_max = b/T (el color dice la temperatura) |
| `PotenciaStefan[T, A]` | P = σAT⁴ (Wien el color, Stefan la potencia: el Sol en 2 líneas) |
| `DilatacionLineal[L0, alpha, dT]` | ΔL = L₀αΔT + L final (el puente respira; el frío acorta) |
| `CalorEspecifico[m, c, dT]` | Q = mcΔT (la pava eléctrica: 335 kJ; ΔT < 0 cede, no falla) |
| `CalorLatente[m, L]` | Q = m·L del cambio de fase (1 kg hielo → 334 kJ sin subir un grado) |
| `LeyEnfriamiento[T0, Tamb, k]` | T(t) viva + τ (café + capacitor + núcleo: una sola EDO) |

### Ondas y óptica

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `OndaEstacionaria[L, n, A]` | Envolvente + nodos fijos (n entero 1–16) |
| `EfectoDoppler[v, c, f]` | 6 frentes desplazados + f' acercando/alejando |
| `DobleRendija[lambda, d, L]` | Franjas I = cos² (sin envolvente, dicho en el mensaje) |
| `RayoRefraccion[n1, n2, theta1]` | Snell en grados + TIR honesta con θc |
| `LenteDelgada[f, do, ho]` | Diagrama de rayos + imagen real (construcción que cierra) |
| `EspejoEsferico[f, do, ho]` | Misma fórmula, imagen del mismo lado |
| `LeyMalus[I0, theta]` | I = I₀cos²θ + extinción a 90° |
| `NivelSonoro[I]` | β = 10·log₁₀(I/I₀) + veredicto (10 motosierras ≠ 10× una) |

### Física moderna

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `EfectoFotoelectrico[f, W]` | K_max = hf − W o "sin emisión" (umbral honesto) |
| `EfectoCompton[theta]` | Δλ = λc(1 − cosθ) (el fotón rebota más rojo) |
| `Desintegracion[N0, lambda]` | N₀e^(−λt) + vida media (misma EDO que RC) |
| `DiagramaMinkowski[v, c]` | Cono de luz + línea de universo + γ (el límite c se ve) |
| `EnergiaBohr[n]` | E_n = −13.6/n² eV del H (n = 1 → −13.6, n = 2 → −3.4) |
| `OndaDeBroglie[m, v]` | λ = h/mv (electrón 0.73 nm difracta, vos a 1e-36 m no) |

### Química

| Comando | Qué dibuja / calcula |
|---------|----------------------|
| `PH[c]` | pH = −log₁₀[H⁺] + veredicto ÁCIDO / NEUTRO / BASE |

> Los campos vectoriales colorean por magnitud (viridis logarítmica,
> configurable) y pueden escalar flechas por magnitud. Ver recetas §1–§44
> para la secuencia didáctica completa rama por rama.
>
> Todo dibujo sigue la skill `estilo-escandinavo-grafito`
> (`.agents/skills/estilo-escandinavo-grafito/SKILL.md`): un color = un
> significado (`docs/paleta-fisica.md`), etiquetas de 1–2 letras, escala
> honesta, cero pantallas nuevas. El test `paleta_fisica_armonica_semantica`
> rompe si un color se desliza.

---

## Asistente tutor

El asistente entiende el nivel pedagógico (`fis1`, `fisica i`, `fis2`,
`discreta`, …), genera ejercicios deterministas sembrados (misma semilla =
mismo ejercicio, con solución exacta derivada de los parámetros) y valida con
tolerancia numérica.

**Programas UTN cubiertos:** AM1, AM2, Álgebra, Probabilidad, Física 1,
Física 2, Discreta — 68 objetivos de aprendizaje en
`crates/grafito-pedagogy/src/curriculum.rs`.

**Familias de ejercicios** (18, creciendo con cada laboratorio nuevo):

| Familia | Pregunta | Puente con |
|---------|----------|------------|
| `alg-matrices` | determinante 2×2 | Álgebra |
| `am2-edo` | EDO lineal | AM2 |
| `prob-var` | distribuciones | Probabilidad |
| `sec-fracc` | fracciones | Secundaria |
| `fis1-cinematica` | MRU: x = v·t | Cinemática |
| `dis-combinatoria` | C(n,k) exacto | Discreta |
| `fis-resonancia` | f₀ = 1/2π√(LC) | `ResonanciaRLC` |
| `fis-doppler` | f' acercando | `EfectoDoppler` |
| `fis-pendulo` | T₀ = 2π√(L/g) | `Pendulo` |
| `fis-energia` | Em = ½mv² + mgh | `EnergiaMecanica` |

> Alcance honesto: el tutor genera, valida y da feedback sobre estas
> familias y niveles; no promete corrección simbólica general ni
> planificación adaptativa (roadmap: FSRS + banco completo de LOs).

---

## Pizarra

Pizarra pro integrada al documento (libro de 32 hojas):

- **8 herramientas:** seleccionar, lápiz, resaltador translúcido,
  rectángulo, elipse, flecha, texto, goma. Grosor 1–24 + 8 colores;
  el lápiz convierte círculo→elipse y recta→flecha (opcional).
- **Instrumentos:** regla, escuadra y transportador arrastrables
  (el trazo se pega al filo) + compás a círculo exacto.
- **Imágenes:** archivo PNG/JPEG, pegar del portapapeles y fórmulas
  (raster real del motor math); límites honestos 4 MiB/2048 px.
- **Presentar:** cortina reveladora, lupa 2×, fondos (cuadriculado,
  rayado, punteado, blanco), 12 figuras (5 geometría + plano,
  transportador, cuartos, hexágono, cubo, reloj, tabla), export PNG/SVG.
- **Aula:** rol Profesor (presentar + exportar) / Alumno (preguntar
  a Mili + seguir), todo local sin cuentas.
- **Goma WYSIWYG:** el círculo del cursor es exactamente la zona que borra.
- **Límites sanos:** 500 trazos por hoja, 2000 puntos por trazo —
  la pizarra nunca cuelga el documento.
- **Didáctica:** anotar sobre el laboratorio (rodear el nodo, tachar la
  asíntota) sin salir de Grafito.

---

## Móvil y Android (misma UI, adaptativa)

Cero pantallas funcionales nuevas: solo el contenedor cambia.

- **Paleta bottom-sheet** bajo 740 px (62 % del alto, al alcance del pulgar);
  chips por rama y MRU horizontales. Desktop intacto.
- **Nivel 0** bajo 480 px: sin rail ni drawers; pill de herramienta + FAB
  lupa 56 px + entrada táctil de 44 px. Gestos: 2 dedos navegan
  (canvas y pizarra), 1 dedo construye.
- **Aula** en bottom-sheet 55 % con cierre explícito; modales con margen
  y botones apilados en angosto.
- **Android:** núcleo didáctico (`core`, `geometry`, `command`, `pedagogy`,
  `tex`, `ggb`, `complex`) compila a `aarch64-linux-android` sin
  cambios; `rfd`/`arboard` viven tras la feature `desktop-dialogs`
  (fallback honesto sin diálogos ni portapapeles). Falta el entry nativo
  (ver `docs/portabilidad-android.md`).

---

## Animación liviana (GIF + recetas)

- **GIF nativo** (crate `gif` puro Rust): 12 fps, hasta 64 cuadros,
  tope 5 MiB. Ideal para apuntes y aulas virtuales.
- **69 plantillas canónicas** re-ejecutables (`protocol.rs`): KB de código
  en vez de MB de video; la animación se regenera idéntica.
- **MP4 solo bajo demanda** con ffmpeg externo, nunca pesado por defecto.
- Detalles en [`docs/anim-liviano.md`](docs/anim-liviano.md).

---

## Referencia de comandos

Todos los comandos usan la sintaxis `Comando[arg1, arg2, ...]`. Casi todos tienen alias en español (ej: `Derivada`, `Integral`). Escribí `Ctrl+K` para ver la lista completa (**764 comandos** registrados; el catálogo generado vive en [`docs/commands.md`](docs/commands.md)). Además, Ctrl+K entiende ramas: `fluidos`, `termodinamica`, `optica`, `ondas`, `mecanica`, `cuantica`…

### Creación de objetos

| Sintaxis | Resultado |
|----------|-----------|
| `(x, y)` | Punto 2D |
| `(x, y, z)` | Punto 3D |
| `y = expr` o `f(x) = expr` | Función y = f(x) |
| `r = expr` o `r(theta) = expr` | Curva polar |
| `(x(t), y(t)) = (fx, fy)` | Curva paramétrica 2D |
| `lhs = rhs` o `lhs >= rhs` | Curva implícita |
| `expr` con `x` libre | Función (variable implícita) |
| `expr` numérica | Crea un slider con ese valor |
| `k = valor` | Crea un slider nombrado |

### Álgebra computacional (CAS)

| Comando | Ejemplo | Resultado |
|---------|---------|-----------|
| `Derivative` / `Derivada` | `Derivative[x^3]` | `3*x^2` |
| `Integral` / `Integrar` | `Integral[sin(x), x, 0, pi]` | `2` (definida) |
| `Solve` / `Resolver` | `Solve[x^2 - 4]` | `{-2, 2}` |
| `Limit` / `Limite` | `Limit[sin(x)/x, x, 0]` | `1` |
| `Factor` / `Factorizar` | `Factor[x^2 + 5x + 6]` | `(x+2)*(x+3)` |
| `Expand` / `Expandir` | `Expand[(x+1)^2]` | `x^2 + 2*x + 1` |
| `Simplify` / `Simplificar` | `Simplify[2x + x]` | `3*x` |
| `Taylor` | `Taylor[cos(x), x, 0, 5]` | `1 - x^2/2 + x^4/24` |

### Análisis de funciones

| Comando | Resultado |
|---------|-----------|
| `Root[f]` | Raíces (f(x) = 0), marca puntos en el canvas |
| `Extremum[f]` | Máximos y mínimos locales |
| `Inflection[f]` | Puntos de inflexión |
| `YIntercept[f]` | Intersección con el eje Y |
| `XIntercept[f]` | Intersecciones con el eje X |
| `Analyze[f]` | Análisis completo (todo lo anterior + asíntotas) |
| `Intersect[a, b]` | Intersección entre dos objetos |

### Cálculo avanzado

| Comando | Ejemplo |
|---------|---------|
| `TangentAt` / `TangenteEn` | `TangentAt[sin(x), 0]` &rarr; recta tangente en x=0 |
| `NormalAt` / `NormalEn` | `NormalAt[sin(x), pi/2]` &rarr; recta normal |
| `ArcLength` / `LongitudArco` | `ArcLength[x^2, 0, 1]` |
| `CurvatureAt` / `CurvaturaEn` | `CurvatureAt[x^3, 0]` &rarr; curvatura y radio |
| `VolumeOfRevolution` / `VolumenRevolucion` | `VolumeOfRevolution[sin(x), 0, pi]` |
| `SurfaceOfRevolution` / `SuperficieRevolucion` | `SurfaceOfRevolution[x^2, 0, 1]` |

### Construcciones geométricas

| Comando | Descripción |
|---------|-------------|
| `Segment[a, b]` | Segmento entre dos puntos |
| `Ray[origen, direccion]` | Semirrecta |
| `Vector[origen, destino]` | Vector |
| `Perpendicular[punto, recta]` | Recta perpendicular |
| `Midpoint[a, b]` | Punto medio entre dos puntos |
| `Intersect[a, b]` | Intersección(es) entre dos objetos |
| `RegularPolygon[centro, lados, radio]` | Polígono regular (3 a 64 lados) |
| `Reflect[objeto, p1, p2]` | Reflejar objeto respecto a una recta |
| `Translate[objeto, vector]` | Trasladar objeto |
| `Rotate[objeto, centro, angulo]` | Rotar objeto (grados) |
| `Dilate[objeto, centro, factor]` | Dilatar objeto |

### Restricciones numéricas

| Comando | Descripción |
|---------|-------------|
| `Distance[a, b, valor]` | Fija la distancia entre a y b |
| `Angle[l1, l2, grados]` | Fija el ángulo entre dos rectas |
| `Tangent[c1, c2]` | Fuerza tangencia entre círculos |
| `Coincident[a, b]` | Fuerza coincidencia de dos puntos |
| `Horizontal[segmento]` | Fuerza horizontalidad |
| `Vertical[segmento]` | Fuerza verticalidad |
| `EqualLength[s1, s2]` | Fuerza igualdad de longitud |
| `Symmetry[p, q, eje]` | Fuerza simetría de p y q |

### Estadística

Escribí listas de datos con `{ }` o usá variables:

```text
datos = {1.2, 3.5, 2.1, 4.8, 2.9}
Mean[{1.2, 3.5, 2.1, 4.8, 2.9}]        # media
StdDev[datos]                             # desviación estándar
Histogram[datos, 5]                       # histograma con 5 bins, dibuja barras
ScatterPlot[{xs}, {ys}]                   # diagrama de dispersión
BoxPlot[datos]                            # diagrama de caja con outliers
LinearRegression[{xs}, {ys}]              # recta de regresión + R²
Correlation[{xs}, {ys}]                   # correlación de Pearson
```

| Comando adicional | Resultado |
|-------------------|-----------|
| `Median[datos]` | Mediana |
| `Mode[datos]` | Moda |
| `Variance[datos]` | Varianza muestral |
| `Percentile[datos, p]` | Percentil (0 a 1) |
| `Q1[datos]` / `Q3[datos]` | Cuartiles (IQR = Q3−Q1) |
| `Covariance[{xs}, {ys}]` | Covarianza |

### Fractales y atractores

```text
Mandelbrot[256]                     # fractal de Mandelbrot
Julia[-0.70176, -0.3842, 128]       # conjunto de Julia
BurningShip[128]                    # fractal Burning Ship
Lorenz[10, 28, 2.666]              # atractor de Lorenz
Rossler[0.2, 0.2, 5.7]            # atractor de Rössler
Thomas[0.208186]                    # mariposa de Thomas
```

### Otros comandos

| Comando | Descripción |
|---------|-------------|
| `ComplexMapping[1/z, obj]` | Mapeo complejo sobre cualquier objeto 2D |
| `VectorField2D[u, v]` | Campo vectorial (x,y) &rarr; (u,v) con flechas |
| `Piecewise[{expr1, cond1}, {expr2, cond2}]` | Función por partes |
| `DomainColoring[1/z]` | Coloración de dominio complejo |
| `HeatMap[exp(-(x^2 + y^2))]` | Mapa de calor |
| `Erase[label]` / `EraseAll` | Borrar objetos |
| `Script[comando1; comando2; ...]` | Ejecutar varios comandos a la vez |

---

## Canvas 2D

Grafito renderiza más de 30 tipos de objetos geométricos sobre un canvas interactivo. Podés crear objetos escribiendo comandos o usando las herramientas de la barra inferior.

### Objetos disponibles

Puntos, rectas, segmentos, semirrectas, vectores, círculos, elipses, parábolas, hipérbolas, polígonos, polígonos regulares, funciones explícitas, curvas paramétricas 2D, curvas polares, curvas implícitas, campos vectoriales 2D, lugares geométricos, dibujos a mano alzada (lápiz y borrador).

### Operaciones booleanas entre polígonos

```text
PolygonUnion[p1, p2]           # unión
PolygonIntersection[p1, p2]    # intersección
PolygonDifference[p1, p2]      # diferencia (p1 menos p2)
PolygonXor[p1, p2]             # diferencia simétrica (XOR)
```

### Cónicas por construcción geométrica

```text
EllipseByFoci[F1, F2, P]                 # elipse dados focos y punto
ParabolaByFocusDirectrix[F, directriz]   # parábola por foco y directriz
HyperbolaByFoci[F1, F2, P]               # hipérbola por focos y punto
ConicByFivePoints[A, B, C, D, E]         # cónica por 5 puntos
```

### Curvas especiales

```text
Cardioid[1.5]                           # cardioide r=a(1+cos θ)
Rose[2, 3, 1]                           # rosa r = a*cos(n/d*θ)
Lissajous[3, 4, 1, 1, 90]              # curva de Lissajous
ArchimedeanSpiral[0.5, 0.2, 10]        # espiral de Arquímedes
LogarithmicSpiral[1, 0.2, 10]          # espiral logarítmica
Epicycloid[2, 3]                        # epicicloide
```

### Mapeos complejos

Aplicá cualquier función de variable compleja a un objeto 2D. El target debe
existir en el documento; sin target se usa el **disco unidad `I`** (se crea
si falta):

```text
ComplexMapping[exp(z)]                   # exp(z) sobre el disco unidad I
ComplexMapping[z^2 + 1]                  # cualquier expresión de z (afines, polinomios, …)
Circle[(0, 0), 3]                        # crea el círculo C (etiqueta automática)
ComplexMapping[1/z, C]                   # invierte ese círculo
ComplexMapping[z^2, C]                   # transformación cuadrática
```

Sobre una región (`x^2 + y^2 < 1`, `Circle`) el mapeo dibuja borde, relleno
(cuando la expresión tiene inversa) y una **retícula de referencia** que hace
visible la deformación interior (p. ej. `z` vs `z^5`). Las singularidades
generan automáticamente asíntotas punteadas.

#### Coloración de dominio (domain coloring) acelerada por GPU

La coloración de dominio (estilo cplot) muestra el argumento de `f(z)` como
matiz y `log|f(z)|` como luminosidad. En Grafito 1.1.9-beta la evaluación
de la grilla 2D corre en un compute shader WGSL (`DomainColoringComputePipeline`),
permitiendo hasta 500×500 = **250 000 celdas evaluadas en un solo dispatch GPU**
con fallback automático a CPU si la GPU no está disponible.

```text
DomainColoring[1/z, -2, 2, -2, 2, 200]   # inversión con domain coloring
DomainColoring[z^6+1, -1.5, 1.5, -1.5, 1.5, 200]
DomainColoring[exp(z), -3, 3, -3, 3, 200]
DomainColoring[gamma(z), -3, 3, -3, 3, 200]   # ceros y polos de Γ(z)
```

#### Funciones complejas especiales

Grafito incluye un conjunto amplio de funciones complejas, disponibles tanto
en la línea de comandos como dentro de cualquier expresión:

| Función | Descripción | GPU |
|---------|-------------|:---:|
| `sin`, `cos`, `tan`, `sec`, `csc`, `cot` | Trigonométricas | ✓ |
| `asin`, `acos`, `atan` | Inversas | ✓ |
| `sinh`, `cosh`, `tanh`, `coth` | Hiperbólicas | ✓ |
| `asinh`, `acosh`, `atanh` | Hiperbólicas inversas | ✓ |
| `exp`, `log`/`ln`, `sqrt`, `pow` | Exponenciales | ✓ |
| `conj(z)` | Conjugado `x − iy` | ✓ |
| `re(z)`, `im(z)` | Parte real / imaginaria | ✓ |
| `arg(z)` | Argumento `atan2(im, re)` | ✓ |
| `abs(z)` | Módulo | ✓ |
| `gamma(z)` | Función Gamma (Lanczos + reflexión) | CPU |
| `bessel_j(z)` | Bessel de 1ª clase (series + integral) | CPU |
| `bessel_y(z)` | Bessel de 2ª clase (serie + narmónicos) | CPU |
| `erf(z)` | Función de error (series + asintótica) | CPU |
| `lambert_w(z)` | Lambert W (Newton) | CPU |
| `zeta(z)` | Zeta de Riemann (Borwein + funcional) | CPU |

> **Nota:** las funciones especiales (`gamma`, `zeta`, `erf`, `lambert_w`,
> `bessel_j`, `bessel_y`) se evalúan en CPU. Si la GPU las encuentra en el
> bytecode, devuelve NaN en esa celda — el sampler cae al path CPU
> automáticamente. Esto preserva la velocidad de domain coloring para
> funciones trigonométricas y algebraicas, mientras permite usar las
> especiales en el sampler interactivo.

#### Cuadrantes del plano complejo

```text
Quadrants[-5, 5, -5, 5]
```

Pinta los cuatro cuadrantes Q1-Q4 con colores distintivos, etiquetas
`+Re`/`+Im` y marcas en los ejes. Útil como overlay pedagógico para
entender inversiones, conjugaciones y mapeos conformes.

#### Superficie 3D de módulo complejo

```text
ComplexSurface[z^2 + 1, -3, 3, -3, 3, 50]      # |z² + 1|
ComplexSurface[1/z, -2, 2, -2, 2, 50]          # singularidad en el origen
ComplexSurface[gamma(z), -3, 3, -3, 3, 60]     # polos de Γ(z)
```

Genera una `Surface3DObj` cuya altura es `|f(x + iy)|`. Se renderiza con
el pipeline GPU de superficies paramétricas existente. Alias:
`complexsurface`, `csurface`.

#### Animación trigonométrica

Menú **Herramientas → Animación Trigonométrica** (o `Tool::TrigAnimation`)
abre un panel lateral con:

- **Círculo unitario** con vector radio animado y proyecciones a ejes
  (coseno en verde, seno en azul).
- **Gráfico 2D** sincronizado de `sin(t)`, `cos(t)` o `tan(t)` con
  línea vertical marcando el ángulo actual.
- **Controles** play/pause, slider de velocidad (±3 rad/s), slider
  manual del ángulo (±2π) y etiqueta del valor numérico en tiempo real.

### Solver de restricciones

Grafito resuelve sistemas de restricciones geométricas con el método de Newton usando Jacobianos analíticos. Las restricciones se pueden combinar: por ejemplo, un triángulo con distancias fijas y un ángulo recto.

---

## Canvas 3D

El visor 3D se activa automáticamente cuando creás objetos 3D. Podés orbitar la cámara arrastrando con el botón derecho, hacer zoom con la rueda del ratón, y paneo arrastrando con la herramienta Seleccionar.

### Objetos 3D

Puntos, segmentos, esferas, cubos, pirámides, conos, cilindros, toros, cintas de Moebius, superficies paramétricas, curvas 3D, atractores extraños, campos vectoriales 3D y proyecciones multidimensionales. Incluye los seis politopos regulares 4D (pentácoron, teseracto, 16-celdas, 24-celdas, 120-celdas y 600-celdas) y las familias simplex, hipercubo y politopo cruzado entre 3 y 10 dimensiones.

### Ejemplos

```text
Sphere[r]                           # esfera de radio r
Cube[l]                             # cubo de lado l
Curve3D[(cos(t), sin(t), t/5), 0, 20]    # hélice
Lorenz[]                            # atractor de Lorenz en 3D
Hypercube[angle1, angle2, angle3]   # teseracto proyectado
Tesseract4D[]                       # teseracto regular 4D con caras y aristas
SixHundredCell4D[0.8]               # 600-celdas proyectado a 3D
HypercubeND[5, 1.0]                 # hipercubo regular en R^5, wireframe
```

### Renderizado GPU

El canvas 2D y 3D se renderizan con `wgpu` (WebGPU) usando compute shaders que compilan las expresiones del usuario a bytecode WGSL en tiempo de ejecución. Los politopos 4D se proyectan con precisión `f64` y se agrupan en los streams GPU 3D existentes para caras con profundidad y aristas; si la GPU no está disponible o hay rotación multidimensional activa, Grafito usa automáticamente un camino CPU acotado. Esto acelera la evaluación de funciones, curvas implícitas y superficies paramétricas en órdenes de magnitud comparado con la CPU.

Pipelines de cómputo disponibles (`crates/grafito-render/src/`):

- `function_compute` — evalúa `y = f(x)` en una grilla 1D.
- `implicit_compute` — evalúa `f(x, y) = 0` sobre una grilla 2D para marching squares.
- `parametric_compute` — evalúa curvas 2D/3D y superficies paramétricas.
- `vector_compute` — evalúa campos vectoriales 2D.
- `complex_compute` — evalúa expresiones complejas sobre vértices (usado en `ComplexMapping`).
- `domain_coloring_compute` — **nuevo en 1.1.9-beta**: evalúa `f(z)` sobre una grilla 2D y produce colores HSL por celda (hasta 250 000 celdas por dispatch, usado por `DomainColoring[...]`).
- `fill_compute` — pinta regiones rellenas de curvas implícitas.

---

## Arquitectura

```
grafito/
├── crates/
│   ├── grafito-app/         Aplicación (eframe/egui) — UI, entrada, orquestación
│   ├── grafito-command/     Comandos de texto: registry (818), CAS-parse, capacidades del asistente
│   ├── grafito-core/        Documento, objetos geométricos, índice espacial, restricciones
│   ├── grafito-geometry/    Motor matemático — CAS, estadística, EDOs, fractales, booleanas
│   ├── grafito-render/      Pipeline wgpu — render 2D/3D, compute shaders, iluminación
│   ├── grafito-ui/          Componentes egui — toolbar, paleta de comandos, temas
│   ├── grafito-anim/        Protocolo de animación (69 plantillas) + GIF nativo liviano
│   ├── grafito-assistant/   Agente tutor pedagógico
│   ├── grafito-assistant-types/  Tipos compartidos del asistente
│   ├── grafito-agent/       Orquestación del agente
│   ├── grafito-pedagogy/    Niveles UTN, currículum (68 LOs), ejercicios (18 familias), feedback
│   ├── grafito-whiteboard/  Pizarra: trazos, formas, texto, libro de 32 hojas
│   ├── grafito-complex/     Funciones complejas especiales (gamma, zeta, Bessel…)
│   ├── grafito-ggb/         Interop GeoGebra (import/export)
│   ├── grafito-lab/         Laboratorios interactivos
│   ├── grafito-classroom/   Aula (P2P, sesiones)
│   ├── grafito-tex/         Render de fórmulas TeX
│   ├── grafito-mcp/         Servidor MCP (protocolo de contexto de modelo)
│   ├── grafito-plugins/     Sistema de plugins
│   └── grafito-profile/     Perfiles de rendimiento
└── assets/                  Iconos, shaders WGSL
```

---

## Controles

| Acción | Entrada |
|--------|--------|
| Pan 2D | Arrastrar en vacío / Espacio+arrastrar / clic medio |
| Zoom 2D/3D | Rueda del ratón |
| Crear objeto | Clic (Punto: clic simple) |
| Cerrar polígono | Clic derecho (3+ vértices) |
| Cancelar | Clic derecho (1 punto pendiente) / Escape |
| Seleccionar | Clic con herramienta Seleccionar |
| Deseleccionar | Clic en vacío |
| Orbitar 3D | Botón derecho + arrastrar |
| Deshacer / Rehacer | Ctrl+Z / Ctrl+Y (Ctrl+Shift+Y: Intercepto Y) |
| Eliminar objeto | Suprimir (con objeto seleccionado) |
| Paleta de comandos | Ctrl+K |
| Foto → solver | Adjuntar imagen en el asistente: si hay `tesseract` (spa+eng) extrae el texto para revisar y después “resolvé … paso a paso” (nunca auto-ejecuta) |
| Solver paso a paso | `StepByStep[Derivative[x^2, x]]` o “resolvé x^2+3x=0 paso a paso” (tarjeta con revelado + gráfico; también Laplace, Simplify/Factor/Expand y plantilla EDO/Euler/Frobenius) |
| Herramientas 2D | F1 Seleccionar / F2 Punto / F3 Recta / F4 Círculo / F5 Polígono / F6 Función / F8 Esfera / F9 Cubo |
| Herramientas por tecla | R Raíz / E Extremo / I Intercepto X / X Intersección / N Inflexión / S Segmento / Y Semirrecta / V Vector / M Punto medio |
| Análisis | Ctrl+A Analizar |
| Lápiz / Borrador | Ctrl+P / Ctrl+E |
| Tema claro/oscuro | Ctrl+T (también en Vista > Modo oscuro) |
| Grilla y ejes | Shift+L (eje X log), Shift+K (eje Y log), Shift+J (ambos), G (snap a grilla) |
| Abrir / Guardar | Ctrl+N Nuevo / Ctrl+O Abrir / Ctrl+S Guardar / Ctrl+Shift+S Guardar como |
| Perspectivas | Ctrl+Shift+1..9,0 (10 perspectivas) |

---

## Desarrollo

### Verificar todo

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

### Perfilar frames

```bash
cargo run -p grafito-app --features profile -- --profile
# Conectar puffin_viewer a 127.0.0.1:8585
```

### Compilar para Windows desde Linux

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
# Las dependencias C (GMP/MPFR) requieren compilación cruzada:
# Usá el GitHub Actions workflow 'build-windows.yml' como alternativa
```

---

## Inspiración y créditos (esquema legal)

Grafito es 100% código propio bajo **GPL-3.0-or-later**. **Ningún código
de terceros está copiado en este repositorio**: las dependencias externas
viven declaradas en `Cargo.toml` / `Cargo.lock` con sus licencias
compatibles. Lo que sigue es atribución académica de buena fe por las
**ideas y patrones didácticos** consultados (las ideas no tienen copyright,
pero el crédito sí tiene valor), con el estado de licencia verificado de
cada proyecto en octubre 2026:

| Proyecto | Qué nos enseñó | Licencia verificada | Recurso extraído |
|----------|----------------|---------------------|------------------|
| [PhET (Univ. Colorado Boulder)](https://github.com/phetsims) | Patrones didácticos: play/pausa/paso, sliders, "¿y si…?" en vivo | GPL-3.0 por sim; marca de UC Boulder — **inspiración, sin código** | Guía docente §1–§3 |
| [myPhysicsLab](https://github.com/myphysicslab/myphysicslab) | RK4 como solver de referencia, derivación abierta del péndulo doble, ~50 sims como mapa de cobertura | **Apache-2.0** — fórmulas y métodos como referencia teórica | Receta caos/péndulo, roadmap RK4 nativo |
| [manim-physics](https://github.com/Matheart/manim-physics) | Mapa de ramas (electrostática, magneto, óptica/lentes, ondas, rígidos) espejado en nuestros 67 laboratorios | **Sin licencia declarada** (issue abierto en su repo) — **solo ideas, cero código** | Estructura de `docs/fisica-animada.md` |
| [matter.js](https://github.com/liabru/matter-js) (Liam Brummitt) | Patrones de sandbox 2D: gravedad, fricción, restitución, toggles | **MIT** — inspiración de UX, sin código | Roadmap simulador rígido |
| [Rapier](https://github.com/dimforge/rapier) (Dimforge) | Candidato a motor nativo de cuerpo rígido 2D/3D en Rust | **Apache-2.0** — compatible con GPL-3.0 | Roadmap (no integrado aún) |
| GeoGebra / Desmos / Mathematica / MATLAB / Manim | Comparativa de cobertura y UX de referencia | Marcas de sus dueños — comparativa, sin código | `plans/PLAN-SUPERACION-TOTAL.md` |

**Reglas legales que cumplimos:**

1. Nada copiado sin licencia compatible y atribución en este archivo.
2. `manim-physics` no se cita como código ni se vende: solo concepto +
   enlace, hasta que declaren licencia.
3. "PhET" y "GeoGebra" se nombran como marcas de sus dueños, sin
   implicar afiliación.
4. Si integrás una dependencia nueva, agregala a esta tabla con licencia
   y qué se tomó — el crédito es parte del gate, no un extra.
5. Todo lo que Grafito enseña de física (fórmulas, recetas, ejercicios)
   es derivación propia verificada con tests; las constantes (G, h, σ, b,
   λc) son hechos de la naturaleza con valor NIST/CODATA.

---

## Contribuir

1. Hacé un fork
2. Creá tu rama (`git checkout -b feature/nueva-funcionalidad`)
3. Escribí tests, ejecutá `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace`
4. Commiteá con [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`)
5. Abrí un Pull Request

Ver [AGENTS.md](AGENTS.md), [CONTRIBUTING.md](.github/CONTRIBUTING.md) y [SECURITY.md](.github/SECURITY.md) para más detalles.

---

## Licencia

GNU General Public License v3.0 o posterior. Ver [LICENSE](LICENSE).

```
Grafito — Geometría interactiva, álgebra y cálculo acelerados por GPU
Copyright (C) 2025-2026  Diez111

Este programa es software libre: puede redistribuirlo y/o modificarlo
bajo los términos de la GNU General Public License publicada por
la Free Software Foundation, versión 3 o (a su elección) cualquier
versión posterior.
```
