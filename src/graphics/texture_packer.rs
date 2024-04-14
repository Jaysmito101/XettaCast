use core::str;
use std::hash::{Hash, Hasher};

use rayon::prelude::*;

struct TexturePackerSpace 
{
    x               : u32,
    y               : u32,
    width           : u32,
    height          : u32,
    layer           : u32,
}

pub struct TexturePacker 
{
    width           : u32,
    height          : u32,
    depth           : u32,
    layers          : Vec<Vec<u8>>,
    spaces          : Vec<TexturePackerSpace>,
    registered      : std::collections::HashMap<String, TexturePackerSpace>,
    to_add          : Vec<(String, u32, u32)>,
}

impl TexturePackerSpace 
{
    pub fn area(&self) -> u32 
    {
        self.width * self.height
    }
    
    pub fn fits(&self, width: u32, height: u32) -> bool 
    {
        self.width >= width && self.height >= height
    }
    
    pub fn fits_in(&self, other: &TexturePackerSpace) -> bool 
    {
        self.width <= other.width && self.height <= other.height
    }
    
    pub fn compare(&self, other: &TexturePackerSpace) -> std::cmp::Ordering 
    {
        self.area().cmp(&other.area())
    }
}

impl TexturePacker 
{
    
    pub async fn new(width: u32, height: u32, layer_count: u32) -> Result<Self, String> 
    {
        let mut layers = Vec::new();
        let mut spaces = Vec::new();
        for i in 0..layer_count {
            layers.push(vec![0u8; (width * height * 4) as usize]);
            spaces.push(TexturePackerSpace {
                x               : 0,
                y               : 0,
                width           : width,
                height          : height,
                layer           : i,
            });
        }
        
        Ok(Self {
            width           : width,
            height          : height,
            depth           : layers.len() as u32,
            layers          : layers,
            spaces          : spaces,
            registered      : std::collections::HashMap::new(),
            to_add          : Vec::new(),
        })
    }
    
    pub fn save(&self, path: &str, layer: u32) -> Result<(), String> 
    {
        assert!(layer < self.depth);
        image::save_buffer(path, &self.layers[layer as usize], self.width, self.height, image::ColorType::Rgba8).map_err(|e| e.to_string())
    }
    
    pub fn reset(&mut self) 
    {
        self.spaces.clear();
        for i in 0..self.depth {
            self.spaces.push(TexturePackerSpace {
                x               : 0,
                y               : 0,
                width           : self.width,
                height          : self.height,
                layer           : i,
            });
        }
        self.registered.clear();
        self.to_add.clear();
    }
    
    
    pub fn add(&mut self, name: &str, width: u32, height: u32) {
        self.to_add.push((name.to_string(), width, height));
    }
    
    pub fn update(&mut self, name: &str, data: &[u8], width: u32, height: u32) {
        if let Some(space) = self.registered.get(name) {
            let layer = &mut self.layers[space.layer as usize];
            // scale the data to the space
            for y in 0..space.height {
                for x in 0..space.width {
                    
                    if width != space.width || height != space.height {
                        let x_f = x as f32 / space.width as f32;
                        let y_f = y as f32 / space.height as f32;
                        let xi = (x_f * (width - 1) as f32) as u32;
                        let yi = (y_f * (height - 1) as f32) as u32;
                        let indexi = (yi * width + xi) as usize * 4;
                        let indexo = ((space.y + y) * self.width + (space.x + x)) as usize * 4;
                        layer[indexo + 0] = data[indexi + 0];
                        layer[indexo + 1] = data[indexi + 1];
                        layer[indexo + 2] = data[indexi + 2];
                        layer[indexo + 3] = data[indexi + 3];
                    }
                    else {
                        let indexi = (y * width + x) as usize * 4;
                        let indexo = ((space.y + y) * self.width + (space.x + x)) as usize * 4;
                        layer[indexo + 0] = data[indexi + 0];
                        layer[indexo + 1] = data[indexi + 1];
                        layer[indexo + 2] = data[indexi + 2];
                        layer[indexo + 3] = data[indexi + 3];
                    }
                }
            }
        }
    }
    
    pub fn pack(&mut self) -> Result<(), String> 
    {
        self.to_add.sort_by(|a, b| {
            let area_a = a.1 * a.2;
            let area_b = b.1 * b.2;
            area_b.cmp(&area_a)
        });
        
        let to_add = self.to_add.clone();
        let mut could_not_pack = vec![];
        for (name, width, height) in to_add.iter().cloned() {
            match self.pack_item(name.as_str(), width, height) {
                Ok(_) => {},
                Err(e) => {
                    could_not_pack.push(name);
                }
            }
        }
        
        self.to_add.retain(|(name, _, _)| could_not_pack.contains(name));
        
        if could_not_pack.len() > 0 {
            return Err(format!("Could not pack: {} of {} {:?}", could_not_pack.len(), to_add.len(), could_not_pack));
        }
        
        Ok(())
    }
    
