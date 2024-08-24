#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::needless_return)]
#![allow(redundant_semicolons)]

pub trait ColorImpl {
    fn from_hex(hex: u32) -> Self;
}

pub trait VectorImpl: Sized {
    type T: std::ops::Add<Output = Self::T>
        + std::ops::Sub<Output = Self::T>
        + std::ops::Mul<Output = Self::T>
        + std::ops::Div<Output = Self::T>;
    fn new(x: Self::T, y: Self::T) -> Self;
    fn x(v: &Self) -> Self::T;
    fn y(v: &Self) -> Self::T;

    fn add(a: Self, b: Self) -> Self {
        Self::new(Self::x(&a) + Self::x(&b), Self::y(&a) + Self::y(&b))
    }

    fn sub(a: Self, b: Self) -> Self {
        Self::new(Self::x(&a) - Self::x(&b), Self::y(&a) - Self::y(&b))
    }

    fn mul(a: Self, b: Self) -> Self {
        Self::new(Self::x(&a) * Self::x(&b), Self::y(&a) * Self::y(&b))
    }

    fn div(a: Self, b: Self) -> Self {
        Self::new(Self::x(&a) / Self::x(&b), Self::y(&a) / Self::y(&b))
    }
}

define_vectors!();
define_colors!();

/* --- Tileset --- */
pub type TilesetID = u32;

#[derive([SERDE]Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tileset {
    pub id: TilesetID,
    pub path: &'static str,
}

impl Tileset {
    pub const fn new(id: TilesetID, path: &'static str) -> Self {
        Self { id, path }
    }
}

#[derive([SERDE]Clone, Debug)]
pub struct Tile {
    pub position: UVec2,
    pub flip: FlipMode,
}

impl Tile {
    pub fn new(position: UVec2, flip: FlipMode) -> Self {
        Self { position, flip }
    }
}

#[derive([SERDE]Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlipMode {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl FlipMode {
    pub fn horizontal(&self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }

    pub fn vertical(&self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
}

/* --- Entity --- */
#[derive([SERDE]Clone, Debug)]
pub struct EntityObject {
    pub entity: Entity,
    pub position: FVec2,
    pub size: UVec2,
}

impl EntityObject {
    pub fn new(entity: Entity, position: FVec2, size: UVec2) -> Self {
        Self {
            entity,
            position,
            size,
        }
    }

    pub fn top_left(&self) -> FVec2 {
        <FVec2 as VectorImpl>::sub(
            self.position,
            <FVec2 as VectorImpl>::mul(
                <FVec2 as VectorImpl>::new(self.size.x as _, self.size.y as _),
                self.entity.pivot(),
            ),
        )
    }
}

#[derive([SERDE]Clone, Debug)]
pub enum RenderMode {
    Rectangle,
    Ellipse,
    Cross,
    Tile {
        tileset: TilesetID,
        tile: UVec2,
        size: UVec2,
    },
}

#[derive([SERDE]Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityRef {
    level: usize,
    layer: usize,
    entity: usize,
}

impl EntityRef {
    pub fn new(level: usize, layer: usize, entity: usize) -> Self {
        Self {
            level,
            layer,
            entity,
        }
    }

    pub fn find<'a>(&self, world: &'a World) -> Option<&'a EntityObject> {
        let level = world.get(self.level)?;
        match self.layer {
            LAYER_INDEX => GET_LAYER!(),
            _ => None,
        }
    }

    pub fn find_mut<'a>(&self, world: &'a mut World) -> Option<&'a mut EntityObject> {
        let level = world.get_mut(self.level)?;
        match self.layer {
            LAYER_INDEX => GET_LAYER_mut!(),
            _ => None,
        }
    }
}

/* --- World --- */
#[derive([SERDE]Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorldLayout {
    Free,
    GridVania,
    LinearHorizontal,
    LinearVertical,
}

/* --- Traits --- */
pub mod traits {
    use super::{Color, ColorImpl, FVec2, IVec2, UVec2, VectorImpl};

