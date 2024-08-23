pub mod entities;
pub mod int_grid;
pub mod tiles;
use crate::definitions::*;

// * ------------------------------------- Defs ------------------------------------- * //
pub fn generate_levels(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    project: &LdtkJson,
    code: &mut Scope,
) -> Result<()> {
    entities::definitions(preferences, definitions, project, code)?;

    let mut level = codegen::Struct::new("Level");
    level.doc("Level in this LDTK project").vis("pub");
    derive_rust_object!(level preferences.serde,);

    level.new_field("bg_color", "Color").vis("pub");
    // TODO: Background image
    level
        .new_field("pixel_size", "UVec2")
        .vis("pub")
        .doc("Size of the level in pixels");
    level.new_field("world_depth", "i64").vis("pub");
    level.new_field("world_x", "i64").vis("pub");
    level.new_field("world_y", "i64").vis("pub");

    code.raw("/* --- Layers --- */");
    for layer_json in &project.defs.layers {
        match layer_json.purple_type {
            Type::IntGrid => {
                int_grid::layer_definition(preferences, definitions, layer_json, code, &mut level)?
            }
            Type::Tiles => {
                tiles::layer_definition(preferences, definitions, layer_json, code, &mut level)?
            }
            Type::AutoLayer => println!("TODO: Auto layer"),
            Type::Entities => {
                entities::layer_definition(preferences, definitions, layer_json, code, &mut level)
            }
        }
    }

    for field in &project.defs.level_fields {
        let rs_type = RsFieldType::parse(field)?;
        level
            .new_field(
                preferences.to_case(&field.identifier, Case::Snake),
                rs_type.string_type(),
            )
            .vis("pub");

        definitions
            .level
            .fields
            .insert(field.identifier.clone(), rs_type);
    }

    code.raw("/* --- Level --- */");
    code.push_struct(level);
    generate_world(preferences, definitions, code, project)?;
    Ok(())
}

