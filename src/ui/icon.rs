//! Icono de la ventana y de la bandeja.
//!
//! Se dibuja en memoria para no depender de un decodificador de imagenes en
//! runtime. El `.ico` del ejecutable se genera aparte y usa el mismo motivo.

use std::sync::Arc;

use eframe::egui;

const SIZE: u32 = 32;

/// Cuatro cuadros blancos sobre el azul de acento. Sin texto: el icono se lee
/// igual a 16 px que en la bandeja.
pub fn rgba() -> Vec<u8> {
    let mut pixels = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let index = ((y * SIZE + x) * 4) as usize;
            let inside = (6..=13).contains(&x) || (18..=25).contains(&x);
            let vertical = (6..=13).contains(&y) || (18..=25).contains(&y);
            let (r, g, b) = if inside && vertical {
                (255, 255, 255)
            } else {
                (37, 99, 235)
            };
            pixels[index] = r;
            pixels[index + 1] = g;
            pixels[index + 2] = b;
            pixels[index + 3] = 255;
        }
    }
    pixels
}

pub fn viewport_icon() -> Arc<egui::IconData> {
    Arc::new(egui::IconData {
        rgba: rgba(),
        width: SIZE,
        height: SIZE,
    })
}
