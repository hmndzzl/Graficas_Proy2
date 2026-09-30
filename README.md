# Graficas_Proy2
Diorama con Raytracing

## Isla inicial reproducible

La escena incluye una isla amplia, cabaña, lago refractivo, árboles, portal al
Nether de obsidiana y una cueva abierta con diamante, oro y lava. La variación
del terreno se controla con una seed: el mismo número siempre crea la misma
isla.

```bash
cargo run -- --seed 424242
```

También se acepta `--seed=424242`. Sin argumento se usa `20260930`. Las teclas
`6` y `7` seleccionan obsidiana y portal en la barra rápida.

## Portal y nuevo bioma

Apunta al centro morado de cualquiera de los portales y presiona `E` para
viajar entre escenas. El destino es un **Bosque Carmesí del Nether** separado:
tiene netherrack, basalto, blackstone, soul sand, hongos carmesíes, glowstone,
lava, iluminación volcánica y niebla roja propia. El portal del Nether permite
regresar a la isla inicial.

Las texturas se toman directamente del atlas: portal `(14, 0)`, obsidiana
`(5, 2)`, piedra infernal `(7, 6)`, soul sand `(8, 6)` y glowstone `(9, 6)`.
Estas coordenadas son visuales (origen arriba-izquierda); el helper compensa la
inversión vertical sólo para estos bloques nuevos, sin alterar las texturas
previamente mapeadas.