// * ----------------------------------- Instance ----------------------------------- * //
pub fn generate_world(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    code: &mut Scope,
    project: &LdtkJson,
) -> Result<()> {
    for (level_index, level) in project.levels.iter().enumerate() {
        for (layer_index, layer) in level
            .layer_instances
            .as_ref()
            .context("External levels are not supported yet!")?
            .iter()
            .enumerate()
        {
            for (entity_index, entity) in layer.entity_instances.iter().enumerate() {
                definitions.entity_instances.insert(
                    entity.iid.clone(),
                    RsEntityInstance {
                        level: level_index,
                        layer: layer_index,
                        entity: entity_index,
                    },
                );
            }
        }
    }

    for tileset in &project.defs.tilesets {
        if let Some(path) = &tileset.rel_path {
            code.raw(format!(
                "pub const {}: Tileset = Tileset::new({}, {:?});",
                preferences.to_case(&tileset.identifier, Case::UpperSnake),
                tileset.uid,
                path
            ));
        }
    }

    code.new_fn("bg_color")
        .ret("Color")
        .vis("pub")
        .line(format_color(&project.bg_color)?);

    code.raw("/* --- World --- */");
    // * World struct
    let world = code
        .new_struct("World")
        .vis("pub")
        .doc("World that contains levels, accessible by snake_case name or by index");
    derive_rust_object!(world preferences.serde,);

    for level in &project.levels {
        world
            .new_field(preferences.to_case(&level.identifier, Case::Snake), "Level")
            .vis("pub");
    }

    let world_impl = code.new_impl("World");

    // * Get level
    macro_rules! generate_get_level {
        ($fn: ident, $self: ident, $ret: ty, $fmt: literal) => {
            let mut match_block = Block::new("match index");
            for (index, level) in project.levels.iter().enumerate() {
                match_block.line(format!(
                    $fmt,
                    index,
                    &preferences.to_case(&level.identifier, Case::Snake)
                ));
            }
            match_block.line("_ => None,");
            world_impl
                .new_fn(stringify!($fn))
                .vis("pub")
                .$self()
                .arg("index", "usize")
                .ret(stringify!($ret))
                .push_block(match_block);
        };
    }
    generate_get_level!(get, arg_ref_self, Option<&Level>, "{} => Some(&self.{}),");
    generate_get_level!(
        get_mut,
        arg_mut_self,
        Option<&mut Level>,
        "{} => Some(&mut self.{}),"
    );

    generate_impl!(code trait "std::ops::Index<usize>" for "World" => {
        type Output = "Level";

        fn index(&self, index: usize) -> &Self::Output {
            return self.get(index).unwrap();
        }
    });

    generate_impl!(code trait "std::ops::IndexMut<usize>" for "World" => {
        fnmut index_mut(&mut self, index: usize) -> &mut Self::Output {
            return self.get_mut(index).unwrap();
        }
    });

    // * World instance
    let mut world = Block::new("Self");
    for level_json in &project.levels {
        let mut level_rs = Block::new(&format!(
            "{}: Level",
            preferences.to_case(&level_json.identifier, Case::Snake)
        ));
        level_rs.after(",");
        level_rs.line(format!(
            "bg_color: {},",
            format_color(&level_json.bg_color)?
        ));
        level_rs.line(format!(
            "pixel_size: <UVec2 as VectorImpl>::new({} as _, {} as _),",
            level_json.px_wid, level_json.px_hei
        ));
        level_rs.line(format!("world_depth: {},", level_json.world_depth));
        level_rs.line(format!("world_x: {},", level_json.world_x));
        level_rs.line(format!("world_y: {},", level_json.world_y));
        for layer_json in level_json
            .layer_instances
            .as_ref()
            // TODO: External levels
            .context("External levels are not supported yet.")?
        {
            let mut layer_rs = Block::new(&format!(
                "{}: {}",
                preferences.to_case(&layer_json.identifier, Case::Snake),
                &layer_json.identifier
            ));
            layer_rs.after(",");
            if !definitions.layers.contains_key(&layer_json.identifier) {
                continue;
            }
            match &definitions.layers[&layer_json.identifier] {
                RsLayerDefinition::IntGrid(definition) => {
                    int_grid::layer_instance(definition, definitions, &mut layer_rs, layer_json)?
                }
                RsLayerDefinition::Tiles(definition) => {
                    tiles::layer_instance(definition, definitions, &mut layer_rs, layer_json)?
                }
                RsLayerDefinition::Entities => {
                    entities::layer_instance(definitions, &mut layer_rs, layer_json)?
                }
            }
            level_rs.push_block(layer_rs);
        }

        for field in &level_json.field_instances {
            let field_type = definitions
                .level
                .fields
                .get(&field.identifier)
                .context(format!(
                    "Level field was not found in definition ({})!",
                    &field.identifier
                ))?;
            level_rs.line(format!(
                "{}: {},",
                preferences.to_case(&field.identifier, Case::Snake),
                field_type.fmt_value(definitions, field.value.as_ref())?
            ));
        }
        world.push_block(level_rs);
    }

    code.new_impl("World")
        .new_fn("load")
        .ret("Self")
        .vis("pub")
        .push_block(world);

    Ok(())
}

pub fn impl_layer_trait(code: &mut Scope, layer_type_name: &str, layer_json: &LayerDefinition) {
    generate_impl!(code trait "traits::Layer" for layer_type_name => {
        const GRID_SIZE: u32 = format!("{}", layer_json.grid_size);
        const OPACITY: f32 = format!("{:.1}", layer_json.display_opacity);
        const PARALLAX_SCALING: bool = format!("{}", layer_json.parallax_scaling);

        get parallax_factor() -> FVec2 = format!("<FVec2 as VectorImpl>::new({} as _, {} as _)", layer_json.parallax_factor_x, layer_json.parallax_factor_y);
        get pixel_offset() -> IVec2 = format!("<IVec2 as VectorImpl>::new({} as _, {} as _)", layer_json.px_offset_x, layer_json.px_offset_y);
        get tile_pivot() -> FVec2 = format!("<FVec2 as VectorImpl>::new({} as _, {} as _)", layer_json.tile_pivot_x, layer_json.tile_pivot_y);

        fn size(&self) -> UVec2 {
            return self.size;
        }
    });
}

