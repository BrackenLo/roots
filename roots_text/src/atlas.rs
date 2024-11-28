//====================================================================

use std::{collections::HashSet, error::Error, fmt::Display, hash::BuildHasherDefault};

use cosmic_text::{CacheKey, SwashImage};
use etagere::{euclid::Size2D, AllocId, BucketedAtlasAllocator};
use lru::LruCache;
use roots_common::Size;
use roots_renderer::{texture::Texture, tools};
use rustc_hash::FxHasher;

//====================================================================

type FastHasher = BuildHasherDefault<FxHasher>;

pub struct GlyphData {
    alloc_id: AllocId,
    pub uv_start: [f32; 2],
    pub uv_end: [f32; 2],
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub enum CacheGlyphError {
    NoGlyphImage,
    OutOfSpace,
    LruStorageError,
}

impl Error for CacheGlyphError {}

impl Display for CacheGlyphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match &self {
            CacheGlyphError::NoGlyphImage => "Unable to get image from proved glyph.",
            CacheGlyphError::OutOfSpace => {
                "Atlas texture is not big enough to store new glyphs - TODO"
            }
            CacheGlyphError::LruStorageError => {
                "Error accessing glyphs from LRU - This shouldn't really happen."
            }
        };

        write!(f, "{}", msg)
    }
}

//====================================================================

pub struct TextAtlas {
    packer: BucketedAtlasAllocator,

    glyphs_in_use: HashSet<CacheKey, FastHasher>,
    cached_glyphs: LruCache<CacheKey, GlyphData, FastHasher>,

    texture: Texture,
    texture_size: Size<u32>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl TextAtlas {
    pub fn new(device: &wgpu::Device) -> Self {
        const DEFAULT_START_SIZE: u32 = 256;

        let packer = BucketedAtlasAllocator::new(Size2D::new(
            DEFAULT_START_SIZE as i32,
            DEFAULT_START_SIZE as i32,
        ));
        let glyphs_in_use = HashSet::with_hasher(FastHasher::default());
        let cached_glyphs = LruCache::unbounded_with_hasher(FastHasher::default());

        let texture_size = Size::new(DEFAULT_START_SIZE, DEFAULT_START_SIZE);
        let texture = Texture::from_size(device, texture_size, Some("Text Atlas Texture"), None);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Text Atlas Bind Group Layout"),
            entries: &[
                tools::bgl_entry(tools::BgEntryType::Texture, 0, wgpu::ShaderStages::FRAGMENT),
                tools::bgl_entry(tools::BgEntryType::Sampler, 1, wgpu::ShaderStages::FRAGMENT),
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Atlas Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        Self {
            packer,
            glyphs_in_use,
            cached_glyphs,
            texture,
            texture_size,
            bind_group_layout,
            bind_group,
        }
    }

    #[inline]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

//--------------------------------------------------

impl TextAtlas {
    // Cache glyph if not already and then promote in LRU
    pub fn use_glyph(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut cosmic_text::FontSystem,
        swash_cache: &mut cosmic_text::SwashCache,
        key: &CacheKey,
    ) -> Result<(), CacheGlyphError> {
        // Already has glyph cached
        if self.cached_glyphs.contains(key) {
            self.cached_glyphs.promote(key);
            self.glyphs_in_use.insert(*key);

            Ok(())
        }
        // Try to cache glyph
        else {
            let image = swash_cache
                .get_image_uncached(font_system, *key)
                .ok_or(CacheGlyphError::NoGlyphImage)?;

            self.cache_glyph(device, queue, key, &image)?;

            self.cached_glyphs.promote(key);
            self.glyphs_in_use.insert(*key);
            Ok(())
        }
    }

    #[inline]
    pub fn get_glyph_data(&mut self, key: &CacheKey) -> Option<&GlyphData> {
        self.cached_glyphs.get(key)
    }

    fn cache_glyph(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: &CacheKey,
        image: &SwashImage,
    ) -> Result<(), CacheGlyphError> {
        let image_width = image.placement.width;
        let image_height = image.placement.height;

        let size = etagere::Size::new(image_width.max(1) as i32, image_height.max(1) as i32);

        let allocation = loop {
            match self.packer.allocate(size) {
                Some(allocation) => break allocation,

                // Keep trying to free space until error or can allocate
                None => self.free_space(device)?,
            }
        };

        let x = allocation.rectangle.min.x as u32;
        let y = allocation.rectangle.min.y as u32;

        self.texture
            .update_area(queue, &image.data, x, y, image_width, image_height);

        let uv_start = [
            allocation.rectangle.min.x as f32 / self.texture_size.width as f32,
            allocation.rectangle.min.y as f32 / self.texture_size.height as f32,
        ];

        let uv_end = [
            allocation.rectangle.max.x as f32 / self.texture_size.width as f32,
            allocation.rectangle.max.y as f32 / self.texture_size.height as f32,
        ];

        let left = image.placement.left as f32;
        let top = image.placement.top as f32;
        let width = image.placement.width as f32;
        let height = image.placement.height as f32;

        // log::trace!(
        //     "Allocated glyph id {:?}, with size {:?} and uv ({:?}, {:?})",
        //     &key.glyph_id,
        //     size,
        //     uv_start,
        //     uv_end
        // );

        let glyph_data = GlyphData {
            alloc_id: allocation.id,
            uv_start,
            uv_end,
            left,
            top,
            width,
            height,
        };

        self.cached_glyphs.put(*key, glyph_data);

        Ok(())
    }

    fn free_space(&mut self, _device: &wgpu::Device) -> Result<(), CacheGlyphError> {
        //
        match self.cached_glyphs.peek_lru() {
            // Check if last used key is in use. If so, grow atlas
            Some((key, _)) => {
                if self.glyphs_in_use.contains(key) {
                    // TODO - Try to grow glyph cache - Make sure to re-set all glyph data UVs
                    return Err(CacheGlyphError::OutOfSpace);
                }
            }
            // Issues with size of lru
            None => return Err(CacheGlyphError::LruStorageError),
        };

        let (key, val) = self.cached_glyphs.pop_lru().unwrap();

        self.packer.deallocate(val.alloc_id);
        self.cached_glyphs.pop(&key);

        return Ok(());
    }

    #[inline]
    pub fn post_render_trim(&mut self) {
        self.glyphs_in_use.clear();
    }
}

//====================================================================
