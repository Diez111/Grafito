# Física animada con recetas (sin video pesado)

Política: GIF nativo 12fps/64f/5MiB o **receta de comandos** re-ejecutable
(ver `docs/anim-liviano.md`). Estas recetas usan solo comandos `Dinámica`
verificados por tests (`oleada2::fisica_visual_comandos_crean_objetos`).
Copiar/pegar en la barra de entrada, en orden.

## 1. Tiro oblicuo completo (trayectoria + vectores + E)

```text
TrayectoriaProyectil[10, 45, 0]
CampoCargas[0, 0, 0]  ← NO: esto da error honesto (carga nula). Omitir.
VectoresProyectil[10, 45, 0, 0]
VectoresProyectil[10, 45, 0, 0.36]
VectoresProyectil[10, 45, 0, 0.72]
VectoresProyectil[10, 45, 0, 1.08]
VectoresProyectil[10, 45, 0, 1.40]
```

Vuelo = 1.44 s (v0=10, 45°). Cada `VectoresProyectil` agrega P + v + a +
at + an en ese instante (leyenda del mensaje: verde=v, rojo=a=g,
naranja=at, azul=an). Avanzar t = animación manual; exportar a GIF con el
exportador nativo.

## 2. Campo eléctrico + equipotenciales (dipolo)

```text
CampoCargas[1, -1, 0, -1, 1, 0]
EquipotencialCargas[1, -1, 0, -1, 1, 0]
```

El campo pinta color=|E| (viridis log); las 8 equipotenciales salen V⊥E.
Probar también `CampoCargas[1, 0, 0]` + `EquipotencialCargas[1, 0, 0]`
(círculos r=0.5/1/2/4).

## 3. MAS + RC (funciones vivas)

```text
OsciladorArmonico[1, 2, 0]
CargaCapacitor[5, 1000, 0.001]
```

El mensaje informa T=3.14 s y τ=1.000 s. Son `Function` normales:
se les aplica tangente, raíz, integral como a cualquier función.

## 4. Campo de direcciones (EDO) + trayectoria

```text
VectorField2D[-y, x]
```

con `color_by_magnitude` activado por defecto + streamlines RK4
automáticas. Para EDO `y'=f(x,y)`, el campo `(1, f)` dibuja el campo de
direcciones y las streamlines son las soluciones.

## 5. Magnetismo (hilo recto, Biot-Savart)

```text
CampoHilo[1, 0, 0]
CampoHilo[-1, 2, 0]
```

B azimutal 1/r con color=|B|: el primer hilo (I>0) gira antihorario,
el segundo (I<0) horario. Dos hilos opuestos muestran la cancelación
entre ellos. Regla de la mano derecha visible sin dibujarla.

## 6. Faraday-Lenz (espira rotante)

```text
FemInducida[100, 0.5, 0.02, 60]
```

fem(t)=60·sin(60·t): la espira de 100 vueltas genera 60 V pico.
Combinar con `OsciladorArmonico` y comparar fase: la FEM va como seno
mientras el flujo va como coseno (derivada a la vista). Lenz queda en el
signo: invertir B invierte la curva.

## 7. Elasticidad (Hooke visual)

```text
BarraHooke[10, 50, 100]
BarraHooke[10, 50, -50]
```

Gris = original (L0=10), verde = tracción (L=12), roja = compresión.
ΔL=F/k a la vista, sin tensor: la barra no miente porque los dos
segmentos comparten el punto fijo en 0. Combinar con
`OsciladorArmonico` para ver de dónde sale la ω=√(k/m) del resorte.

## 8. Conducción térmica (Fourier 1D)

```text
ConduccionTermica[100, 0, 2, 50]
```

T(x)=100−50x en [0,2], q=2500 W/m² hacia +x. El signo del mensaje
da el sentido del calor; T1=T2 informa equilibrio (q=0). Es una
`Function` normal: su pendiente ES el gradiente térmico (−q/k).
Puente con `edo-calor-onda`: esto es el estado estacionario al que
tiende la serie de senos.

## 9. Gravitación (campo + escape)

```text
CampoGravitatorio[1, -1, 0, 1, 1, 0]
VelocidadEscape[5.972e24, 6.371e6]
```

