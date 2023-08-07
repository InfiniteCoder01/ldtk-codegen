pub mod math {
    SERDE_USE!();
    #[derive([SERDE]Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Vec2<T> {
        pub x: T,
        pub y: T,
    }

    impl<T> Vec2<T> {
        pub const fn new(x: T, y: T) -> Self {
            Self { x, y }
        }

        pub fn casted<I>(self) -> I
        where
            Self: Into<I>,
        {
            self.into()
        }

        pub fn min<I: Into<Self>>(self, other: I) -> Self
        where
            T: Ord,
        {
            let other = other.into();
            Self::new(
                std::cmp::min(self.x, other.x),
                std::cmp::min(self.y, other.y),
            )
        }

        pub fn max<I: Into<Self>>(self, other: I) -> Self
        where
            T: Ord,
        {
            let other = other.into();
            Self::new(
                std::cmp::max(self.x, other.x),
                std::cmp::max(self.y, other.y),
            )
        }
    }

    macro_rules! implement_value {
        ($($type: ty),+) => {
            $(
                impl Vec2<$type> {
                    pub const fn zero() -> Self {
                        Self::new(0 as $type, 0 as $type)
                    }

                    pub const fn one() -> Self {
                        Self::new(1 as $type, 1 as $type)
                    }
                }
            )+
        };
    }

    implement_value!(i8, u8, i16, u16, i32, u32, f32, i64, u64, f64);

    macro_rules! implement_binary_op {
        ($trait: ident: $fn: ident ($self: ident, $rhs: ident) => $value: expr) => {
            impl<T, O> std::ops::$trait<O> for Vec2<T>
            where
                O: Into<Vec2<T>>,
                T: std::ops::$trait<T, Output = T>,
            {
                type Output = Self;

                fn $fn($self, rhs: O) -> Self::Output {
                    let $rhs = rhs.into();
                    $value
                }
            }
        };
    }

    implement_binary_op!(Add: add (self, rhs) => Vec2::new(self.x + rhs.x, self.y + rhs.y));
    implement_binary_op!(Sub: sub (self, rhs) => Vec2::new(self.x - rhs.x, self.y - rhs.y));
    implement_binary_op!(Mul: mul (self, rhs) => Vec2::new(self.x * rhs.x, self.y * rhs.y));
    implement_binary_op!(Div: div (self, rhs) => Vec2::new(self.x / rhs.x, self.y / rhs.y));

    macro_rules! implement_assign_op {
        ($trait: ident, $na_trait: ident: $fn: ident ($self: ident, $rhs: ident) => $value: expr) => {
            impl<T, O> std::ops::$trait<O> for Vec2<T>
            where
                O: Into<Vec2<T>>,
                T: std::ops::$na_trait<Output = T> + Copy,
            {
                fn $fn(&mut $self, rhs: O) {
                    let $rhs = rhs.into();
                    *$self = $value;
                }
            }
        };
    }

    impl<T: std::ops::Neg> std::ops::Neg for Vec2<T> {
        type Output = Vec2<T::Output>;

        fn neg(self) -> Self::Output {
            Self::Output::new(-self.x, -self.y)
        }
    }

    implement_assign_op!(AddAssign, Add: add_assign (self, rhs) => Vec2::new(self.x + rhs.x, self.y + rhs.y));
    implement_assign_op!(SubAssign, Sub: sub_assign (self, rhs) => Vec2::new(self.x - rhs.x, self.y - rhs.y));
    implement_assign_op!(MulAssign, Mul: mul_assign (self, rhs) => Vec2::new(self.x * rhs.x, self.y * rhs.y));
    implement_assign_op!(DivAssign, Div: div_assign (self, rhs) => Vec2::new(self.x / rhs.x, self.y / rhs.y));

    macro_rules! implement_from {
        ($type1: ty => $($type2: ty),+) => {
            $(impl From<Vec2<$type2>> for Vec2<$type1> {
                fn from(other: Vec2<$type2>) -> Self {
                    Self::new(other.x as _, other.y as _)
                }
            })+
        };
    }

    implement_from!(i8 => u8, i16, u16, i32, u32, f32, i64, u64, f64);
    implement_from!(u8 => i8, i16, u16, i32, u32, f32, i64, u64, f64);
    implement_from!(i16 => i8, u8, u16, i32, u32, f32, i64, u64, f64);
    implement_from!(u16 => i8, u8, i16, i32, u32, f32, i64, u64, f64);
    implement_from!(i32 => i8, u8, i16, u16, u32, f32, i64, u64, f64);
    implement_from!(u32 => i8, u8, i16, u16, i32, f32, i64, u64, f64);
    implement_from!(f32 => i8, u8, i16, u16, i32, u32, i64, u64, f64);
    implement_from!(i64 => i8, u8, i16, u16, i32, u32, f32, u64, f64);
    implement_from!(u64 => i8, u8, i16, u16, i32, u32, f32, i64, f64);
    implement_from!(f64 => i8, u8, i16, u16, i32, u32, f32, i64, u64);

    impl<T: Copy> From<T> for Vec2<T> {
        fn from(value: T) -> Self {
            Self::new(value, value)
        }
    }

    impl<T: Copy> From<(T, T)> for Vec2<T> {
        fn from(value: (T, T)) -> Self {
            Self::new(value.0, value.1)
        }
    }

    impl<T: Copy> From<Vec2<T>> for (T, T) {
        fn from(value: Vec2<T>) -> Self {
            (value.x, value.y)
        }
    }

    impl<T: Copy> From<[T; 2]> for Vec2<T> {
        fn from(value: [T; 2]) -> Self {
            Self::new(value[0], value[1])
        }
    }

    impl<T: Copy> From<Vec2<T>> for [T; 2] {
        fn from(value: Vec2<T>) -> Self {
            [value.x, value.y]
        }
    }

    macro_rules! generate_from_into {
        (!for $type: ty, $($self_scalar: ident),+) => {
            $(impl From<$type> for Vec2<$self_scalar> {
                fn from(other: $type) -> Self {
                    Self::new(other.x as _, other.y as _)
                }
            }

            impl From<Vec2<$self_scalar>> for $type {
                fn from(other: Vec2<$self_scalar>) -> Self {
                    Self::new(other.x as _, other.y as _)
                }
            })+
        };
        ($($type: ty),+) => {
            $(generate_from_into!(!for $type, i8, u8, i16, u16, i32, u32, f32, i64, u64, f64);)+
        };
    }
    CUSTOM_VECTORS!();
}

