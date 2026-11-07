# Que es un shader, explicado con este laboratorio

## 1. Blender, renderer y shader hacen trabajos diferentes

**Blender** permite crear un modelo 3D, por ejemplo una esfera. Ese modelo es una
coleccion de vertices y caras; por si solo no define el color final de cada pixel.

Un **renderer** convierte un modelo 3D en una imagen. En este laboratorio usamos
un software renderer: nuestro programa de Rust dibuja los triangulos en CPU.
No delegamos ese dibujo al motor de Blender. En esta version generamos la esfera
por codigo, asi que tampoco necesitamos exportar un modelo desde Blender.

Un **shader** calcula propiedades de lo que se dibuja. En particular, nuestro
fragment shader recibe informacion de un punto de la superficie y devuelve un
color. Aunque en muchos motores los shaders se ejecutan en GPU, aqui son funciones
normales de Rust ejecutadas por la CPU.

## 2. La esfera es la misma; cambia la receta de color

Imagina que tienes una esfera blanca. Para cada pixel que pertenece a ella,
preguntas: "en esta posicion, que color corresponde?".

Una receta sencilla para pintar franjas seria:

```rust
let bandas = (posicion.y * 28.0).sin() * 0.5 + 0.5;
let color = mix(azul, crema, bandas);
```

Es un ejemplo conceptual: los nombres `posicion`, `azul` y `crema` representan
datos que tendrias que definir. La funcion real esta en `gas()` de `src/shaders.rs`.

- `posicion.y` indica la altura del punto dentro de la esfera.
- Multiplicar por `28.0` hace que el seno oscile muchas veces entre polo y polo.
- `sin()` produce valores entre -1 y 1; multiplicar por 0.5 y sumar 0.5 los lleva a 0..1.
- `mix(azul, crema, bandas)` mezcla los colores segun ese valor.

Todos los puntos a alturas parecidas reciben colores parecidos: aparecen franjas.
No hemos pegado una foto de Jupiter; hemos calculado los colores.

## 3. Que recibe nuestro fragment shader

En `src/shaders.rs`, la estructura `Fragment` tiene dos datos:

```rust
pub struct Fragment {
    pub position: Vec3,
    pub normal: Vec3,
}
```

**`position`** es la posicion en la esfera original, antes de girarla. Sus componentes
estan aproximadamente entre -1 y 1. Usarla para los patrones hace que las manchas
giren junto con el planeta en lugar de quedarse pegadas a la pantalla.

**`normal`** indica hacia donde apunta la superficie en la escena. Arriba apunta
hacia arriba, a la derecha apunta hacia la derecha, etc. Se usa para comparar esa
direccion con la direccion de la luz y decidir que zonas se ven claras u oscuras.

El rasterizador calcula estos datos interpolando los vertices de cada triangulo
con coordenadas baricentricas. No necesitas cambiar ese paso para experimentar
con los colores.

La funcion `shade()` elige cual receta ejecutar. Su resultado es `Some(color)`
para pintar el fragmento, o `None` para descartarlo: los huecos del anillo usan esto.

## 4. Que significa una capa

La rubrica cuenta cambios de color. Una forma de organizar un shader es:

```text
color = roca oscura mezclada con minerales        ← capa 1
color = modificar el color con grano fino         ← capa 2
color = oscurecer crateres y aclarar sus bordes    ← capa 3
color = agregar fisuras brillantes                ← capa 4
devolver color
```

Son operaciones dentro de una funcion. No requieren cuatro objetos, cuatro
texturas ni cuatro archivos de shader. Tampoco basta con declarar cuatro colores:
cada capa debe producir un efecto visible sobre la superficie.

**Pruebalo:** ejecuta `cargo run --release`, pulsa `2` para ver Ignis y despues
pulsa `L`. El titulo indica cuantas capas estan activas. Cambia de una a cuatro
y observa que aporta cada paso. La iluminacion y el tinte del borde permanecen
para que la esfera conserve volumen mientras comparas sus capas.

## 5. Por que usamos ruido

Un seno produce un patron muy regular. Para que el plasma, la roca o las nubes
tengan variaciones organicas usamos **ruido procedural**: una funcion que asigna
valores suaves e irregulares a las posiciones del espacio.

No es un numero aleatorio nuevo cada cuadro. La misma posicion siempre devuelve
el mismo valor, por lo que el planeta no parpadea al dibujarlo de nuevo.

`noise.rs` calcula ruido 3D interpolando valores de una rejilla. `fbm()` combina
varias escalas de ese ruido: formas grandes, detalle mediano y detalle pequeno.
Esto ayuda a que las superficies tengan mas riqueza visual.

**Ojo:** las octavas de fBm no se cuentan automaticamente como las cuatro capas
de la rubrica. En este proyecto las cuatro capas son efectos distintos,
identificados con comentarios `CAPA 1` a `CAPA 4` en cada shader principal.

## 6. Luz, emision y volumen