    /// A layer trait
    pub trait Layer {
        const GRID_SIZE: u32;
        const OPACITY: f32;
        const PARALLAX_SCALING: bool;

        fn parallax_factor() -> FVec2;
        fn pixel_offset() -> IVec2;
        fn tile_pivot() -> FVec2;

        fn size(&self) -> UVec2;
        fn pixel_size(&self) -> UVec2 {
            <UVec2 as VectorImpl>::mul(self.size(), self.grid_size())
        }

        fn grid_size(&self) -> UVec2 {
            <UVec2 as VectorImpl>::new(Self::GRID_SIZE as _, Self::GRID_SIZE as _)
        }
    }

    /// A trait for layers that can be indexed
    pub trait IndexableLayer: Layer {
        type Tile;

        fn get(&self, position: IVec2) -> Option<&Self::Tile>;
        fn get_mut(&mut self, position: IVec2) -> Option<&mut Self::Tile>;
        fn rect(&self, start: IVec2, size: UVec2) -> TileRegion<'_, Self>
        where
            Self: std::marker::Sized,
        {
            TileRegion::new(self, start, size)
        }
    }

    // * --- Actual layers--- * //
    use super::EntityObject;
    use super::{Tile, TilesetID};

    /// An integer grid layer trait
    pub trait IntGrid: IndexableLayer {}

    /// A tile layer trait
    pub trait Tiles: IndexableLayer<Tile = Tile> {
        const TILESET_ID: TilesetID;
    }

    /// An auto layer trait
    pub trait AutoLayer: Layer {
        const TILESET_ID: TilesetID;

        fn get_autotile(&self, position: IVec2) -> Vec<Tile>;
        fn autotile_rect(&self, start: IVec2, size: UVec2) -> AutoLayerRegion<'_, Self>
        where
            Self: std::marker::Sized,
        {
            AutoLayerRegion::new(self, start, size)
        }
    }

    /// An entities layer trait
    pub trait Entities: Layer {
        fn entities(&self) -> &Vec<EntityObject>;
        fn entities_mut(&mut self) -> &mut Vec<EntityObject>;
    }

    macro_rules! rectangular_region {
        ($name:ident ($source:ident) -> $type:ty: $self:ident -> $expr:expr) => {
            pub struct $name<'a, S: $source> {
                start: IVec2,
                size: UVec2,
                position: IVec2,
                source: &'a S,
            }

            impl<'a, S: $source> $name<'a, S> {
                pub fn new(source: &'a S, start: IVec2, size: UVec2) -> Self {
                    Self {
                        source,
                        start,
                        size,
                        position: start,
                    }
                }
            }

            impl<'a, S: $source> Iterator for $name<'a, S>  {
                type Item = (IVec2, $type);

                fn next(&mut $self) -> Option<Self::Item> {
                    if <IVec2 as VectorImpl>::y(&$self.position) >= <IVec2 as VectorImpl>::y(&$self.start) + <UVec2 as VectorImpl>::y(&$self.size) as <IVec2 as VectorImpl>::T {
                        return None;
                    }
                    let tile = ($self.position, $expr);
                    $self.position = <IVec2 as VectorImpl>::add($self.position, <IVec2 as VectorImpl>::new(1 as _, 0 as _));
                    if <IVec2 as VectorImpl>::x(&$self.position) >= <IVec2 as VectorImpl>::x(&$self.start) + <UVec2 as VectorImpl>::x(&$self.size) as <IVec2 as VectorImpl>::T {
                        $self.position = <IVec2 as VectorImpl>::new(<IVec2 as VectorImpl>::x(&$self.start), <IVec2 as VectorImpl>::y(&$self.position) + 1 as <IVec2 as VectorImpl>::T);
                    }
                    Some(tile)
                }
            }
        };
    }

    rectangular_region!(TileRegion(IndexableLayer) -> Option<&'a S::Tile>: self -> self.source.get(self.position));
    rectangular_region!(AutoLayerRegion(AutoLayer) -> Vec<Tile>: self -> self.source.get_autotile(self.position));
}