    fn pack_item(&mut self, name: &str, width: u32, height: u32) -> Result<(u32, u32), String> 
    {
        for index in (0..self.spaces.len()).rev() {
            let space = &mut self.spaces[index];
            if space.width < width || space.height < height {
                continue;
            }
            
            let space = self.spaces.remove(index);
            
            let x = space.x;
            let y = space.y;
            
            let reg_space = TexturePackerSpace {
                x               : x,
                y               : y,
                width           : width,
                height          : height,
                layer           : space.layer,
            };
            self.registered.insert(name.to_string(), reg_space);
            
            // case 0 : if the space is exactly the same size as the glyph
            if space.width == width && space.height == height {
            }
            // case 1: if the space is wider than texture but same height
            else if space.width > width && space.height == height {
                self.insert_space( x + width, y, space.width - width, height, space.layer);
            }
            // case 2: if the space is taller than texture but same width
            else if space.width == width && space.height > height {
                self.insert_space( x, y + height, width, space.height - height, space.layer);
            }
            // case 3: if the space is wider and taller than texture
            else if space.width > width && space.height > height {
                // case split 0
                let ar0_sp0 = (space.width - width) * height;
                let ar1_sp0 = space.width * (space.height - height);
                let diff0 = (ar0_sp0 as i32 - ar1_sp0 as i32).abs();
                
                // case split 1
                let ar0_sp1 = (space.width - width) * space.height;
                let ar1_sp1 = width * (space.height - height);
                let diff1 = (ar0_sp1 as i32 - ar1_sp1 as i32).abs();
                
                if diff0 > diff1 {
                    if ar0_sp0 > ar1_sp0 {
                        self.insert_space( x + width, y, space.width - width, height, space.layer);
                        self.insert_space( x, y + height, space.width, space.height - height, space.layer);                        
                    } else {
                        self.insert_space( x, y + height, space.width, space.height - height, space.layer);
                        self.insert_space( x + width, y, space.width - width, height, space.layer);
                    }                        
                    
                } else {
                    if ar0_sp1 > ar1_sp1 {
                        self.insert_space( x + width, y, space.width - width, space.height, space.layer);
                        self.insert_space( x, y + height, width, space.height - height, space.layer);
                    } else {
                        self.insert_space( x, y + height, width, space.height - height, space.layer);
                        self.insert_space( x + width, y, space.width - width, space.height, space.layer);
                    }
                }
                
                
            }
            
            return Ok((x, y));
        }
        
        Err("No space left".to_string())
    }
    
    // for visualizing the texture packer
    pub fn fill_color(&mut self) -> Result<(), String> 
    {
        // first clear all layers with (0, 0, 0, 0)
        // then draw all registered spaces with a unique color
        for layer in self.layers.iter_mut() {
            layer.par_iter_mut().for_each(|pixel| *pixel = 0);
        }
        
        for (name, space) in self.registered.iter() {
            let color = Self::name_color(name.as_str());
            let layer = &mut self.layers[space.layer as usize];
            for y in 0..space.height {
                for x in 0..space.width {
                    let index = ((space.y + y) * self.width + (space.x + x)) as usize * 4;
                    layer[index + 0] = (color >> 24) as u8;
                    layer[index + 1] = (color >> 16) as u8;
                    layer[index + 2] = (color >> 8) as u8;
                    layer[index + 3] = (color >> 0) as u8;
                }
            }
        }
        
        Ok(())
    }
    
    pub fn fill_color_empty(&mut self) -> Result<(), String> 
    {
        // first clear all layers with (0, 0, 0, 0)
        // then draw all registered spaces with a unique color
        for layer in self.layers.iter_mut() {
            layer.par_iter_mut().for_each(|pixel| *pixel = 0);
        }
        
        for space in self.spaces.iter() {
            let color = rand::random::<u32>() | 0x000000FF;
            let layer = &mut self.layers[space.layer as usize];
            for y in 0..space.height {
                for x in 0..space.width {
                    let index = ((space.y + y) * self.width + (space.x + x)) as usize * 4;
                    layer[index + 0] = (color >> 24) as u8;
                    layer[index + 1] = (color >> 16) as u8;
                    layer[index + 2] = (color >> 8) as u8;
                    layer[index + 3] = (color >> 0) as u8;
                }
            }
        }
        
        Ok(())
    }
    
    
    fn insert_space(&mut self, x: u32, y: u32, width: u32, height: u32, layer: u32) 
    {
        let space = TexturePackerSpace {
            x               : x,
            y               : y,
            width           : width,
            height          : height,
            layer           : layer,
        };
        
        self.spaces.push(space);
    }
    
    fn name_color(str: &str) -> u32 
    {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        str.hash(&mut hasher);
        hasher.finish() as u32 | 0x000000FF
    }
    
}