El campo es el hermano atractivo de Coulomb (mismo 1/r², signo
opuesto): comparar `CampoCargas[1,-1,0,1,1,0]` vs el gravitatorio
muestra repulsión vs atracción en el punto medio. La Tierra da
11186 m/s = 11.186 km/s (segunda cósmica real).

## 10. Fluidos (Venturi dibujado)

```text
TuboVenturi[4, 1, 1, 100000, 1000]
```

Anchos ∝ A, flechas ∝ v (v2=4×v1 a la vista), mensaje con
p2=92500 Pa (Δp=−7500). Continuidad y Bernoulli en un dibujo:
donde se angosta, corre más rápido y la presión cae.

## 11. Óptica (Snell dibujado + TIR)

```text
RayoRefraccion[1, 1.5, 30]
RayoRefraccion[1.5, 1, 60]
```

Amarillo = incidente, verde = refractado (θ2=19.47°), naranja =
reflejado en TIR (θc=41.8°). La ley se lee en el dibujo: el rayo se
acerca a la normal al entrar al vidrio; volviendo, más allá del
ángulo crítico no hay verde sino naranja.

## 12. Ondas (modo estacionario)

```text
OndaEstacionaria[2, 3, 1]
```

Envolvente 2·sin(3πx/2) + 4 puntos en los nodos (x=0, 2/3, 4/3, 2).
Los puntos no vibran y la curva sí los cruza siempre: condición de
cuerda fija a la vista. Cambiar n=1..4 muestra λ=2L/n encajando.

## 13. Termodinámica (isoterma dibujada)

```text
GasIdeal[1, 300]
GasIdeal[1, 600]
```

Dos hipérbolas p=2494/V y p=4988/V: a más T, más arriba.
Boyle-Mariotte sin tabla: a V fijo, duplicar T duplica p (se lee
en vertical). Es `Function`: el área bajo la curva entre V1 y V2
(integral) ES el trabajo isotérmico.

## 14. Circuitos AC (resonancia serie)

```text
ResonanciaRLC[10, 0.1, 0.0001]
```

Campana I(f) con pico en f₀=50.33 Hz, Q=3.16. Bajar R a 1
estrecha la campana (Q=31.6): la selectividad se VE antes de
calcularse. Puente con `CargaCapacitor`: el transitorio RC muere
y queda el estacionario sinusoidal.

## 15. Ondas en movimiento (Doppler)

```text
EfectoDoppler[100, 340, 440]
```

