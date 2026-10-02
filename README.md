# Graficas_Proy2 - Minecraft Diorama Raytracer


Diorama estilo Minecraft renderizado 100% con raytracing por software en CPU, escrito en Rust. Este proyecto recrea una porción del universo de Minecraft con iluminación global, reflejos, refracciones y sombras precisas generadas a través de trazado de rayos.

<img width="1276" height="714" alt="image" src="https://github.com/user-attachments/assets/34741b21-6a97-4cd5-9eb8-1fa0544861c3" />

Link del video de demostración: https://youtu.be/zlFKC8qfUyQ

## Características Principales

*   **Tres Dimensiones Interconectadas:** Explora el **Overworld** (con su cabaña, bosque y lago), viaja al **Nether** (con su bosque carmesí, basalto y mares de lava), y aventúrate al **End** (con sus pilares de obsidiana, cristales y cielo procedimental).
*   **Teletransporte Inmersivo:** Acércate a los portales y presiona `E` para viajar sin tiempos de carga entre dimensiones. Al regresar al Overworld, aparecerás físicamente frente al portal del que saliste.
*   **Interacción de Bloques:** Sistema de inventario con hotbar (teclas `1` a `9`). Apunta con el ratón/centro de la pantalla y presiona `Space` para colocar bloques o `X` para romperlos.
*   **Raytracing Físicamente Basado:** 
    *   **Refracción y Transparencia:** El agua y el cristal refractan la luz de manera realista, mostrando la superficie gracias a su albedo ajustado.
    *   **Materiales Emisivos:** La lava, la glowstone y los cristales del End emiten su propia luz para iluminar la escena.
    *   **Texturas Animadas:** Los portales cuentan con un efecto visual de cascada mediante el desplazamiento continuo de coordenadas UV en tiempo real.
    *   **Sombras:** Sombras duras precisas generadas mediante raycasting al sol/fuentes de luz.
*   **Cielo Procedimental:** El cielo cambia dinámicamente. El Overworld tiene ciclo día/noche; el Nether una niebla volcánica rojiza; y el End cuenta con una textura nebulosa generada proceduralmente usando Fractal Brownian Motion (FBM) y ruido estático 3D.
*   **Audio Espacial y Música:** Integración con la librería `rodio` para reproducir música ambiental dinámica que cambia dependiendo de la dimensión en la que te encuentres.
*   **Optimizado con Rayon y Estructuras de Aceleración:** Renderizado multihilo para procesar los rayos en paralelo y uso de `VoxelGrid` para gestionar y renderizar más de **290,000 bloques concurrentes** a tasas de FPS jugables.

## Librerías Utilizadas

| Librería | Descripción e Importancia |
| :--- | :--- |
| `minifb` | Crítica para el renderizado del juego. Proporciona la creación de la ventana, entrada por teclado/mouse, y pinta directamente nuestro buffer de píxeles (`Framebuffer`) en pantalla. |
| `nalgebra-glm` | Facilita cálculos vectoriales como intersecciones espaciales, direcciones de rayos, normales y operaciones matriciales 3D. |
| `image` | Carga de forma eficiente el atlas de texturas (PNG/JPG) y decodifica los píxeles para aplicarlos sobre los bloques. |
| `rodio` | Sistema de reproducción de audio utilizado para cargar y reproducir las pistas de música ambiental en cada dimensión. |
| `rayon` | **Vital para el rendimiento.** Permite ejecutar el trazado de rayos en múltiples hilos simultáneamente (renderizado paralelo), aprovechando todos los núcleos del procesador. |
| `noise` | Motor principal para la generación procedural de las islas, elevaciones del terreno y las nubes / nebulosas del cielo. |

## Estructura del Proyecto


```text
Graficas_Proy2/
├── assets/         # Carpeta con el atlas de texturas (.png) y pistas de audio (.mp3, .ogg)
├── src/
│   ├── main.rs         # Ciclo principal del juego, cámara orbital, inventario y saltos entre dimensiones.
│   ├── render.rs       # Motor principal de Raytracing (sombras, reflejos, refracciones de Fresnel/Snell).
│   ├── diorama.rs      # Construcción procedural de las islas (Overworld, Nether, End) bloque a bloque.
│   ├── sky.rs          # Generación procedural de cielos (Perlin Noise, FBM, estrellas y niebla).
│   ├── cube.rs         # Matemáticas de intersección de AABB para los bloques del mundo.
│   ├── voxel_grid.rs   # Estructura de aceleración de voxeles para optimizar el cálculo de los rayos.
│   ├── camera.rs       # Lógica de la cámara orbital controlada por el jugador.
│   ├── audio.rs        # Envoltorio de `rodio` para cargar y transicionar pistas de música de fondo.
│   ├── texture.rs      # Abstracción para cargar imágenes y manejar el mapeo UV de las caras.
│   ├── material.rs     # Definición física de los bloques (albedo, transparencia, IOR, emisión).
│   └── framebuffer.rs  # Abstracción para el dibujo de píxeles puros a nivel de buffer.
└── Cargo.toml      # Configuración de Rust y dependencias
```

## Recursos de Audio y Música

Las pistas de audio utilizadas para ambientar el recorrido pertenecen a sus respectivos creadores (solo usadas con fines demostrativos):
*   **Música del Overworld:** Haunt Muskie / C418
*   **Música del Nether:** Dead Voxel / C418
*   **Música del End:** Haunt Muskie / C418

## Requisitos

