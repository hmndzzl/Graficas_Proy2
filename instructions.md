# Proyecto: Diorama Raytracer de Minecraft en Rust
**Destino:** Antigravity IDE
**Objetivo:** Desarrollar un motor de Raytracing en software (CPU) utilizando Rust para renderizar un diorama interactivo estilo Minecraft. El proyecto debe cumplir con una rúbrica estricta de evaluación y estar optimizado para una MacBook Pro M2.

---

## 1. Configuración del Proyecto y Dependencias
El proyecto está restringido a no usar librerías externas ajenas al ecosistema de Rust, pero se permiten *crates* estándar para matemáticas, ventanas y manejo de imágenes.

**`Cargo.toml` recomendado:**
```toml
[dependencies]
nalgebra = "0.32"       # Álgebra lineal (Vectores, Matrices)
minifb = "0.25"         # Manejo de ventanas (Window manager) y buffer de píxeles
image = "0.24"          # Carga y manipulación del Texture Atlas y texturas del Skybox
rayon = "1.7"           # Opcional/Recomendado: Paralelización para renderizado rápido en CPU (aprovechar los núcleos del M2)
```

---

## 2. Especificaciones de la Ventana y Rendimiento
*   **Resolución Base:** `1300 x 800` píxeles.
*   **Manejo de Rendimiento:** Dado que el raytracing por CPU a esta resolución es pesado, se debe implementar **Renderizado Progresivo** o **Resolución Dinámica**:
    *   *Mientras la cámara se mueve:* Renderizar a una resolución menor (ej. lanzando 1 rayo cada bloque de 4x4 píxeles o limitando la profundidad de rebotes) para mantener interactividad.
    *   *Cuando la cámara se detiene:* Renderizar con resolución completa, Anti-Aliasing (múltiples rayos por píxel) y máxima profundidad de rebotes.

---

## 3. Rúbrica de Evaluación y Tareas Críticas

### A. Cámara Interactiva [20 Puntos]
*   **Requisito:** Implementar rotación en el diorama y acercamiento/alejamiento (zoom).
*   **Implementación:** Configurar una **Cámara Orbital** (Arcball).
    *   **Rotación:** Controlada por las flechas del teclado o el arrastre del mouse. Actualizar el origen y la dirección de la cámara usando trigonometría esférica alrededor del centro de la isla `(0,0,0)`.
    *   **Zoom:** Controlado por el *scroll* del mouse o las teclas `W`/`S`. Modificar el radio de la órbita (acercando el `origin` al centro).

### B. Sistema de Materiales [25 Puntos - 5 por cada material distinto]
Se requiere un sistema robusto. Cada bloque debe ser un AABB (Axis-Aligned Bounding Box) con un material asignado. 
**Estructura del Material en Rust (ejemplo lógico):**
```rust
struct Material {
    albedo_color: Vec3,
    uv_coordinates: (Vec2, Vec2), // Para mapear en el Texture Atlas (.png)
    specular: f32,
    transparency: f32, // 0.0 (opaco) a 1.0 (totalmente transparente)
    refraction_index: f32, // ior (Index of Refraction)
    reflectivity: f32, // 0.0 a 1.0
    emissive: Vec3, // Para la lava
}
```
**Lista de Materiales a Implementar (Supera el mínimo de 5):**
1.  **Agua / Cristal (Refracción):** Texturas semitransparentes, especular alto.
2.  **Oro (Reflexión):** Textura amarilla/metálica, reflectividad muy alta, albedo bajo, especular alto.
3.  **Madera (Tronco y Tablas) & Hojas:** Albedo desde textura, opacos, sin reflectividad, especular bajo (mate).
4.  **Césped & Piedra:** Opacos, rugosos, mate.
5.  **Mineral de Diamante (Diamond Ore):** Mezcla de opaco con pequeños destellos (podría tener un mapa especular o un especular medio).
6.  **Lava:** Material *Emisivo*. No requiere luces externas para brillar, su albedo actúa como fuente de luz para los bloques cercanos (Radiosidad/Point light).

