use crate::definitions::*;

pub fn layer_definition(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    layer_json: &LayerDefinition,
    code: &mut Scope,
    level: &mut codegen::Struct,
) -> Result<()> {
    let layer_type_name = &layer_json.identifier;

    // * Tiles
    let tile_type_name = format!("{}Tile", &layer_type_name);
    let tile_enum = code.new_enum(&tile_type_name).vis("pub");
    derive_rust_object!(tile_enum preferences.serde, Copy, Default, Hash !partial Eq, Ord);
    tile_enum.new_variant("Empty").annotation("#[default]");

    let mut tile_variants = std::collections::HashMap::new();
    for (index, cell_value) in layer_json.int_grid_values.iter().enumerate() {
        let tile_name = cell_value
            .identifier
            .clone()
            .unwrap_or_else(|| format!("Tile{}", index));
        tile_enum.new_variant(&tile_name);
        tile_variants.insert(cell_value.value, tile_name);
    }

    let layer_struct = code.new_struct(layer_type_name).vis("pub");
    derive_rust_object!(layer_struct preferences.serde,);
    layer_struct.new_field("size", "UVec2").vis("pub");
    layer_struct
        .new_field("tiles", format!("Vec<{tile_type_name}>"))
        .vis("pub");

    if !layer_json.auto_rule_groups.is_empty() {
        layer_struct.new_field("auto_tiles", "Vec<Vec<Tile>>");
    }

    super::impl_layer_trait(code, layer_type_name, layer_json);
    super::impl_indexable_layer(code, layer_type_name, &tile_type_name, false);
    code.new_impl(layer_type_name).impl_trait("traits::IntGrid");

    if !layer_json.auto_rule_groups.is_empty() {
        super::impl_auto_layer(code, layer_type_name, layer_json)?;
    }

    // * Update definitions
    tile_variants.insert(0, "Empty".to_owned());
    definitions.layers.insert(
        layer_type_name.clone(),
        RsLayerDefinition::IntGrid(RsIntGridDefinition {
            grid_size: layer_json.grid_size as u32,
            tile_variants,
            tile_enum: tile_type_name,
            auto_layer: if !layer_json.auto_rule_groups.is_empty() {
                Some(RsAutoLayerDefinition {})
            } else {
                None
            },
        }),
    );

    level
        .new_field(
            preferences.to_case(&layer_json.identifier, Case::Snake),
            layer_type_name,
        )
        .vis("pub");

    Ok(())
}

pub fn layer_instance(
    definition: &RsIntGridDefinition,
    definitions: &RsDefinitions,
    layer_rs: &mut Block,
    layer_json: &LayerInstance,
) -> Result<()> {
    layer_rs.line(format!("size: <UVec2 as VectorImpl>::new({} as _, {} as _),", layer_json.c_wid, layer_json.c_hei));
    let mut tiles = Block::new("tiles: vec!");
    for tile in &layer_json.int_grid_csv {
        tiles.line(format!(
            "{}::{},",
            definition.tile_enum,
            definition.tile_variants[tile].clone()
        ));
    }
    tiles.after(",");
    layer_rs.push_block(tiles);
    if definition.auto_layer.is_some() {
        let tileset_id = layer_json
            .tileset_def_uid
            .context("Tileset UID is not present in autotiled int grid!")?;
        let tileset = definitions
            .tilesets
            .get(&tileset_id)
            .context("Tileset from autotiled int grid was not found!")?;
        let mut tiles = vec![Vec::new(); layer_json.c_wid as usize * layer_json.c_hei as usize];
        for tile in &layer_json.auto_layer_tiles {
            let tile_pos = (
                tile.px[0] as usize / definition.grid_size as usize,
                tile.px[1] as usize / definition.grid_size as usize,
            );
            let tileset_tile = (
                tile.src[0] as u32 / tileset.tile_size,
                tile.src[1] as u32 / tileset.tile_size,
            );
            tiles[tile_pos.0 + tile_pos.1 * layer_json.c_wid as usize].push(format!(
                "Tile::new(<UVec2 as VectorImpl>::new({} as _, {} as _), {})",
                tileset_tile.0,
                tileset_tile.1,
                match tile.f {
                    0 => "FlipMode::None",
                    1 => "FlipMode::Horizontal",
                    2 => "FlipMode::Vertical",
                    3 => "FlipMode::Both",
                    _ => bail!("Invalid flip mode: {}", tile.f),
                }
            ));
        }
        let mut auto_tiles = Block::new("auto_tiles: vec!");
        for tiles in tiles {
            auto_tiles.line(format!("vec![{}],", tiles.join(", ")));
        }
        auto_tiles.after(",");
        layer_rs.push_block(auto_tiles);
    }
    Ok(())
}