En `lighting()` se usa el producto punto de la normal con la direccion de la luz:
cuanto mas hacia la luz apunta la superficie, mas clara se ve. Se mantiene un poco
de luz ambiente para que el lado oscuro no sea completamente negro.

La estrella no usa esa iluminacion externa porque representa un cuerpo que emite
luz. Las fisuras de Ignis se mezclan despues de iluminar la roca para conservar
su brillo. Esto cambia el color de esos pixeles; no ilumina otros objetos ni
produce un halo fuera de la silueta.

## 7. Donde cambiar cosas sin tocar todo el programa

Abre `src/shaders.rs` y prueba un cambio a la vez:

1. En `star()`, cambia uno de los colores `rgb(...)` para crear una estrella mas roja.
2. En `rocky()`, cambia los colores turquesa de las fracturas para hacer lava naranja.
3. En `gas()`, cambia `latitude * 28.0` a `latitude * 45.0` para tener mas bandas.
4. En `ring()`, cambia los intervalos que devuelven `None` para variar las divisiones.

Despues de editar, vuelve a ejecutar `cargo run --release`. Para actualizar las
imagenes del README, ejecuta `cargo run --release -- --export-all`.

## 8. Como explicarlo al presentar

"Mi programa genera una esfera de triangulos y la rasteriza por software. Para
cada fragmento interpola su posicion y normal, ejecuta un shader de color y
resuelve la visibilidad con un buffer de profundidad. Uso ruido 3D, senos,
distancias y mezclas de colores para producir cuatro efectos por cuerpo sin
cargar texturas. Los anillos y la luna son objetos separados."

Usa esta explicacion como guia y verifica que puedes identificar cada paso
en el codigo y mostrarlo con los controles.

## 9. Rotacion y traslacion en el sistema completo

Ahora hay **seis cuerpos principales**: una estrella y cinco planetas, mas una
luna. Los tres planetas adicionales son Pelagia (oceanos), Nivis (hielo) y
Coruscant (ciudad). Sus shaders tambien tienen cuatro efectos de color identificados.

Al ejecutar `cargo run --release` veras el sistema animado. Pulsa `0` para volver
a esa vista desde cualquier cuerpo; `1` a `6` abren los individuales. Espacio
pausa todo; `+` y `-` cambian la velocidad; `T` reinicia el tiempo. Puedes pausar,
cambiar de vista y examinar una posicion concreta sin perder el tiempo actual.

**Rotacion** significa que cambia el angulo del propio cuerpo:

```text
angulo = angulo_manual + tiempo * velocidad_de_rotacion
```

El shader sigue usando posiciones locales para sus patrones, por lo que los
continentes o las manchas giran junto con la esfera.

**Traslacion** significa que cambia el centro del cuerpo. Para una orbita circular
en un plano, se usan seno y coseno:

```text
fase = fase_inicial + 2*pi * tiempo / periodo
x = radio * cos(fase)
z = radio * sin(fase)
```

`Orbit::position()` en `scene.rs` inclina ese plano para poder ver las orbitas.
Cada planeta tiene su radio, periodo y fase inicial; el Sol mantiene su centro.
Los tamanos y tiempos son artisticos, no una simulacion gravitacional real.

La luna necesita dos movimientos combinados:

```text
centro_luna = centro_actual_del_planeta + desplazamiento_local_de_la_luna
```

Por eso sigue a Ignis aunque el planeta se encuentre al otro lado del Sol.
Su periodo de siete segundos es independiente de la rotacion de Ignis. La pausa
detiene el tiempo compartido y, con el, todas las rotaciones y traslaciones.

En la vista general, cada cuerpo recibe la direccion desde su centro hacia el
Sol para el calculo de luz. La estrella sigue usando su shader emisivo. En las
vistas individuales se conserva una luz fija para comparar los detalles.

## 10. Las placas de Dyson del Sol

La estrella conserva sus cuatro capas de color. Alrededor se dibujan 36 placas
como objetos independientes, con otras transformaciones y un shader de celdas
azules y marcos dorados. Los enlaces de energia son cintas estrechas con un color
emisivo calculado. Ninguno usa imagenes de textura.

Los colectores necesitan coordenadas locales sin normalizar: son superficies
planas, mientras que la esfera usa posiciones normalizadas. `Shader::is_sphere()`
permite al rasterizador distinguirlos. Las placas se dibujan por ambos lados y
el buffer de profundidad resuelve su posicion delante o detras del Sol.

## 11. Una ciudad que cubre un planeta

Coruscant usa la posicion local de la esfera para obtener longitud y latitud.
Sobre ellas, distintas rejillas generan distritos, edificios y ventanas. Las
avenidas se calculan por distancia a las lineas de una rejilla; las circunvalaciones
usan distancias a centros urbanos sobre la esfera.

Los distritos y edificios se iluminan normalmente. Las avenidas y ventanas se
mezclan despues de la iluminacion para conservar luz propia en la noche. La
apariencia procede de cuatro capas de color dentro del shader, no de una textura
ni de modelar cada edificio como geometria independiente.

