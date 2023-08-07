use crate::definitions::*;

pub fn layer_definition(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    layer_json: &LayerDefinition,
    code: &mut Scope,
    level: &mut codegen::Struct,
) -> Result<()> {
    let layer_type_name = &layer_json.identifier;

    code.raw(&format!(
        r#"layer::generate_tiles_layer!(
    {}{layer_type_name}:
        grid_size = {},
        guide_grid_size = {},
        px_offset = {},
        parallax_factor = {},
        tileset = {},
);"#,
        layer_json
            .doc
            .as_ref()
            .map(|doc| format!("!doc \"{doc}\"\n    "))
            .unwrap_or_default(),
        layer_json.grid_size,
        fmt_vec!(layer_json.guide_grid wid/hei),
        fmt_vec!(layer_json.px_offset),
        fmt_vec!(!float layer_json.parallax_factor),
        layer_json
            .tileset_def_uid
            .context("Tiles layer doesn't have tileset!")?,
    ));

    // * Update definitions
    definitions.layers.insert(
        layer_type_name.clone(),
        RsLayerDefinition::Tiles(RsTilesDefinition {
            grid_size: layer_json.grid_size as u32,
        }),
    );

    level
        .new_field(
            &preferences.to_case(&layer_json.identifier, Case::Snake),
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

    layer_rs.line(format!("size: {},", fmt_vec!(layer_json.c wid/hei)));
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
            "Some(Tile::new(Vec2::new({}, {}), {}))",
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