use math::*;

/* --- Traits --- */
pub mod layer {
    use super::math::*;

    /// A layer trait
    pub trait Layer {
        const GRID_SIZE: u32;
        const GUIDE_GRID_SIZE: Vec2<u32>;
        const PX_OFFSET: Vec2<i32>;
        const PARALLAX_FACTOR: Vec2<f32>;
        // TODO: parallaxScaling, requiredTags, excludedTags, tilePivot
        // TODO: vars: px_offset, total_px_offset

        fn size(&self) -> Vec2<u32>;
        fn pixel_size(&self) -> Vec2<u32> {
            self.size() * Self::GRID_SIZE
        }

        fn grid_size(&self) -> Vec2<u32> {
            Vec2::from(Self::GRID_SIZE)
        }
    }

    #[macro_export]
    macro_rules! generate_layer {
        (
            $(!doc $layer_doc: literal)?
            $layer: ident:
                grid_size = $grid_size: expr,
                guide_grid_size = $guide_grid_size: expr,
                px_offset = $px_offset: expr,
                parallax_factor = $parallax_factor: expr,
                $($field: ident: $field_type: ty,)*
        ) => {
            $(#[doc = $layer_doc])?
            #[derive([SERDE]Clone, Debug, PartialEq, PartialOrd)]
            pub struct $layer {
                size: Vec2<u32>,
                $($field: $field_type,)*
            }

            impl layer::Layer for $layer {
                const GRID_SIZE: u32 = $grid_size;
                const GUIDE_GRID_SIZE: Vec2<u32> = $guide_grid_size;
                const PX_OFFSET: Vec2<i32> = $px_offset;
                const PARALLAX_FACTOR: Vec2<f32> = $parallax_factor;
                fn size(&self) -> Vec2<u32> {
                    return self.size;
                }
            }
        };
    }
    pub use generate_layer;

    pub struct RectangularRegion {
        start: Vec2<u32>,
        size: Vec2<u32>,
        position: Vec2<u32>,
    }

    impl RectangularRegion {
        pub fn new(start: Vec2<u32>, size: Vec2<u32>) -> Self {
            Self {
                start,
                size,
                position: start,
            }
        }
    }

    impl Iterator for RectangularRegion {
        type Item = Vec2<u32>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.position.y >= self.start.y + self.size.y {
                return None;
            }
            let tile = self.position;
            self.position.x += 1;
            if self.position.x > self.start.x + self.size.x {
                self.position.x = self.start.x;
                self.position.y += 1;
            }
            Some(tile)
        }
    }