### C. Refracción [10 Puntos] y Reflexión [5 Puntos]
*   **Refracción:** Aplicar la Ley de Snell en los bloques de **Agua** (lago) y **Cristal** (ventanas de la cabaña). El rayo debe doblarse al atravesar el bloque. Asegurarse de manejar el rayo saliente.
*   **Reflexión:** Aplicar en bloques de **Oro** (colocados como tesoros en la mina o decoración en la cabaña). Calcular el rayo reflejado: `R = V - 2(V·N)N`.
*   *Nota de Raytracing:* Limitar la profundidad de recursión (ej. `max_depth = 3 o 4`) para reflejos/refracciones para no saturar el CPU.

### D. Skybox Dinámico [20 Puntos]
*   **Implementación:** Cuando un rayo no intersecta con ningún cubo de la isla, no debe devolver negro, sino muestrear una textura de Skybox.
*   **Transición Día/Noche:** 
    *   Mapear una textura de día (cielo azul, nubes cuadradas) y una de noche (estrellas, luna).
    *   Utilizar una variable de tiempo (`t`) o asociarlo a la rotación de la cámara/mundo. Hacer un `lerp` (interpolación lineal) entre los colores de la textura de día y de noche basado en la altura del "sol" simulado.

### E. Composición del Diorama (Complejidad y Atractivo Visual) [50 Puntos]
Para asegurar los 50 puntos subjetivos, la escena debe ser espectacular:
*   **Estructura de la Isla:** Una semiesfera o cono invertido de bloques de piedra y tierra flotando en el centro del espacio.
*   **Superficie:**
    *   Un pequeño lago (Agua - refracción).
    *   Una cabaña hecha de tablas de madera, pilares de troncos y ventanas de cristal (Refracción).
    *   Árboles (Troncos y hojas).
*   **Subterráneo (La Mina):**
    *   Hacer un corte transversal (como un pastel cortado) para que la cámara pueda ver el interior al rotar.
    *   Dentro de la mina: Piedra, vetas de **Diamond Ore**, bloques de **Oro** (Reflexión) y un foso de **Lava** (Emisivo).
    *   *Atractivo Visual:* La lava debe iluminar las paredes de la mina con un tono naranja/rojizo intenso, contrastando fuertemente si la cámara rota hacia la zona oscura y el skybox está en modo "Noche".

---

## 4. Pasos para Antigravity IDE
1.  **Módulos Base:** Crear módulos de vectores, rayos y esferas/AABBs (`geom.rs`, `ray.rs`).
2.  **Motor de Intersecciones:** Implementar el algoritmo de intersección de Rayo contra AABB (Slab method). Optimizar agrupando los bloques en un AABB general (Bounding Volume Hierarchy - BVH simplificado) si el rendimiento cae.
3.  **Carga de Texturas:** Leer el PNG (Texture Atlas) con el crate `image` y crear una función que mapee una coordenada `(u, v)` de una cara de un bloque a su pixel correspondiente en el Atlas.
4.  **Sistema de Sombreado (Shading):** Implementar modelo de Phong o Blinn-Phong para la luz directa. Implementar las llamadas recursivas para reflexión y refracción.
5.  **Loop Principal (`minifb`):** 
    *   Inicializar la ventana a 1300x800.
    *   Capturar eventos de teclado/mouse.
    *   Actualizar posición de cámara y luz del skybox.
    *   Ejecutar el render (lanzar rayos) frame a frame y actualizar el buffer de la ventana.
6.  **Construcción de la Escena:** Instanciar la lista de cubos con sus coordenadas 3D y materiales asignados construyendo la isla, la cabaña y la cueva.

¡Manos a la obra, priorizando las físicas de luz (reflexión/refracción) y la manipulación de la cámara para asegurar todos los puntos técnicos!