- Rust (edición 2021) con `cargo`.
- Dependencias clave (manejadas automáticamente por cargo): `minifb` para la ventana, `rayon` para paralelismo, `nalgebra-glm` para vectores/matrices, `noise` para generación, y `rodio` para audio.

## Cómo Correr el Proyecto

```bash
cargo run --release
```
*(Es importante usar `--release` para que el renderizado por raytracing alcance tasas de FPS jugables).*

## Controles

| Tecla / Acción | Efecto |
|---|---|
| **W / S / A / D** | Orbitar la cámara alrededor de la isla / Zoom |
| **Q / E o Rueda Mouse** | Zoom in / Zoom out |
| **E** | Teletransportarse al estar cerca de un portal |
| **1 al 9** | Seleccionar bloque del inventario |
| **Espacio** | Colocar el bloque seleccionado |
| **X** | Romper el bloque al que se está mirando |
| **D** | Cambiar a luz de Día |
| **N** | Cambiar a luz de Noche |
| **T** | Avanzar el tiempo suavemente (Ciclo Día/Noche animado) |
| **Esc** | Salir del programa |

## Inventario (Hotbar)

Puedes seleccionar diferentes bloques para construir en el mundo pulsando las teclas del 1 al 9.

| Tecla | Bloque |
| :---: | :--- |
| **1** | Césped |
| **2** | Tablas de Madera |
| **3** | Piedra |
| **4** | Hojas |
| **5** | Cristal |
| **6** | Ladrillos de Piedra |
| **7** | Bloque de Oro |
| **8** | Cristal del End |
| **9** | Obsidiana |

## Estructura de Dimensiones

### Overworld
La isla principal cuenta con generación de terreno basada en Perlin Noise. Contiene una cabaña acogedora de madera, árboles de roble, un lago refractivo, y portales pre-construidos hacia el Nether (obsidiana) y el End (marcos del portal).

### Nether
Terreno generado con ruido plegado para crear un ambiente cavernoso. Materiales incluyen Netherrack, Basalto, Blackstone, Glowstone, Magma, Lava, y hongos carmesíes. Posee su propio altar y portal para regresar.

### End
Plataforma de End Stone flotando en un vacío de nebulosas moradas. Destacan los enormes pilares de obsidiana rematados con End Crystals emisivos y llamas, además del portal de retorno a casa.

## Materiales

Cada material tiene su propia textura (extraída del atlas de texturas) y parámetros de shading. Los albedos y las propiedades están balanceados para que funcionen armónicamente bajo el sistema de raytracing.

### Base (Overworld)

| Material | Textura | Albedo | Specular (coef/exp) | Transparencia | Reflectividad | IOR | Emision |
|---|---|---|---|---|---|---|---|
| Césped / Tierra / Arena | Extraída del atlas | Color base | 0.0 / 0 | 0 | 0 | 1.0 | - |
| Ladrillos de Piedra | Extraída del atlas | Gris | 0.1 / 10 | 0 | 0 | 1.0 | - |
| Madera / Tronco | Extraída del atlas | Marrón | 0.0 / 0 | 0 | 0 | 1.0 | - |
| Hojas | Extraída del atlas | Verde | 0.0 / 0 | 0 | 0 | 1.0 | - |
| Agua | Color fijo translúcido | Azul | 0.3 / 40 | 0.58 | 0.08 | 1.33 | - |
| Cristal | Color fijo translúcido | Blanco azulado | 0.4 / 50 | 0.65 | 0.1 | 1.5 | - |
| Minerales (Hierro/Oro/Diamante) | Extraída del atlas | Coloreado | 0.8 / 80 | 0 | 0.5 | 1.0 | - |
| Obsidiana | Extraída del atlas | Negro/Morado | 0.3 / 40 | 0 | 0 | 1.0 | - |

### Dimensiones Mágicas (Nether y End)

| Material | Textura | Albedo | Specular (coef/exp) | Transparencia | Reflectividad | IOR | Emision |
|---|---|---|---|---|---|---|---|
| Netherrack / End Stone | Extraída del atlas | Rojo / Amarillo | 0.0 / 0 | 0 | 0 | 1.0 | - |
| Basalto | Extraída del atlas | Gris oscuro | 0.0 / 0 | 0 | 0 | 1.0 | - |
| Magma | Extraída del atlas | Naranja / Rojo | 0.0 / 0 | 0 | 0 | 1.0 | Media |
| Glowstone / Lámpara End | Extraída del atlas | Amarillo / Blanco | 0.0 / 0 | 0 | 0 | 1.0 | Alta |
| Portal | Extraída del atlas | Morado | 0.1 / 10 | 0.5 | 0.1 | 1.1 | Luz violeta suave |
| Lava | Extraída del atlas | Naranja | 0.0 / 0 | 0 | 0 | 1.0 | Alta |

## Galería

### El Overworld
<img width="1275" height="713" alt="image" src="https://github.com/user-attachments/assets/43e211bd-83dc-41b3-afc4-2e48756e3ed8" />

<img width="1276" height="718" alt="image" src="https://github.com/user-attachments/assets/8ae28149-6893-4509-911b-12ae33bc314e" />

### El Nether
<img width="1270" height="710" alt="image" src="https://github.com/user-attachments/assets/5525a707-2ff2-4b35-858f-9e051430cad2" />

### El End
<img width="1274" height="718" alt="image" src="https://github.com/user-attachments/assets/d1f2c5d4-fa6c-4baf-bbb0-9cb965ddef82" />