    macro_rules! rectangular_region {
        ($name: ident ($source: ident) -> $type: ty: $self: ident -> $expr: expr) => {
            pub struct $name<'a, S: $source> {
                start: Vec2<u32>,
                size: Vec2<u32>,
                position: Vec2<u32>,
                source: &'a S,
            }

            impl<'a, S: $source> $name<'a, S> {
                pub fn new(source: &'a S, start: Vec2<u32>, size: Vec2<u32>) -> Self {
                    Self {
                        source,
                        start,
                        size,
                        position: start,
                    }
                }
            }

            impl<'a, S: $source> Iterator for $name<'a, S>  {
                type Item = (Vec2<u32>, $type);

                fn next(&mut $self) -> Option<Self::Item> {
                    if $self.position.y >= $self.start.y + $self.size.y {
                        return None;
                    }
                    let tile = ($self.position, $expr);
                    $self.position.x += 1;
                    if $self.position.x > $self.start.x + $self.size.x {
                        $self.position.x = $self.start.x;
                        $self.position.y += 1;
                    }
                    Some(tile)
                }
            }
        };
    }

    rectangular_region!(IntGridRegion(IntGrid) -> &'a S::Tile: self -> self.source.get(self.position)?);
    rectangular_region!(AutoLayerRegion(AutoLayer) -> Vec<Tile>: self -> self.source.get_autotile(self.position));

    // * -------------------------------------------------------------------------------- Int Grid -------------------------------------------------------------------------------- * //
    /// An integer grid layer trait
    pub trait IntGrid: Layer {
        type Tile;

        fn get(&self, position: impl Into<Vec2<i32>>) -> Option<&Self::Tile>;
        fn get_mut(&mut self, position: impl Into<Vec2<i32>>) -> Option<&mut Self::Tile>;
        fn rect(
            &self,
            start: impl Into<Vec2<i32>>,
            size: impl Into<Vec2<u32>>,
        ) -> IntGridRegion<'_, Self>
        where
            Self: std::marker::Sized,
        {
            IntGridRegion::new(self, start.into().max(0).into(), size.into())
        }
    }

    #[macro_export]
    macro_rules! generate_int_grid_layer {
        (
            $(!doc $layer_doc: literal)?
            $layer: ident:
                grid_size = $grid_size: expr,
                guide_grid_size = $guide_grid_size: expr,
                px_offset = $px_offset: expr,
                parallax_factor = $parallax_factor: expr,

            $(!auto_layer $auto_layer: ident)?

            $layer_tile: ident:
                $($tile_variant: ident),*
        ) => {
            #[doc = concat!("Possible tiles for '", stringify!($layer_tile), "' int grid layer")]
            #[derive([SERDE]Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
            pub enum $layer_tile {
                #[default]
                Empty,
                $($tile_variant,)*
            }

            layer::generate_layer!(
                $(!doc $layer_doc)?
                $layer:
                    grid_size = $grid_size,
                    guide_grid_size = $guide_grid_size,
                    px_offset = $px_offset,
                    parallax_factor = $parallax_factor,
                    tiles: Vec<$layer_tile>,
                    $(
                        $auto_layer: Vec<Vec<Tile>>,
                        tileset: TilesetID,
                    )?
            );

            impl layer::IntGrid for $layer {
                type Tile = $layer_tile;

                fn get(&self, position: impl Into<Vec2<i32>>) -> Option<&Self::Tile> {
                    let position = position.into();
                    if position.x < 0 || position.y < 0 || position.x as u32 >= self.size.x || position.y as u32 >= self.size.y {
                        return None;
                    }
                    return self.tiles.get(position.x as usize + position.y as usize * self.size.x as usize);
                }

                fn get_mut(&mut self, position: impl Into<Vec2<i32>>) -> Option<&mut Self::Tile> {
                    let position = position.into();
                    if position.x < 0 || position.y < 0 || position.x as u32 >= self.size.x || position.y as u32 >= self.size.y {
                        return None;
                    }
                    return self.tiles.get_mut(position.x as usize + position.y as usize * self.size.x as usize);
                }
            }

            impl<T: Into<Vec2<i32>>> std::ops::Index<T> for $layer {
                type Output = <Self as layer::IntGrid>::Tile;

                fn index(&self, position: T) -> &Self::Output {
                    use layer::IntGrid;
                    return self.get(position).unwrap();
                }
            }

            impl<T: Into<Vec2<i32>>> std::ops::IndexMut<T> for $layer {
                fn index_mut(&mut self, position: T) -> &mut Self::Output {
                    use layer::IntGrid;
                    return self.get_mut(position).unwrap();
                }
            }

            $(
                layer::implement_auto_layer!($auto_layer, $layer);
            )?
        };
    }
    pub(super) use generate_int_grid_layer;

    // * -------------------------------------------------------------------------------- AutoLayer ------------------------------------------------------------------------------- * //
    // TODO: Move to TileLayer once exists
    use super::Tile;
    use super::TilesetID;

    /// An integer grid layer trait
    pub trait AutoLayer: Layer {
        fn tileset_id(&self) -> TilesetID;
        fn get_autotile(&self, position: impl Into<Vec2<i32>>) -> Vec<Tile>;

        fn autotile_rect(
            &self,
            start: impl Into<Vec2<i32>>,
            size: impl Into<Vec2<u32>>,
        ) -> AutoLayerRegion<'_, Self>
        where
            Self: std::marker::Sized,
        {
            AutoLayerRegion::new(self, start.into().max(0).into(), size.into())
        }
    }

    #[macro_export]
    macro_rules! implement_auto_layer {
        ($source: ident, $layer: ident) => {
            impl layer::AutoLayer for $layer {
                fn tileset_id(&self) -> TilesetID {
                    self.tileset
                }

                fn get_autotile(&self, position: impl Into<Vec2<i32>>) -> Vec<Tile> {
                    let position = position.into();
                    if position.x < 0
                        || position.y < 0
                        || position.x as u32 >= self.size.x
                        || position.y as u32 >= self.size.y
                    {
                        return Vec::new();
                    }
                    return self
                        .auto_tiles
                        .get(position.x as usize + position.y as usize * self.size.x as usize)
                        .cloned()
                        .unwrap_or_default();
                }
            }
        };
    }
    pub(super) use implement_auto_layer;

    // * -------------------------------------------------------------------------------- Entities -------------------------------------------------------------------------------- * //
    use super::EntityObject;

    /// An entities layer trait
    pub trait Entities: Layer {
        fn entities(&self) -> &Vec<EntityObject>;
        fn entities_mut(&mut self) -> &mut Vec<EntityObject>;
    }

    #[macro_export]
    macro_rules! generate_entities_layer {
        (
            $(!doc $layer_doc: literal)?
            $layer: ident:
                grid_size = $grid_size: expr,
                guide_grid_size = $guide_grid_size: expr,
                px_offset = $px_offset: expr,
                parallax_factor = $parallax_factor: expr,
        ) => {
            layer::generate_layer!(
                $(!doc $layer_doc)?
                $layer:
                    grid_size = $grid_size,
                    guide_grid_size = $guide_grid_size,
                    px_offset = $px_offset,
                    parallax_factor = $parallax_factor,
                    entities: Vec<EntityObject>,
            );

            impl layer::Entities for $layer {
                fn entities(&self) -> &Vec<EntityObject> {
                    &self.entities
                }

                fn entities_mut(&mut self) -> &mut Vec<EntityObject> {
                    &mut self.entities
                }
            }

            impl std::ops::Deref for $layer {
                type Target = Vec<EntityObject>;

                fn deref(&self) -> &Self::Target {
                    use layer::Entities;
                    self.entities()
                }
            }

            impl std::ops::DerefMut for $layer {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    use layer::Entities;
                    self.entities_mut()
                }
            }

        };
    }
    pub(super) use generate_entities_layer;
}