6 frentes circulares apiñados a la derecha, estirados a la
izquierda: 623.3 Hz de frente, 340.0 Hz detrás (la=440 Hz).
La asimetría ES la fórmula f'=fc/(c∓v) dibujada. Subir v→c
muestra la pared sónica; v=c da error honesto (no hay f').

## 16. Óptica ondulatoria (Young)

```text
DobleRendija[0.0000005, 0.0001, 1]
```

Franjas cos² con Δy=5 mm: máximos donde la diferencia de camino
es kλ, ceros a mitad de camino. Achicar d ensancha Δy (se lee en
la curva). Honesto: sin envolvente de difracción de rendija
finita; el mensaje lo dice.

## 17. Relatividad (Minkowski)

```text
DiagramaMinkowski[0.6, 1]
```

Cono amarillo ct=±x + línea verde del observador a 0.6c con
γ=1.250. A más v, más pegada a la luz: el límite c se VE antes
de postularse. v=c da error honesto (nada masivo llega a c).

## 18. Nuclear (misma EDO, otra física)

```text
Desintegracion[1000, 0.1]
CargaCapacitor[5, 1000, 0.001]
```

N(t)=1000e^(−0.1t) con t½=6.931 junto a la carga del RC: la
misma exponencial gobierna núcleos y capacitores. La matemática
no sabe de física; el contexto (núcleos vs electrones) lo pone el
mensaje. En t=t½ queda 1/2, en 2t½ queda 1/4.

## 19. Cuántica (umbral fotoeléctrico)

```text
EfectoFotoelectrico[5.66e14, 3.43e-19]
EfectoFotoelectrico[1e14, 3.43e-19]
```

Luz verde sobre cesio: K_max=0.20 eV; luz roja: sin emisión
(error honesto, no K negativa). La intensidad no importa, solo
f: la segunda corrida lo demuestra cambiando f, no el número
de fotones.

## 20. Mecánica clásica (péndulo honesto)

```text
Pendulo[1, 30]
Pendulo[1, 5]
```

T=2.042 s (+1.7% sobre el chico) vs T≈2.007 s a 5°: la serie
corrige lo que el libro esconde. Más allá de 60° el comando se
niega (serie fuera de rango) en vez de mentir.

## 21. Termodinámica (techo de Carnot)

```text
CicloCarnot[300, 600]
GasIdeal[1, 300]
```

Banda entre p=2494/V y p=4988/V con η=50% techo: ninguna
máquina entre 300 y 600 K supera el 50%. El área del ciclo real
(que el estudiante dibuja con integrales sobre estas curvas)
siempre queda bajo ese techo.

## 22. Óptica geométrica (lente real)

```text
LenteDelgada[2, 6, 1]
```

Lente + objeto verde + imagen roja invertida (di=3, hi=−0.5) +
rayo paralelo y central que se cruzan justo en la punta de la
imagen: la construcción cierra. do<f se niega (ahí la imagen es
virtual y este diagrama no la cubre).

## 23. Aire (el vacío era mentira útil)

```text
CaidaRozamiento[1, 0.5, 0]
CaidaRozamiento[1, 0.5, 20]
```

vt=19.60 m/s asíntota: soltar (v0=0) y lanzar arriba (v0=20)
terminan igual. Comparar con `TrayectoriaProyectil` (vacío): el
aire achata y frena; la parábola era el caso c→0.

## 24. Cuántica II (el fotón choca)

```text
EfectoCompton[90]
EfectoFotoelectrico[5.66e14, 3.43e-19]
```

Δλ=2.426e-12 m a 90° junto al fotoeléctrico: en uno el fotón
saca electrones, en el otro rebota y sale más rojo. Dos caras
del mismo cuanto.

## 25. Gravedad II (pozos, no colinas)

```text
PotencialGravitatorio[1, 0, 0]
CampoGravitatorio[1, 0, 0]
```

4 curvas V<0 junto al campo atractivo: el potencial eléctrico
tenía ±, el gravitatorio solo −. La comparación dibujada enseña
el signo antes que la fórmula.

## 26. Óptica geométrica II (el espejo pliega)

```text
EspejoEsferico[2, 6, 1]
LenteDelgada[2, 6, 1]
```

Mismos números (di=3, hi=−0.5), distinto lado: la lente manda
la imagen al otro lado, el espejo la deja en casa. Una fórmula,
dos geometrías.

## 27. Polarización (el filtro que apaga)

```text
LeyMalus[100, 60]
LeyMalus[100, 90]
```

I=25 a 60°, I=0 a 90°: dos polarizadores cruzados apagan la
luz. La curva cos² lo muestra para todo ángulo; el mensaje lo
dice para el pedido.

## 28. Radiación (el color dice la temperatura)

```text
LeyWien[5800]
LeyWien[300]
```

λ_max=500 nm (Sol verde-amarillo) vs 9.7 µm (cuerpo infrarrojo):
la temperatura se lee en el color. Puente con `CicloCarnot`: la
misma T que limita máquinas pinta espectros.

## 29. Radiación II (cuánta energía)

```text
LeyWien[5800]
PotenciaStefan[5800, 6.09e18]
```

Wien dice el color (500 nm), Stefan la potencia (3.9e26 W): el
Sol entero en dos líneas. La T⁴ explica por qué un poco más
caliente es muchísimo más brillante.

## 30. Fluidos II (el peso del agua)

```text
PresionHidrostatica[1000, 10]
TuboVenturi[4, 1, 1, 1e5, 1000]
```

98000 Pa (0.97 atm) a 10 m junto a Venturi: el agua quieta
pesa, el agua en movimiento acelera y baja su presión. Reposo
y movimiento, las dos presiones.

## 31. Fluidos III (flota o se hunde)

```text
EmpujeArquimedes[1000, 1, 917]
EmpujeArquimedes[1000, 1, 7800]
```

Hielo FLOTA, hierro SE HUNDE: el veredicto en mayúsculas
antes que el número. La densidad decide, no el peso.

## 32. Fluidos IV (el número que ordena el caos)

```text
NumeroReynolds[1000, 0.1, 0.01, 0.001]
NumeroReynolds[1000, 5, 0.01, 0.001]
```

LAMINAR a 0.1 m/s, TURBULENTO a 5 m/s en el mismo caño: la
velocidad cruza el umbral. Cierra fluidos: Venturi mueve,
hidrostática pesa, Arquímedes empuja, Reynolds ordena.

## 33. Materiales III (el puente respira)

```text
DilatacionLineal[100, 0.000012, 40]
BarraHooke[10, 50, 100]
```

100 m de acero crecen 4.8 cm con 40 K: por eso los puentes
llevan juntas. Junto a Hooke y Fourier cierra materiales:
la fuerza deforma, el calor conduce, la temperatura alarga.

## 34. Mecánica VII (la rampa que todo lo explica)

```text
PlanoInclinado[2, 30]
VectoresProyectil[10, 45, 0, 0.5]
```

a=4.900 m/s² con P roja, P∥ naranja y N azul: la
descomposición que el pizarrón promete y el canvas cumple.
Junto al tiro oblicuo cierra cinemática + dinámica.

## 35. Circuitos I (la ley que todo lo enciende)

```text
LeyOhm[12, 4]
LeyOhm[2, 220]
```

3 A y 36 W en la lámpara, 9 mA en el LED: la misma ley en
dos escalas. Puerta de entrada a circuitos junto a
`CargaCapacitor` y `ResonanciaRLC`.

## 36. Calorimetría (la pava eléctrica)

```text
CalorEspecifico[1, 4186, 80]
ConduccionTermica[100, 0, 2, 50]
```

335 kJ para 1 L a 80 K: el número que explica la factura
de luz. Junto a Fourier cierra el calor: cuánto entra
(calorimetría) y cómo viaja (conducción).

## 37. Electrostática II (el signo decide)

```text
FuerzaCoulomb[0.000001, 0.000001, 1]
CampoCargas[1, -1, 0, 1, 1, 0]
```

8.988 mN y REPULSIÓN en mayúsculas antes que el número:
el signo antes que la fórmula. El campo dibuja, la fuerza
cuantifica: E y F por fin juntos.

## 38. Enfriamiento (el café y el capacitor)

```text
LeyEnfriamiento[90, 20, 0.1]
CargaCapacitor[12, 1000, 0.001]
Desintegracion[1000, 0.1]
```

Tres fenómenos, una sola EDO: el café se enfría, el
capacitor se carga y el núcleo decae con la misma
matemática. La curva vive: derivá, integrá, tangents.

## 39. Cuántica III (todos somos ondas)

```text
OndaDeBroglie[9.11e-31, 1000000]
OndaDeBroglie[70, 1]
EfectoFotoelectrico[5.66e14, 3.43e-19]
```

El electrón difracta (0.73 nm), vos no (1e-36 m): la
dualidad tiene escala y el número la muestra. Cierra
cuántica: el fotón patea, el electrón ondula.

## 40. Energía (lo que se conserva)

```text
EnergiaMecanica[2, 3, 5]
PlanoInclinado[2, 30]
```

Ec=9, Ep=98, Em=107: la cuenta que el plano promete
(a=g·sinθ) y la energía verifica. Sin rozamiento en
ambos: la conservación se ve, no se declama.

## 41. Magnetismo III (la mano manda)

```text
FuerzaLorentz[1.6e-19, 1000000, 1]
CampoHilo[5, 0, 0]
```

1.6e-13 N con mano derecha en el mensaje: el protón gira,
el electrón al revés, el neutrón sigue recto. El hilo
dibuja B, Lorentz pone la fuerza: magnetismo completo.

## 42. Cinemática I (la recta antes que la parábola)

```text
MovimientoAcelerado[0, 10, 2]
MovimientoAcelerado[5, 3, 0]
CaidaRozamiento[1, 0.5, 0]
```

MRUV, MRU (a=0, sin comando aparte) y Stokes: la recta
antes que la parábola del tiro. En vacío vale siempre;
con aire, la terminal manda.

## 43. Gravitación III (quedarse vs escapar)

```text
OrbitaCircular[5.972e24, 6.771e6]
VelocidadEscape[5.972e24, 6.371e6]
```

7672 m/s para quedarse, 11186 m/s para irse: el factor √2
entre órbita y escape se ve comparando los dos comandos.
La ISS da 92 minutos de período.

## 44. Acústica (el logaritmo que se escucha)

```text
NivelSonoro[0.000001]
EfectoDoppler[100, 340, 440]
```

60 dB de conversación junto a los frentes desplazados:
el oído mide logaritmos y el movimiento corre frecuencias.
10 motosierras no son 10× una: el test lo blinda.

## 45. Química I (el otro logaritmo)

```text
PH[0.01]
NivelSonoro[0.000001]
```

pH 2 ácido junto a 60 dB de conversación: dos logaritmos,
dos sentidos. El −log₁₀ une química y ondas en una línea.

## 46. Fluidos II (la densidad decide antes)

```text
Densidad[917, 1]
EmpujeArquimedes[1000, 1, 917]
```

917 kg/m³ flota antes de dibujar nada: la densidad decide,
Arquímedes verifica. El hielo y el hierro en dos líneas.

## 47. Energía II (el trabajo entra, la energía queda)

```text
Trabajo[10, 5]
EnergiaMecanica[2, 3, 5]
```

50 J de trabajo junto a 107 J de mecánica: el trabajo
entra, la energía queda. Negativo frena, no falla.

## 48. Energía III (qué tan rápido)

```text
PotenciaMecanica[100, 4]
Trabajo[100, 4]
```

25 W junto a 100 J: la potencia dice qué tan rápido,
el trabajo dice cuánto. Tríptico completo en dos líneas.

## 49. Gravitación III (la constante que verifica)

```text
TerceraKepler[5.972e24]
OrbitaCircular[5.972e24, 6.771e6]
```

k = 9.904e-14 junto a la ISS a 7672 m/s: si tu órbita
no cumple k, no es kepleriana. La constante verifica.

## 50. Mecánica II (el plano deja de ser ideal)

```text
PlanoInclinado[2, 30]
FuerzaRozamiento[0.3, 16.97]
```

5.091 N de roce junto al plano ideal: con μ el bloque
puede no deslizar. El ideal promete, el roce corrige.

## 51. Electromagnetismo II (el dipolo real es 3D)

```text
CampoCargas3D[1, -1, 0, 0, -1, 1, 0, 0]
CampoCargas[1, -1, 0, -1, 1, 0]
```

E(0,0,0) = 2 en x junto al dipolo plano: el corte z=0
engaña, el volumen no. Esferas roja/azul = signo.

## 52. Gravitación IV (el plano que el número no ve)

```text
OrbitaCircular[5.972e24, 6.771e6]
SistemaOrbital[5.972e24, 6.771e6, 51.6]
TerceraKepler[5.972e24]
```

v = 7672 m/s y T = 5544 s valen para cualquier inclinación;
el dibujo normalizado (R = 5) muestra lo que cambia: el plano
de la ISS a 51.6°. Y k = 9.904e-14 verifica que la órbita
cumple Kepler.

## 53. Ondas II (la onda avanza, no vibra en el lugar)

```text
OndaEstacionaria[4, 2, 1]
VelocidadOnda[2, 3]
```

La cuerda fija tiene nodos que no se mueven; la viajera
sí: la foto gris (t = T/4) va un cuarto de λ a la derecha.
v = λf = 6 m/s pone el número al movimiento.

## 54. Mecánica III (lo que el choque conserva)

```text
CantidadMovimiento[1000, 20]
Trabajo[50, 4]
EnergiaMecanica[1000, 20, 0]
```

p = 20000 kg·m/s no se pierde en el choque interno: se
reparte. El trabajo entra (200 J), la energía queda, y el
momento dice dónde va cada parte.

## 55. Mecánica IV (curva sin frenar)

```text
 MovimientoAcelerado[0, 20, 0]
AceleracionCentripeta[20, 50]
```

El MRU va derecho sin acelerar; en la curva de 50 m a
20 m/s aparecen 8 m/s² hacia el centro. Dobla, no frena:
la rapidez sigue 20, la dirección no.

## 56. Gravitación V (el número detrás de la órbita)

```text
LeyGravitacion[5.972e24, 7.342e22, 3.844e8]
TerceraKepler[5.972e24]
SistemaOrbital[5.972e24, 6.771e6, 51.6]
```

F = 1.980e20 N entre Tierra y Luna con G real: la fuerza
que el campo dibuja y la órbita equilibra. k verifica que
es kepleriana; el dibujo inclinado muestra el plano.

## 57. Cuántica IV (atar y liberar)

```text
EnergiaBohr[1]
EnergiaBohr[2]
OndaDeBroglie[9.11e-31, 1e6]
EfectoFotoelectrico[1e15, 2.3e-19]
```

n = 1 ata con −13.6 eV; n = 2 afloja a −3.4 eV; n → ∞
libera en 0. Entre esos dos extremos vive todo el espectro
del H: el electrón que difracta y el fotón que arranca.

## 58. Electromagnetismo VI (la ley que opone)

```text
LeyFaraday[100, 0.01, 0.5]
FemInducida[100, 0.5, 0.01, 6.28]
```

N = 100 vueltas, ΔΦ = 0.01 Wb en medio segundo: fem = −2 V.
El signo menos es Lenz — el flujo que sube induce corriente
que lo baja. La ley discreta y la espira rotando dicen lo mismo.

## 59. Electromagnetismo VII (la carga manda el flujo)

```text
FlujoGauss[1e-9]
```

Q = 1 nC encerrado: Φ = 112.941 N·m²/C en todas direcciones.
El flujo no se crea ni se destruye: sale de la carga, y una caja
vacía (Q = 0) da cero honesto. Hermano de Coulomb con otra cara:
Coulomb pone la fuerza sobre otra carga, Gauss cuenta el campo total
que la carga encerrada irradia.

## 60. Termodinámica IX (fundir sin calentar)

```text
CalorLatente[1, 334000]
```

1 kg de hielo a 0 °C traga 334 kJ sin subir un grado: toda la
energía rompe puentes de hidrógeno en vez de agitar moléculas.
La meseta que el termómetro no ve — y que CalorEspecifico no
explica (ese calienta, este transforma). Solidificar devuelve
lo mismo: L > 0 siempre, el signo lo pone el sentido del cambio.

## 61. Mecánica V (el que llega frena en seco)

```text
ChoqueElastico[1, 3, 1, 0]
```

Masas iguales: intercambian velocidades — el de atrás frena en
seco (v₁′ = 0) y el de adelante sale con todo (v₂′ = 3 m/s).
Es p y Ec conservadas sin dibujar un solo vector. Y pesado
contra liviano (`ChoqueElastico[10, 2, 1, 0]`): el liviano
sale a 3.636 m/s, casi el doble — el camión ni se inmuta.

## 62. Ondas IV (el aire caliente apura la voz)

```text
VelocidadSonido[20]
```

343.215 m/s a 20 °C: el número que todo libro cita, sin tabla.
La raíz cuadrada con T absoluta dice por qué el trueno de
verano llega antes que el de invierno. Bajo el cero absoluto
no hay aire que vibre: T ≥ −273.15 °C o error honesto.

## 63. Mecánica VI (45° rinde más)

```text
AlcanceProyectil[20, 45]
```

20 m/s a 45° caen a 40.775 m: el sin(2θ) en una línea, sin
rozamiento y sin dibujar la parábola. A 30° da 35.312 m —
la simetría 30°/60° se verifica con dos enteros. Hermano
numérico de TrayectoriaProyectil (ese dibuja, este responde).

## 64. Electromagnetismo VIII (lo que el capacitor guarda)

```text
EnergiaCapacitor[0.001, 100]
```

1 mF a 100 V guardan 5 J en silencio: el flash de la cámara
vieja en un número. El signo de U no importa (va al cuadrado)
y C > 0 siempre. Hermano de CargaCapacitor (ese cuenta la
carga, este la energía).
