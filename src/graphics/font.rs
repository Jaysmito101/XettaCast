pub struct FontGlyph {
    pub id              : char,
    pub name            : String,
    pub advance         : (f32, f32),
    pub bearing         : (f32, f32),
    pub origin          : (f32, f32),
    pub bitmap          : Vec<u8>,
    pub width           : u32,
    pub height          : u32,
}

pub struct Font
{
    glyph_count             : u32,
    glyphs                  : std::collections::HashMap<char, FontGlyph>,
    
}

impl Font 
{
    
    pub async fn new(bytes: &[u8], scale: f32) -> Result<Self, String> {

        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).map_err(|e| format!("Failed to create font: {}", e))?;

        let mut glyphs = std::collections::HashMap::new();

            
        let mut max_size = (0.0f32, 0.0f32);
        let mut total_size = (0.0f32, 0.0f32);

        let keys = font.chars().keys().cloned().collect::<Vec<char>>();
        keys.into_iter().for_each(|id| {
            let (glyph, bitmap) = font.rasterize(id, scale);
            
            glyphs.insert(id as char, FontGlyph {
                id              : id as char,
                name            : format!("glyph{}", id),
                advance         : (glyph.advance_width, glyph.advance_height),
                bearing         : (0.0, 0.0),
                origin          : (glyph.xmin as f32, glyph.ymin as f32),
                bitmap          : bitmap,
                width           : glyph.width as u32,
                height          : glyph.height as u32,
            });

            max_size.0 = max_size.0.max(glyph.width as f32);
            max_size.1 = max_size.1.max(glyph.height as f32);

            total_size.0 += glyph.width as f32;
            total_size.1 += glyph.height as f32;

        });

        log::info!("Max size: {:?}", max_size);
        log::info!("Total size: {:?}", total_size);
        log::info!("Average size: {:?}", (total_size.0 / glyphs.len() as f32, total_size.1 / glyphs.len() as f32));
        log::info!("Glyph count: {}", glyphs.len());
        log::info!("Font created");
        
        
        Ok(Self {
            glyph_count     : glyphs.len() as u32,
            glyphs          : glyphs,
        })
    }

    pub fn save_bitmaps(&self, path2: &str) -> Result<(), String> {
        for (id, glyph) in self.glyphs.iter() {
            if glyph.bitmap.len() == 0 {
                continue;
            }

            let path = format!("{}/{}.png", path2, *id as u32);
            let img = image::GrayImage::from_raw(glyph.width, glyph.height, glyph.bitmap.clone()).ok_or("Failed to create image")?;
            img.save(path).map_err(|e| format!("Failed to save image: {}", e))?;
        }
        Ok(())
    }

    pub fn glyphs(&self) -> &std::collections::HashMap<char, FontGlyph> {
        &self.glyphs
    }

    
}