#[derive([SERDE]Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LDTKColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl LDTKColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn casted<I>(self) -> I
    where
        Self: Into<I>,
    {
        self.into()
    }
}

impl From<u32> for LDTKColor {
    fn from(value: u32) -> Self {
        Self::new(
            (value >> 24 & 0xff) as u8,
            (value >> 16 & 0xff) as u8,
            (value >> 8 & 0xff) as u8,
            (value & 0xff) as u8,
        )
    }
}

impl From<LDTKColor> for u32 {
    fn from(value: LDTKColor) -> Self {
        (value.r as u32) << 24 | (value.g as u32) << 16 | (value.b as u32) << 8 | value.a as u32
    }
}

macro_rules! generate_color_from_into {
    ($($type: ty),+) => {
        $(impl From<$type> for LDTKColor {
            fn from(other: $type) -> Self {
                Self::new(other.r, other.g, other.b, other.a)
            }
        }

        impl From<LDTKColor> for $type {
            fn from(other: LDTKColor) -> Self {
                Self::new(other.r, other.g, other.b, other.a)
            }
        })+
    };
}
CUSTOM_COLORS!();
/* --- Tileset --- */
type TilesetID = u32;

#[derive([SERDE]Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tileset {
    id: TilesetID,
    path: std::path::PathBuf,
}

