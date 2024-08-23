use crate::definitions::*;

pub fn layer_definition(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    layer_json: &LayerDefinition,
    code: &mut Scope,
    level: &mut codegen::Struct,
) -> Result<()> {
    let layer_type_name = &layer_json.identifier;

    let layer_struct = code.new_struct(layer_type_name).vis("pub");
    derive_rust_object!(layer_struct preferences.serde,);
    layer_struct.new_field("size", "UVec2").vis("pub");
    layer_struct
        .new_field("tiles", "Vec<Option<Tile>>".to_owned())
        .vis("pub");

    super::impl_layer_trait(code, layer_type_name, layer_json);
    super::impl_indexable_layer(code, layer_type_name, "Tile", true);
    code.new_impl(layer_type_name)
        .impl_trait("traits::Tiles")
        .associate_const(
            "TILESET_ID",
            "TilesetID",
            format!(
                "{}",
                layer_json
                    .tileset_def_uid
                    .context("Tiles layer doesn't have a tileset ID!")?
            ),
            "",
        );

    // * Update definitions
    definitions.layers.insert(
        layer_type_name.clone(),
        RsLayerDefinition::Tiles(RsTilesDefinition {
            grid_size: layer_json.grid_size as u32,
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
    definition: &RsTilesDefinition,
    definitions: &RsDefinitions,
    layer_rs: &mut Block,
    layer_json: &LayerInstance,
) -> Result<()> {
    let tileset_id = layer_json
        .tileset_def_uid
        .context("Tileset UID is not present in autotiled int grid!")?;
    let tileset = definitions
        .tilesets
        .get(&tileset_id)
        .context("Tileset from autotiled int grid was not found!")?;

    layer_rs.line(format!("size: <UVec2 as VectorImpl>::new({} as _, {} as _),", layer_json.c_wid, layer_json.c_hei));
    let mut tiles = vec!["None".to_owned(); layer_json.c_wid as usize * layer_json.c_hei as usize];
    for tile in &layer_json.grid_tiles {
        let tile_pos = (
            tile.px[0] as usize / definition.grid_size as usize,
            tile.px[1] as usize / definition.grid_size as usize,
        );
        let tileset_tile = (
            tile.src[0] as u32 / tileset.tile_size,
            tile.src[1] as u32 / tileset.tile_size,
        );
        tiles[tile_pos.0 + tile_pos.1 * layer_json.c_wid as usize] = format!(
            "Some(Tile::new(<UVec2 as VectorImpl>::new({} as _, {} as _), {}))",
            tileset_tile.0,
            tileset_tile.1,
            match tile.f {
                0 => "FlipMode::None",
                1 => "FlipMode::Horizontal",
                2 => "FlipMode::Vertical",
                3 => "FlipMode::Both",
                _ => bail!("Invalid flip mode: {}", tile.f),
            }
        );
    }
    let mut tiles_field = Block::new("tiles: vec!");
    for tiles in tiles {
        tiles_field.line(format!("{},", tiles));
    }
    tiles_field.after(",");
    layer_rs.push_block(tiles_field);
    Ok(())
}
