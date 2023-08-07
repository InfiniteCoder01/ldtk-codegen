use crate::definitions::*;

pub fn layer_definition(
    preferences: &Preferences,
    definitions: &mut RsDefinitions,
    layer_json: &LayerDefinition,
    code: &mut Scope,
    level: &mut codegen::Struct,
) {
    let layer_type_name = &layer_json.identifier;

    // * Tiles
    let tile_type_name = format!("{}Tile", &layer_type_name);
    let mut tile_variants = Vec::new();
    for (index, cell_value) in layer_json.int_grid_values.iter().enumerate() {
        tile_variants.push(
            cell_value
                .identifier
                .clone()
                .unwrap_or_else(|| format!("Tile{}", index)),
        );
    }

    code.raw(&format!(
        r#"layer::generate_int_grid_layer!(
    {}{layer_type_name}:
        grid_size = {},
        guide_grid_size = {},
        px_offset = {},
        parallax_factor = {},
{}
    {tile_type_name}:
        {}
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
        if !layer_json.auto_rule_groups.is_empty() {
            "\n        !auto_layer auto_tiles\n"
        } else {
            ""
        },
        tile_variants.join(",\n        "),
    ));

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
            &preferences.to_case(&layer_json.identifier, Case::Snake),
            layer_type_name,
        )
        .vis("pub");
}

pub fn layer_instance(
    definition: &RsIntGridDefinition,
    definitions: &RsDefinitions,
    layer_rs: &mut Block,
    layer_json: &LayerInstance,
) -> Result<()> {
    layer_rs.line(format!("size: {},", fmt_vec!(layer_json.c wid/hei)));
    let mut tiles = Block::new("tiles: vec!");
    for tile in &layer_json.int_grid_csv {
        tiles.line(format!(
            "{}::{},",
            definition.tile_enum,
            definition.tile_variants[*tile as usize].clone()
        ));
    }
    tiles.after(",");
    layer_rs.push_block(tiles);
    if definition.auto_layer.is_some() {
        let tileset_id = layer_json
            .tileset_def_uid
            .context("Tileset UID is not present in autotiled int grid!")?;
        layer_rs.line(format!("tileset: {},", tileset_id));
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
                "Tile::new(Vec2::new({}, {}), {})",
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
