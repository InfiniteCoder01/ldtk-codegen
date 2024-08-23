use crate::definitions::*;

pub fn definitions(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    project: &LdtkJson,
    code: &mut Scope,
) -> Result<()> {
    code.raw("/* --- Entities --- */");
    let mut entity = codegen::Enum::new("Entity");
    entity.vis("pub");
    derive_rust_object!(entity preferences.serde,);
    for entity_json in &project.defs.entities {
        let entity_rs = code.new_struct(&entity_json.identifier);
        entity_rs.vis("pub");
        derive_rust_object!(entity_rs preferences.serde,);
        let mut entity_definition = RsEntityDefinition::default();

        for field in &entity_json.field_defs {
            let rs_type = RsFieldType::parse(field)?;
            entity_rs
                .new_field(&field.identifier, rs_type.string_type())
                .vis("pub");
            entity_definition
                .fields
                .insert(field.identifier.clone(), rs_type);
        }
        entity
            .new_variant(&entity_json.identifier)
            .tuple(&entity_json.identifier);
        definitions
            .entities
            .insert(entity_json.identifier.clone(), entity_definition);
    }
    code.push_enum(entity);

    let entity = code.new_impl("Entity");
    macro_rules! generate_get_const {
        ($fn:ident -> $ret:ty; $variant:ident => $line:expr) => {
            entity
                .new_fn(stringify!($fn))
                .vis("pub")
                .arg_ref_self()
                .ret(stringify!($ret))
                .push_block({
                    let mut match_block = Block::new("match self");
                    if project.defs.entities.is_empty() {
                        match_block.line("_ => unreachable!()");
                    } else {
                        for $variant in &project.defs.entities {
                            match_block
                                .line(format!("Self::{}(_) => {},", $variant.identifier, $line));
                        }
                    }
                    match_block
                });
        };
    }

    generate_get_const!(pivot -> FVec2; variant => format!("<FVec2 as VectorImpl>::new({} as _, {} as _)", variant.pivot_x, variant.pivot_y));
    generate_get_const!(render_mode -> RenderMode; variant => match variant.render_mode {
        RenderMode::Cross => "RenderMode::Cross".to_owned(),
        RenderMode::Ellipse => "RenderMode::Ellipse".to_owned(),
        RenderMode::Rectangle => "RenderMode::Rectangle".to_owned(),
        RenderMode::Tile => {
            let rect = variant.tile_rect.as_ref().context("Tile render mode doesn't have tile rect!")?;
            let tileset = variant.tileset_id.context("Tile render mode doesn't have tileset ID!")?;
            let tile_size = definitions.tilesets.get(&tileset).context("Entity tileset not found!")?.tile_size;
            format!(
                "RenderMode::Tile {{ tileset: {}, tile: <UVec2 as VectorImpl>::new({} as _, {} as _), size: <UVec2 as VectorImpl>::new({} as _, {} as _) }}",
                tileset,
                rect.x as u32 / tile_size,
                rect.y as u32 / tile_size,
                rect.w as u32 / tile_size,
                rect.h as u32 / tile_size,
            )
        },
    });

    Ok(())
}

pub fn layer_definition(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    layer_json: &LayerDefinition,
    code: &mut Scope,
    level: &mut codegen::Struct,
) {
    let layer_type_name = &layer_json.identifier;

    let layer_struct = code.new_struct(layer_type_name).vis("pub");
    derive_rust_object!(layer_struct preferences.serde,);
    layer_struct.new_field("size", "UVec2").vis("pub");
    layer_struct
        .new_field("entities", "Vec<EntityObject>".to_owned())
        .vis("pub");

    super::impl_layer_trait(code, layer_type_name, layer_json);

    generate_impl!(code trait "traits::Entities" for layer_type_name => {
        fn entities(&self) -> &Vec<EntityObject> {
            return &self.entities;
        }

        fnmut entities_mut(&mut self) -> &mut Vec<EntityObject> {
            return &mut self.entities;
        }
    });

    generate_impl!(code trait "std::ops::Deref" for layer_type_name => {
        type Target = "Vec<EntityObject>";

        fn deref(&self) -> &Self::Target {
            use traits::Entities;;
            return self.entities();
        }
    });

    generate_impl!(code trait "std::ops::DerefMut" for layer_type_name => {
        fnmut deref_mut(&mut self) -> &mut Self::Target {
            use traits::Entities;;
            return self.entities_mut();
        }
    });

    // * Update definitions
    definitions
        .layers
        .insert(layer_type_name.clone(), RsLayerDefinition::Entities);

    level
        .new_field(
            preferences.to_case(&layer_json.identifier, Case::Snake),
            layer_type_name,
        )
        .vis("pub");
}

pub fn layer_instance(
    definitions: &RsDefinitions,
    layer_rs: &mut Block,
    layer_json: &LayerInstance,
) -> Result<()> {
    layer_rs.line(format!(
        "size: <UVec2 as VectorImpl>::new({} as _, {} as _),",
        layer_json.c_wid, layer_json.c_hei
    ));
    let mut entities = Block::new("entities: vec!");
    for entity in &layer_json.entity_instances {
        let definition = definitions
            .entities
            .get(&entity.identifier)
            .context("Entity from level was not found!")?;
        let mut instance = Block::new(&format!(
            "EntityObject::new(Entity::{}({}",
            entity.identifier, entity.identifier,
        ));

        for field in &entity.field_instances {
            let field_type = definition.fields.get(&field.identifier).context(format!(
                "Entity field was not found in definition ({})!",
                &field.identifier
            ))?;
            instance.line(format!(
                "{}: {},",
                &field.identifier,
                field_type.fmt_value(definitions, field.value.as_ref())?
            ));
        }

        instance.after(&format!(
            "), <FVec2 as VectorImpl>::new({} as _, {} as _), <UVec2 as VectorImpl>::new({} as _, {} as _)),",
            entity.px[0], entity.px[1],
            entity.width, entity.height
        ));
        entities.push_block(instance);
    }
    layer_rs.push_block(entities);
    Ok(())
}
