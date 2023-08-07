pub mod entities;
pub mod int_grid;
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
    derive_rust_object!(level preferences.serde(), PartialEq, PartialOrd);

    code.raw("/* --- Layers --- */");
    for layer_json in &project.defs.layers {
        match layer_json.purple_type {
            Type::IntGrid => {
                int_grid::layer_definition(preferences, definitions, layer_json, code, &mut level)
            }
            Type::AutoLayer => todo!(),
            Type::Entities => {
                entities::layer_definition(preferences, definitions, layer_json, code, &mut level)
            }
            Type::Tiles => todo!(),
        }
    }

    for field in &project.defs.level_fields {
        let rs_type = RsFieldType::parse(field)?;
        level
            .new_field(&field.identifier, rs_type.string_type())
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

    code.raw("/* --- World --- */");
    // * World struct
    let world = code
        .new_struct("World")
        .vis("pub")
        .doc("World that contains levels, accessible by snake_case name or by index");
    derive_rust_object!(world preferences.serde(), PartialEq, PartialOrd);
    for tileset in &project.defs.tilesets {
        if tileset.rel_path.is_some() {
            world
                .new_field(
                    &preferences.to_case(&tileset.identifier, Case::Snake),
                    "Tileset",
                )
                .vis("pub");
        }
    }

    for level in &project.levels {
        world
            .new_field(
                &preferences.to_case(&level.identifier, Case::Snake),
                "Level",
            )
            .vis("pub");
    }

    // * Get level
    let world_impl = code.new_impl("World");
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
    for tileset in &project.defs.tilesets {
        if let Some(path) = &tileset.rel_path {
            world.line(format!(
                "{}: Tileset::new({}, \"{}\".into()),",
                &preferences.to_case(&tileset.identifier, Case::Snake),
                tileset.uid,
                path
            ));
        }
    }
    for level_json in &project.levels {
        let mut level_rs = Block::new(&format!(
            "{}: Level",
            preferences.to_case(&level_json.identifier, Case::Snake)
        ));
        level_rs.after(",");
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
            match &definitions.layers[&layer_json.identifier] {
                RsLayerDefinition::IntGrid(definition) => {
                    int_grid::layer_instance(definition, definitions, &mut layer_rs, layer_json)?
                }
                RsLayerDefinition::Entities => {
                    entities::layer_instance(definitions, &mut layer_rs, layer_json)?
                }
            }
            layer_rs.after(",");
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
                &field.identifier,
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
    //     "WORLD",
    //     "&Self",
    //     {
    //         let mut world_string = String::new();
    //         world.fmt(&mut codegen::Formatter::new(&mut world_string))?;
    //         world_string
    //     },
    //     "pub",
    // );

    Ok(())
}