impl Tileset {
    pub fn new(id: TilesetID, path: std::path::PathBuf) -> Self {
        Self { id, path }
    }

    pub fn id(&self) -> TilesetID {
        self.id
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

#[derive([SERDE]Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tile {
    position: Vec2<u32>,
    flip: FlipMode,
}

impl Tile {
    pub fn new(position: Vec2<u32>, flip: FlipMode) -> Self {
        Self { position, flip }
    }

    pub fn position(&self) -> Vec2<u32> {
        self.position
    }

    pub fn flip(&self) -> FlipMode {
        self.flip
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
#[derive([SERDE]Clone, Debug, PartialEq, PartialOrd)]
pub struct EntityObject {
    pub entity: Entity, // Vec<Component> if LDtk will support ECS
    pub position: Vec2<f32>,
    pub size: Vec2<u32>,
    // TODO: LDTKColor, tileRenderMode (nineSliceBorders, tileRect), tags
}

impl EntityObject {
    pub fn new(entity: Entity, position: Vec2<f32>, size: Vec2<u32>) -> Self {
        Self {
            entity,
            position,
            size,
        }
    }

    pub fn top_left(&self) -> Vec2<f32> {
        self.position - self.size * self.entity.pivot()
    }
}

#[derive([SERDE]Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderMode {
    Rectangle,
    Ellipse,
    Cross,
    Tile {
        tileset: TilesetID,
        tile: Vec2<u32>,
        size: Vec2<u32>,
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