pub fn impl_indexable_layer(
    code: &mut Scope,
    layer_type_name: &str,
    tile_type_name: &str,
    flatten: bool,
) {
    if flatten {
        generate_impl!(code trait "traits::IndexableLayer" for layer_type_name => {
            type Tile = tile_type_name;

            fn get(&self, position: IVec2) -> Option<&Self::Tile> {
                if (<IVec2 as VectorImpl>::x(&position) as i32) < 0 || (<IVec2 as VectorImpl>::y(&position) as i32) < 0 || <IVec2 as VectorImpl>::x(&position) as u32 >= <UVec2 as VectorImpl>::x(&self.size) as u32 || <IVec2 as VectorImpl>::y(&position) as u32 >= <UVec2 as VectorImpl>::y(&self.size) as u32 { return None; };
                return self.tiles.get(<IVec2 as VectorImpl>::x(&position) as usize + <IVec2 as VectorImpl>::y(&position) as usize * <UVec2 as VectorImpl>::x(&self.size) as usize).and_then(|tile| tile.as_ref());
            }

            fnmut get_mut(&mut self, position: IVec2) -> Option<&mut Self::Tile> {
                if (<IVec2 as VectorImpl>::x(&position) as i32) < 0 || (<IVec2 as VectorImpl>::y(&position) as i32) < 0 || <IVec2 as VectorImpl>::x(&position) as u32 >= <UVec2 as VectorImpl>::x(&self.size) as u32 || <IVec2 as VectorImpl>::y(&position) as u32 >= <UVec2 as VectorImpl>::y(&self.size) as u32 { return None; };
                return self.tiles.get_mut(<IVec2 as VectorImpl>::x(&position) as usize + <IVec2 as VectorImpl>::y(&position) as usize * <UVec2 as VectorImpl>::x(&self.size) as usize).and_then(|tile| tile.as_mut());
            }
        });
    } else {
        generate_impl!(code trait "traits::IndexableLayer" for layer_type_name => {
            type Tile = tile_type_name;

            fn get(&self, position: IVec2) -> Option<&Self::Tile> {
                if (<IVec2 as VectorImpl>::x(&position) as i32) < 0 || (<IVec2 as VectorImpl>::y(&position) as i32) < 0 || <IVec2 as VectorImpl>::x(&position) as u32 >= <UVec2 as VectorImpl>::x(&self.size) as u32 || <IVec2 as VectorImpl>::y(&position) as u32 >= <UVec2 as VectorImpl>::y(&self.size) as u32 { return None; };
                return self.tiles.get(<IVec2 as VectorImpl>::x(&position) as usize + <IVec2 as VectorImpl>::y(&position) as usize * <UVec2 as VectorImpl>::x(&self.size) as usize);
            }

            fnmut get_mut(&mut self, position: IVec2) -> Option<&mut Self::Tile> {
                if (<IVec2 as VectorImpl>::x(&position) as i32) < 0 || (<IVec2 as VectorImpl>::y(&position) as i32) < 0 || <IVec2 as VectorImpl>::x(&position) as u32 >= <UVec2 as VectorImpl>::x(&self.size) as u32 || <IVec2 as VectorImpl>::y(&position) as u32 >= <UVec2 as VectorImpl>::y(&self.size) as u32 { return None; };
                return self.tiles.get_mut(<IVec2 as VectorImpl>::x(&position) as usize + <IVec2 as VectorImpl>::y(&position) as usize * <UVec2 as VectorImpl>::x(&self.size) as usize);
            }
        });
    }
    generate_impl!(code trait "std::ops::Index<IVec2>" for layer_type_name => {
        type Output = "<Self as traits::IndexableLayer>::Tile";

        fn index(&self, position: IVec2) -> &Self::Output {
            use traits::IndexableLayer;;
            return self.get(position).unwrap();
        }
    });
    generate_impl!(code trait "std::ops::IndexMut<IVec2>" for layer_type_name => {
        fnmut index_mut(&mut self, position: IVec2) -> &mut Self::Output {
            use traits::IndexableLayer;;
            return self.get_mut(position).unwrap();
        }
    });
}

pub fn impl_auto_layer(
    code: &mut Scope,
    layer_type_name: &str,
    layer_json: &LayerDefinition,
) -> Result<()> {
    generate_impl!(code trait "traits::AutoLayer" for layer_type_name => {
        const TILESET_ID: TilesetID = format!("{}", layer_json.tileset_def_uid.context("No tileset UID in auto layer!")?);

        fn get_autotile(&self, position: IVec2) -> Vec<Tile> {
            if (<IVec2 as VectorImpl>::x(&position) as i32) < 0 || (<IVec2 as VectorImpl>::y(&position) as i32) < 0 || <IVec2 as VectorImpl>::x(&position) as u32 >= <UVec2 as VectorImpl>::x(&self.size) as u32 || <IVec2 as VectorImpl>::y(&position) as u32 >= <UVec2 as VectorImpl>::y(&self.size) as u32 { return Vec::new(); };
            return self.auto_tiles.get(<IVec2 as VectorImpl>::x(&position) as usize + <IVec2 as VectorImpl>::y(&position) as usize * <UVec2 as VectorImpl>::x(&self.size) as usize).cloned().unwrap_or_default();
        }
    });
    Ok(())
}
